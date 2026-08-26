"""Micro-Crucible FastAPI Application.

Intentionally broken, chaos-capable target sandbox backend for QA/SDET drills.
"""

import asyncio
from datetime import datetime, timedelta, timezone
import json
import math
from pathlib import Path
import random
import re
import time
from typing import Any

from fastapi import FastAPI, File, HTTPException, Request, UploadFile
from fastapi.middleware.cors import CORSMiddleware
from fastapi.responses import HTMLResponse, JSONResponse, StreamingResponse
import jwt

from crucible.backend.chaos import ChaosMiddleware
from crucible.backend.curriculum import ManifestError, build_curriculum
from crucible.backend.models import (
    AllureSummaryResponse,
    BalanceResponse,
    ChaosTestResultItem,
    CheckoutRequest,
    CheckoutResponse,
    CheckoutStateResponse,
    GraphQLRequest,
    HealthResponse,
    LlmEntities,
    LlmResponse,
    LoginRequest,
    LoginResponse,
    PipelineRunRequest,
    PipelineRunResult,
    PipelineValidateRequest,
    PipelineValidation,
    ProductItem,
    ProductListResponse,
    RagResponse,
    ResetResponse,
    ReviewFixRequest,
    ReviewFixResponse,
    ReviewReport,
    ReviewRequest,
    SearchResponse,
    TransferRequest,
    TransferResponse,
    TriageResultResponse,
    TriageSubmissionRequest,
    UploadResponse,
    UserMeResponse,
)
from crucible.backend.pipeline import simulate_pipeline_run, validate_workflow_yaml
from crucible.backend.reports import generate_chaos_dataset, render_html_report_string, summarize_dataset
from crucible.backend.review import apply_review_fix, run_code_review
from crucible.backend.triage import evaluate_triage_submission
from crucible.backend.ai_generator import generate_pytest_from_openapi

# JWT configuration
import os as _os
SECRET_KEY = _os.getenv("CRUCIBLE_JWT_SECRET", "cherenkov-crucible-secret-key-2026")
ALGORITHM = "HS256"
_ledger_lock = asyncio.Lock()
_progress_lock = asyncio.Lock()
_MAX_UPLOAD_BYTES = 5 * 1024 * 1024

# In-memory Ledger State
DEFAULT_ACCOUNTS: dict[str, float] = {
    "ACC-001": 1000.00,
    "ACC-002": 500.00,
}
accounts: dict[str, float] = dict(DEFAULT_ACCOUNTS)
pending_transfers: list[dict[str, Any]] = []

# Search Autocomplete Catalog
SEARCH_CATALOG: list[str] = [
    "Playwright",
    "Playwright TypeScript",
    "Playwright Python",
    "Playwright Java",
    "Playwright C#",
    "Playground",
    "Playbook",
    "Platform",
    "Plugin",
    "Playlist",
    "Python",
    "PHP",
    "Perl",
    "PostgreSQL",
    "PowerShell",
]

# Embedded Payment Frame HTML
HTML_PAYMENT_FRAME = """<!DOCTYPE html>
<html lang="en">
<head>
  <meta charset="UTF-8">
  <title>Crucible Secure Gateway</title>
  <style>
    body {
      font-family: -apple-system, BlinkMacSystemFont, "Segoe UI", Roboto, sans-serif;
      background: #0f172a;
      color: #f8fafc;
      margin: 0;
      padding: 12px;
      box-sizing: border-box;
    }
    h4 {
      margin: 0 0 8px 0;
      color: #38bdf8;
      font-size: 14px;
    }
    label {
      font-size: 12px;
      color: #94a3b8;
      display: block;
      margin-bottom: 4px;
    }
    input {
      width: 100%;
      max-width: 200px;
      padding: 6px 8px;
      background: #1e293b;
      border: 1px solid #475569;
      border-radius: 4px;
      color: #fff;
      font-size: 13px;
      margin-bottom: 8px;
      box-sizing: border-box;
    }
    input:focus {
      outline: none;
      border-color: #38bdf8;
    }
    button {
      display: block;
      padding: 6px 14px;
      background: #0284c7;
      color: #ffffff;
      border: none;
      border-radius: 4px;
      font-size: 13px;
      font-weight: 500;
      cursor: pointer;
      transition: background 0.15s;
    }
    button:hover {
      background: #0369a1;
    }
    #frame-auth-status {
      margin-top: 8px;
      font-size: 13px;
      font-weight: 600;
      color: #4ade80;
      display: none;
    }
  </style>
</head>
<body>
  <h4>Crucible Secure Payment Gateway (Frame)</h4>
  <label for="secure-card-pin">Security PIN:</label>
  <input id="secure-card-pin" data-testid="secure-card-pin" type="password" placeholder="1234" />
  <button id="btn-authorize" data-testid="btn-authorize">Authorize Payment</button>
  <div id="frame-auth-status" data-testid="frame-auth-status">Payment Authorized</div>
  <script>
    document.getElementById('btn-authorize').addEventListener('click', function() {
      document.getElementById('frame-auth-status').style.display = 'block';
    });
  </script>
</body>
</html>
"""

HTML_CHECKOUT_FRAME = """<!DOCTYPE html>
<html lang="en">
<head>
  <meta charset="UTF-8">
  <title>Checkout Gateway Frame</title>
  <style>
    body {
      font-family: -apple-system, BlinkMacSystemFont, "Segoe UI", Roboto, sans-serif;
      background: #0f172a;
      color: #f8fafc;
      margin: 0;
      padding: 12px;
      box-sizing: border-box;
    }
    h4 {
      margin: 0 0 10px 0;
      color: #38bdf8;
      font-size: 14px;
    }
    .field {
      margin-bottom: 10px;
    }
    label {
      font-size: 12px;
      color: #94a3b8;
      display: block;
      margin-bottom: 4px;
    }
    input {
      width: 100%;
      max-width: 220px;
      padding: 6px 8px;
      background: #1e293b;
      border: 1px solid #475569;
      border-radius: 4px;
      color: #fff;
      font-size: 13px;
      box-sizing: border-box;
    }
    input:focus {
      outline: none;
      border-color: #38bdf8;
    }
    button {
      display: inline-block;
      padding: 7px 16px;
      background: #0284c7;
      color: #ffffff;
      border: none;
      border-radius: 4px;
      font-size: 13px;
      font-weight: 500;
      cursor: pointer;
      transition: background 0.15s;
    }
    button:hover {
      background: #0369a1;
    }
    #payment-status {
      margin-top: 10px;
      font-size: 13px;
      font-weight: 600;
      display: none;
    }
    #payment-status.ok { color: #4ade80; }
    #payment-status.err { color: #f87171; }
  </style>
</head>
<body>
  <h4>Secure Checkout Gateway (Frame)</h4>
  <div class="field">
    <label for="card-number">Card Number:</label>
    <input id="card-number" type="text" placeholder="4242 4242 4242 4242" />
  </div>
  <div class="field">
    <label for="card-expiry">Expiry Date (MM/YY):</label>
    <input id="card-expiry" type="text" placeholder="12/28" />
  </div>
  <button id="btn-submit-payment">Submit Payment</button>
  <div id="payment-status"></div>
  <script>
    document.getElementById('btn-submit-payment').addEventListener('click', function() {
      var card = document.getElementById('card-number').value.trim();
      var expiry = document.getElementById('card-expiry').value.trim();
      var status = document.getElementById('payment-status');
      if (card.length >= 12 && /\\d{2}\\/\\d{2}/.test(expiry)) {
        status.textContent = 'Payment Authorized - Success';
        status.className = 'ok';
      } else {
        status.textContent = 'Error: invalid card details';
        status.className = 'err';
      }
      status.style.display = 'block';
    });
  </script>
</body>
</html>
"""


def settle_pending_transfers() -> None:
    """Settle all pending Kafka ledger transfers whose lag delay has elapsed."""
    global accounts, pending_transfers
    now = time.time()
    remaining: list[dict[str, Any]] = []

    for t in pending_transfers:
        if now >= t["settle_time"]:
            from_acc = t["from_account"]
            to_acc = t["to_account"]
            amount = t["amount"]
            if accounts.get(from_acc, 0.0) >= amount:
                accounts[from_acc] = round(accounts[from_acc] - amount, 2)
                accounts[to_acc] = round(accounts.get(to_acc, 0.0) + amount, 2)
        else:
            remaining.append(t)

    pending_transfers = remaining


# Create FastAPI instance
app = FastAPI(
    title="Micro-Crucible Chaos Backend",
    version="1.0.0",
    description="Intentionally broken, chaos-capable target sandbox backend for QA/SDET drills",
)

# Attach Middlewares
# The documented local setup (React on 8080, this API on 8081). Overridable via
# CRUCIBLE_ALLOWED_ORIGINS as a comma-separated list, so a run on other ports --
# CI, or a box where 8080 is taken -- does not require editing this file. The
# default stays a closed allowlist; there is no wildcard fallback.
_DEFAULT_ALLOWED_ORIGINS = [
    "http://localhost:8080",
    "http://127.0.0.1:8080",
    "http://localhost:8081",
    "http://127.0.0.1:8081",
]

_ALLOWED_ORIGINS = [
    origin.strip()
    for origin in _os.environ.get("CRUCIBLE_ALLOWED_ORIGINS", "").split(",")
    if origin.strip()
] or _DEFAULT_ALLOWED_ORIGINS

app.add_middleware(
    CORSMiddleware,
    allow_origins=_ALLOWED_ORIGINS,
    allow_credentials=True,
    allow_methods=["*"],
    allow_headers=["*"],
    expose_headers=["X-Chaos-Stale-DOM", "X-Chaos"],
)
app.add_middleware(ChaosMiddleware)


@app.get("/", response_model=HealthResponse)
@app.get("/health", response_model=HealthResponse)
async def get_health() -> HealthResponse:
    """Service health and identification check."""
    return HealthResponse(
        status="ok", service="micro-crucible-backend", version="1.0.0"
    )


@app.get("/checkout", response_model=CheckoutStateResponse)
@app.get("/api/checkout", response_model=CheckoutStateResponse)
async def get_checkout() -> CheckoutStateResponse:
    """Retrieve current shopping cart and pricing breakdown."""
    return CheckoutStateResponse()


@app.post("/checkout", response_model=CheckoutResponse)
@app.post("/api/checkout", response_model=CheckoutResponse)
async def post_checkout(
    request: Request,
    req: CheckoutRequest | None = None,
) -> Any:
    """Execute order checkout payment."""
    chaos = getattr(request.state, "chaos", {})
    if chaos.get("idempotency_conflict"):
        return JSONResponse(
            status_code=409,
            content={
                "status": "conflict",
                "error": "IDEMPOTENCY_CONFLICT",
                "message": "Order conflict: duplicate transaction detected with idempotency key",
                "order_id": "ORD-4821",
            },
        )
    order_num = random.randint(10000, 99999)
    return CheckoutResponse(
        status="success",
        order_id=f"ORD-{order_num}",
        message="Order Confirmed",
        total_charged=160.92,
        timestamp=int(time.time()),
    )


@app.get("/balance", response_model=BalanceResponse)
async def get_balance(account_id: str = "ACC-001") -> BalanceResponse:
    """Inquire bank account balance after settling matured Kafka ledger transfers."""
    async with _ledger_lock:
        settle_pending_transfers()
        if account_id not in accounts:
            raise HTTPException(status_code=404, detail=f"Account '{account_id}' not found")
        pending_cnt = sum(1 for t in pending_transfers if t["from_account"] == account_id or t["to_account"] == account_id)
        bal = accounts[account_id]
    return BalanceResponse(account_id=account_id, balance=bal, pending_count=pending_cnt, currency="USD")


@app.post("/transfer", response_model=TransferResponse)
async def post_transfer(req: TransferRequest, request: Request) -> TransferResponse:
    """Initiate an async ledger transfer subject to Kafka lag."""
    global accounts, pending_transfers
    async with _ledger_lock:
        settle_pending_transfers()
        if req.from_account not in accounts:
            raise HTTPException(status_code=404, detail=f"Source account '{req.from_account}' not found")
        if req.to_account not in accounts:
            raise HTTPException(status_code=404, detail=f"Destination account '{req.to_account}' not found")
        if accounts[req.from_account] < req.amount:
            raise HTTPException(status_code=400, detail="Insufficient funds")
        chaos = getattr(request.state, "chaos", {})
        kafka_lag_ms = float(chaos.get("kafka_lag", 1500.0 if "kafka_lag" in chaos else 0.0))
        transfer_id = f"TX-{random.randint(10000, 99999)}"
        if kafka_lag_ms > 0.0:
            settle_time = time.time() + (kafka_lag_ms / 1000.0)
            pending_transfers.append({"transfer_id": transfer_id, "from_account": req.from_account, "to_account": req.to_account, "amount": req.amount, "settle_time": settle_time})
        else:
            accounts[req.from_account] = round(accounts[req.from_account] - req.amount, 2)
            accounts[req.to_account] = round(accounts[req.to_account] + req.amount, 2)
    return TransferResponse(status="QUEUED_LEDGER", transfer_id=transfer_id, amount=req.amount, lag_ms=kafka_lag_ms, message="Transfer queued in Kafka topic ledger-events")


@app.post("/reset", response_model=ResetResponse)
async def post_reset() -> ResetResponse:
    """Reset in-memory bank ledger balances and pending transfers."""
    global accounts, pending_transfers
    async with _ledger_lock:
        accounts = dict(DEFAULT_ACCOUNTS)
        pending_transfers = []
    return ResetResponse(status="ok", message="Ledger and state reset to initial values")


@app.get("/search", response_model=SearchResponse)
async def get_search(request: Request, q: str = "") -> Any:
    """Debounced search autocomplete with out-of-order latency simulation."""
    chaos = getattr(request.state, "chaos", {})
    if chaos.get("db_timeout"):
        return JSONResponse(status_code=504, content={"error": "Database query timeout", "status": "error"})

    query = q.strip()
    if not query:
        return SearchResponse(query=q, results=[], count=0)

    # Inverted latency simulation: short queries take longer (800ms), long queries are fast (50ms)
    if len(query) <= 2:
        await asyncio.sleep(0.8)
    else:
        await asyncio.sleep(0.05)

    if query.lower() == "p":
        matches = ["Python", "PHP", "Perl", "PostgreSQL", "PowerShell"]
    else:
        matches = [
            item for item in SEARCH_CATALOG if query.lower() in item.lower()
        ]

    if chaos.get("dast_xss"):
        # Inject reflected XSS payload
        matches.insert(0, f"<script>alert('xss:{q}')</script>")

    return SearchResponse(query=q, results=matches, count=len(matches))


@app.post("/auth/login", response_model=LoginResponse)
async def auth_login(req: LoginRequest, request: Request) -> LoginResponse:
    """Authenticate and issue JWT token with optional token_expire chaos."""
    chaos = getattr(request.state, "chaos", {})
    token_expire_directive = chaos.get("token_expire")

    now = datetime.now(timezone.utc)
    if token_expire_directive == "immediate":
        exp = now - timedelta(seconds=10)
        expires_in = 0
    else:
        exp = now + timedelta(hours=1)
        expires_in = 3600

    payload = {
        "sub": req.username,
        "role": "sdet_engineer",
        "exp": exp,
        "iat": now,
    }
    token = jwt.encode(payload, SECRET_KEY, algorithm=ALGORITHM)
    return LoginResponse(
        access_token=token, token_type="bearer", expires_in=expires_in
    )


@app.get("/auth/me", response_model=UserMeResponse)
async def auth_me(request: Request) -> UserMeResponse:
    """Retrieve current authenticated user from Bearer JWT token."""
    auth_header = request.headers.get("authorization")
    if not auth_header or not auth_header.startswith("Bearer "):
        raise HTTPException(
            status_code=401, detail="Missing or invalid authorization header"
        )

    token = auth_header.split(" ", 1)[1].strip()
    try:
        payload = jwt.decode(token, SECRET_KEY, algorithms=[ALGORITHM])
    except jwt.ExpiredSignatureError:
        raise HTTPException(status_code=401, detail="Token expired mid-session")
    except jwt.InvalidTokenError:
        raise HTTPException(status_code=401, detail="Invalid token")

    return UserMeResponse(
        user_id="usr-4819",
        username=str(payload.get("sub", "sdet_student")),
        role=str(payload.get("role", "sdet_engineer")),
        status="active",
    )


@app.get("/embed/payment-frame", response_class=HTMLResponse)
async def get_payment_frame() -> HTMLResponse:
    """Serve cross-origin payment iframe HTML."""
    return HTMLResponse(content=HTML_PAYMENT_FRAME, status_code=200)


@app.get("/embed/checkout-frame", response_class=HTMLResponse)
async def get_checkout_frame() -> HTMLResponse:
    """Serve full card checkout iframe HTML (card number, expiry, submit)."""
    return HTMLResponse(content=HTML_CHECKOUT_FRAME, status_code=200)


# ---------------------------------------------------------------------------
# GenAI QA Mock Endpoints (Sprint 3)
# ---------------------------------------------------------------------------

# Source document facts — used by drills to verify RAG faithfulness
RAG_SOURCE_DOCUMENT = {
    "title": "Cherenkov Radiation Primer",
    "facts": [
        "Cherenkov radiation occurs when a charged particle moves faster than light in a medium.",
        "The radiation was discovered by Pavel Cherenkov in 1934.",
        "It appears as a characteristic blue glow in nuclear reactors.",
        "The angle of emission depends on the particle velocity and the refractive index.",
    ],
}

# Response variants — simulate minor LLM rephrasing on repeated calls
_LLM_RESPONSE_VARIANTS = [
    "The transfer was successfully initiated and is pending ledger settlement.",
    "Your transfer request has been queued and will settle shortly.",
    "Transfer initiated — the ledger update is in progress.",
    "The requested transfer is now pending confirmation in the ledger system.",
]
_llm_call_counter = 0


@app.get("/api/rag", response_model=RagResponse)
async def api_rag(query: str = "") -> RagResponse:
    """Mock RAG endpoint. Returns an answer grounded in the source document.

    Drill contract:
    - Response always contains key facts from RAG_SOURCE_DOCUMENT['facts']
    - The 'source_facts' field lists which facts were used (for faithfulness checking)
    - The 'answer' field contains a natural-language response derived from those facts
    """
    trimmed = query.strip()
    if not trimmed:
        return RagResponse(
            query=query,
            answer="Please provide a search query.",
            source_facts=[],
            grounded=False,
            document_title=str(RAG_SOURCE_DOCUMENT["title"]),
        )

    # Deterministic fact selection based on query keywords
    matched_facts = [
        fact
        for fact in RAG_SOURCE_DOCUMENT["facts"]
        if any(word.lower() in fact.lower() for word in trimmed.split())
    ] or RAG_SOURCE_DOCUMENT["facts"][:2]

    selected_facts = matched_facts[:2]
    answer = (
        f"Based on '{RAG_SOURCE_DOCUMENT['title']}': "
        + " ".join(selected_facts)
    )
    return RagResponse(
        query=query,
        answer=answer,
        source_facts=selected_facts,
        grounded=True,
        document_title=str(RAG_SOURCE_DOCUMENT["title"]),
    )


@app.get("/api/llm", response_model=LlmResponse)
async def api_llm(prompt: str = "") -> LlmResponse:
    """Mock LLM endpoint. Returns structured intent/entity fields plus varied raw text.

    Drill contract:
    - 'intent' and 'entities' are STABLE across calls (for assertion-safe testing)
    - 'raw_text' VARIES slightly on each call (exposing string-equality flakiness)
    - Drills should assert on 'intent'/'entities', not 'raw_text'
    """
    global _llm_call_counter
    _llm_call_counter += 1
    variant_idx = _llm_call_counter % len(_LLM_RESPONSE_VARIANTS)

    return LlmResponse(
        prompt=prompt,
        intent="transfer_status_inquiry",
        entities=LlmEntities(
            action="transfer",
            status="pending",
            domain="ledger",
        ),
        confidence=0.94,
        raw_text=_LLM_RESPONSE_VARIANTS[variant_idx],
        model="mock-llm-v1",
    )


# ---------------------------------------------------------------------------
# R4 Crucible Backend Expansion
# ---------------------------------------------------------------------------

PRODUCT_CATALOG: list[dict[str, Any]] = [
    {
        "id": f"prod-{i:03d}",
        "name": f"SDET Automation Toolset Item {i}",
        "price": round(19.99 + (i * 7.50) % 150.0, 2),
        "category": "testing-tools" if i % 2 == 0 else "hardware",
        "in_stock": i % 7 != 0,
    }
    for i in range(1, 26)
]

GRAPHQL_USER_ENTITY: dict[str, Any] = {
    "id": "usr-4819",
    "name": "sdet_student",
    "email": "student@cherenkov.qa",
    "role": "sdet_engineer",
    "status": "active",
}


def execute_minimal_graphql(query_str: str) -> dict[str, Any]:
    """Parse and resolve minimal GraphQL queries and field aliases."""
    if not query_str or not isinstance(query_str, str):
        return {"errors": [{"message": "Empty GraphQL query."}]}

    q = query_str.strip()
    q = re.sub(r"^(query|mutation)\s*[A-Za-z0-9_]*\s*", "", q).strip()
    if q.startswith("{") and q.endswith("}"):
        q = q[1:-1].strip()

    if not q:
        return {"errors": [{"message": "Empty GraphQL selection set."}]}

    pattern = r"^(?:(?P<alias>[A-Za-z0-9_]+)\s*:\s*)?(?P<field>[A-Za-z0-9_]+)\s*\{(?P<subfields>[^}]+)\}"
    match = re.search(pattern, q, re.DOTALL)
    if not match:
        return {
            "errors": [
                {
                    "message": f"Syntax error or unsupported GraphQL query: {query_str}"
                }
            ]
        }

    alias = match.group("alias")
    field = match.group("field")
    subfields_raw = match.group("subfields")
    requested_fields = [
        f.strip() for f in re.split(r"[\s,]+", subfields_raw) if f.strip()
    ]

    if field == "user":
        data_obj = {
            k: GRAPHQL_USER_ENTITY[k]
            for k in requested_fields
            if k in GRAPHQL_USER_ENTITY
        }
        target_key = alias if alias else "user"
        return {"data": {target_key: data_obj}}

    return {
        "errors": [
            {"message": f"Cannot query field '{field}' on type 'Query'."}
        ]
    }


@app.post("/upload", response_model=UploadResponse)
async def post_upload(
    request: Request,
    file: UploadFile = File(...),
) -> Any:
    """Handle multipart file uploads with chaos partial-drop simulation."""
    chaos = getattr(request.state, "chaos", {})
    if chaos.get("drop_partial"):
        return JSONResponse(
            status_code=400,
            content={
                "status": "error",
                "error": "PARTIAL_UPLOAD_DROPPED",
                "message": "Upload aborted: connection dropped mid-transfer (partial upload simulated)",
                "bytes_received": 0,
            },
        )

    content = await file.read()
    if len(content) > _MAX_UPLOAD_BYTES:
        raise HTTPException(status_code=413, detail=f"Upload too large: {len(content)} bytes exceeds {_MAX_UPLOAD_BYTES} limit")
    return UploadResponse(
        filename=file.filename or "unknown",
        content_type=file.content_type or "application/octet-stream",
        size_bytes=len(content),
        status="uploaded",
        message="File uploaded successfully",
    )


@app.get("/products", response_model=ProductListResponse)
async def get_products(
    page: int = 1,
    per_page: int = 10,
) -> ProductListResponse:
    """Retrieve paginated product catalog."""
    safe_page = max(1, page)
    safe_per_page = max(1, min(100, per_page))
    total = len(PRODUCT_CATALOG)
    total_pages = max(1, math.ceil(total / safe_per_page))

    start_idx = (safe_page - 1) * safe_per_page
    end_idx = start_idx + safe_per_page
    items = PRODUCT_CATALOG[start_idx:end_idx] if start_idx < total else []

    return ProductListResponse(
        total=total,
        page=safe_page,
        per_page=safe_per_page,
        total_pages=total_pages,
        products=[ProductItem(**item) for item in items],
    )


@app.get("/events/stream")
async def get_events_stream(request: Request) -> StreamingResponse:
    """Server-Sent Events stream with chaos connection-drop simulation."""
    chaos = getattr(request.state, "chaos", {})
    drop_after_raw = chaos.get("drop_after")
    drop_after: int | None = None
    if drop_after_raw is not None:
        try:
            drop_after = int(drop_after_raw)
        except (ValueError, TypeError):
            drop_after = None

    async def event_generator():
        count = 0
        while True:
            if drop_after is not None and count >= drop_after:
                break
            count += 1
            payload = {
                "id": count,
                "timestamp": int(time.time()),
                "event": "tick",
                "data": f"Crucible live telemetry stream event #{count}",
            }
            yield f"id: {count}\nevent: message\ndata: {json.dumps(payload)}\n\n"
            if drop_after is not None and count >= drop_after:
                break
            await asyncio.sleep(1.0)

    return StreamingResponse(
        event_generator(),
        media_type="text/event-stream",
        headers={
            "Cache-Control": "no-cache",
            "Connection": "keep-alive",
            "X-Accel-Buffering": "no",
        },
    )


@app.post("/api/generate-tests")
async def generate_tests() -> JSONResponse:
    """Auto-generates basic Pytest endpoint validation tests."""
    code = generate_pytest_from_openapi("http://localhost:8081/openapi.json")
    return JSONResponse(content={"status": "success", "code": code})

@app.post("/graphql")
async def post_graphql(req: GraphQLRequest) -> JSONResponse:
    """Execute minimal GraphQL query with field aliases."""
    result = execute_minimal_graphql(req.query)
    status_code = 400 if "errors" in result and "data" not in result else 200
    return JSONResponse(status_code=status_code, content=result)


@app.get("/api/llm/stream")
async def stream_llm_tokens(prompt: str = "Explain test automation") -> StreamingResponse:
    """Simulates LLM streaming response with time-to-first-token (TTFT) and token intervals."""
    tokens = [
        "Test", " automation", " provides", " rapid", " deterministic",
        " feedback", " on", " software", " quality", " across", " all", " regression", " suites."
    ]

    async def token_generator():
        # TTFT: Initial inference latency
        await asyncio.sleep(0.3)
        for idx, token in enumerate(tokens):
            chunk = {"index": idx, "token": token, "done": idx == len(tokens) - 1}
            yield f"data: {json.dumps(chunk)}\n\n"
            await asyncio.sleep(0.05)

    return StreamingResponse(
        token_generator(),
        media_type="text/event-stream",
        headers={"Cache-Control": "no-cache", "Connection": "keep-alive"},
    )


@app.post("/api/llm/agent")
async def post_llm_agent(req: dict[str, Any]) -> JSONResponse:
    """Agent simulation with guardrails against direct prompt injection."""
    user_prompt = str(req.get("prompt", ""))
    system_role = "QA Assistant"

    injection_patterns = [
        r"ignore\s+(all\s+)?previous\s+instructions",
        r"reveal\s+(system\s+)?prompt",
        r"bypass\s+safety",
        r"admin\s+override",
    ]

    is_injection = any(re.search(pat, user_prompt, re.IGNORECASE) for pat in injection_patterns)

    if is_injection:
        return JSONResponse(
            status_code=400,
            content={
                "status": "blocked",
                "error": "PROMPT_INJECTION_DETECTED",
                "message": "Direct prompt injection attempt neutralized by agent input guardrails.",
                "safe_response": "I am a helpful QA Assistant and cannot disregard my security instructions.",
            },
        )

    return JSONResponse(
        status_code=200,
        content={
            "status": "success",
            "role": system_role,
            "response": f"Processed query safely: '{user_prompt[:50]}...'",
            "grounding_score": 0.98,
        },
    )


@app.get("/api/security/user-lookup")
async def get_user_lookup(user_id: str = "1") -> JSONResponse:
    """Security drill endpoint simulating parameterized vs blind SQL injection vulnerability."""
    # Simulated vulnerable check
    if "SLEEP(" in user_id.upper() or "PG_SLEEP" in user_id.upper():
        await asyncio.sleep(1.0)
        return JSONResponse(
            status_code=200,
            content={"id": 1, "username": "admin", "warning": "Blind timing SQLi detected in legacy query handler!"},
        )

    users_db = {"1": "alice_qa", "2": "bob_sdet", "3": "carol_lead"}
    username = users_db.get(user_id, "unknown_user")
    return JSONResponse(status_code=200, content={"id": user_id, "username": username, "status": "active"})


@app.post("/api/security/fetch-url")
async def post_fetch_url(req: dict[str, Any]) -> JSONResponse:
    """Security drill endpoint demonstrating SSRF prevention against cloud metadata."""
    target_url = str(req.get("url", ""))

    blocked_hosts = ["169.254.169.254", "localhost", "127.0.0.1", "metadata.google.internal"]
    is_ssrf = any(host in target_url.lower() for host in blocked_hosts)

    if is_ssrf:
        return JSONResponse(
            status_code=403,
            content={
                "status": "blocked",
                "error": "SSRF_ATTEMPT_PREVENTED",
                "message": f"Access to private/cloud metadata URL '{target_url}' is strictly forbidden.",
            },
        )

    return JSONResponse(
        status_code=200,
        content={"status": "fetched", "url": target_url, "content_type": "text/html", "bytes_received": 1420},
    )


@app.get("/api/security/cors-sensitive")
async def get_cors_sensitive(request: Request) -> JSONResponse:
    """Security drill endpoint demonstrating secure CORS vs wildcard credentials vulnerability."""
    origin = request.headers.get("origin", "")
    allowed_origins = ["http://localhost:8080", "http://127.0.0.1:8080"]

    if origin and origin not in allowed_origins:
        return JSONResponse(
            status_code=403,
            content={"error": "CORS_ORIGIN_DENIED", "message": f"Untrusted origin '{origin}' rejected."},
        )

    return JSONResponse(
        status_code=200,
        content={
            "user_id": "USR-9941",
            "email": "lead-sdet@cherenkov.dev",
            "roles": ["ADMIN", "TEST_ENGINEER"],
            "api_key_last_4": "8931",
        },
    )


@app.get("/api/pact/orders")
async def get_pact_orders() -> JSONResponse:
    """Consumer-Driven Contract test provider endpoint."""
    orders = [
        {"id": "ORD-101", "total": 149.00, "status": "COMPLETED", "currency": "USD"},
        {"id": "ORD-102", "total": 299.50, "status": "PENDING", "currency": "USD"},
    ]
    return JSONResponse(status_code=200, content={"orders": orders, "count": len(orders)})


@app.get("/api/progress")
async def get_progress() -> JSONResponse:
    """Retrieve learner progress, XP, level, streak, and achievements."""
    progress_file = Path(".cherenkov-progress.json")
    if progress_file.exists():
        try:
            data = json.loads(progress_file.read_text(encoding="utf-8"))
            return JSONResponse(status_code=200, content=data)
        except Exception:
            pass

    default_progress = {
        "total_xp": 0,
        "completed_drills": [],
        "unlocked_achievements": [],
        "streak_days": 0,
        "last_activity_date": None,
        "consecutive_perfect_flakiness": 0,
        "perfect_locator_count": 0,
    }
    return JSONResponse(status_code=200, content=default_progress)


@app.get("/api/curriculum")
async def get_curriculum() -> JSONResponse:
    """Retrieve all available tracks, drills, metadata, and theoretical summaries.

    Sourced from the lings.toml curriculum manifest, which is the single source
    of truth shared with the Rust engine.
    """
    try:
        tracks_data = build_curriculum()
    except ManifestError as exc:
        return JSONResponse(
            status_code=500,
            content={"error": "MANIFEST_UNAVAILABLE", "message": str(exc)},
        )

    total = sum(len(t["drills"]) for t in tracks_data)
    return JSONResponse(status_code=200, content={"tracks": tracks_data, "total_drills": total})


@app.get("/api/drill/theory")
async def get_drill_theory(path: str) -> JSONResponse:
    """Retrieve full theoretical markdown, hints, and production story for a specific drill."""
    drill_path = Path(path).resolve()
    base_path = Path("exercises").resolve()

    # Path traversal protection
    try:
        drill_path.relative_to(base_path)
    except ValueError:
        return JSONResponse(status_code=400, content={"error": "INVALID_PATH", "message": "Access restricted to exercises directory."})

    if not drill_path.exists() or not drill_path.is_dir():
        return JSONResponse(status_code=404, content={"error": "DRILL_NOT_FOUND", "message": f"Drill directory '{path}' not found."})

    theory_file = drill_path / "theory.md"
    hints_file = drill_path / "hints.md"

    theory_content = theory_file.read_text(encoding="utf-8") if theory_file.exists() else "# Theoretical Context\nNo theory module available for this drill."
    hints_content = hints_file.read_text(encoding="utf-8") if hints_file.exists() else "No progressive hints available."

    # Extract title from theory.md
    title = drill_path.name
    for line in theory_content.splitlines():
        if line.startswith("# "):
            title = line[2:].strip()
            break

    return JSONResponse(
        status_code=200,
        content={
            "drill_id": drill_path.name,
            "title": title,
            "theory_markdown": theory_content,
            "hints_markdown": hints_content,
            "has_theory": theory_file.exists(),
            "has_hints": hints_file.exists(),
        },
    )


# =============================================================================
# Sprint 4 REST Endpoints: Review, CI Pipeline, Allure Reports, Triage
# =============================================================================


def _guard_path(p: Path, base: Path = Path("exercises").resolve()) -> None:
    try:
        p.resolve().relative_to(base)
    except ValueError:
        raise HTTPException(status_code=400, detail=f"Access denied: '{p}' outside exercises directory")
    if ".." in str(p):
        raise HTTPException(status_code=400, detail="Path traversal not allowed")


@app.post("/api/review", response_model=ReviewReport)
async def post_review(req: ReviewRequest) -> ReviewReport:
    """Execute static AST code review, rule violation detection, and AI Senior QA critique."""
    code_content = req.code
    file_path = req.file_path or req.exercise_path or req.target or "exercise.ts"
    if not code_content:
        p = Path(file_path)
        if ".." in str(p):
            try:
                _guard_path(p)
            except HTTPException:
                raise HTTPException(status_code=400, detail=f"Invalid file path '{file_path}'")
        if p.exists() and p.is_file():
            try:
                code_content = p.read_text(encoding="utf-8")
            except Exception as e:
                raise HTTPException(status_code=400, detail=f"Failed to read file '{file_path}': {e}")
        else:
            raise HTTPException(status_code=400, detail=f"No code provided and file '{file_path}' does not exist.")
    return run_code_review(content=code_content, file_path=file_path, strict=req.strict, score_threshold=req.score_threshold)


@app.post("/api/review/fix", response_model=ReviewFixResponse)
async def post_review_fix(req: ReviewFixRequest) -> ReviewFixResponse:
    """Apply automated AST patch fixes to code or target exercise file."""
    code_content = req.code
    file_path = req.file_path or "exercise.ts"
    fix_id = req.fix_id or req.rule_id or "all"
    if req.file_path and ".." in str(req.file_path):
        try:
            _guard_path(Path(req.file_path))
        except HTTPException:
            raise HTTPException(status_code=400, detail=f"Invalid file path '{req.file_path}'")
    if not code_content and req.file_path:
        p = Path(req.file_path)
        if p.exists() and p.is_file():
            try:
                code_content = p.read_text(encoding="utf-8")
            except Exception as e:
                raise HTTPException(status_code=400, detail=f"Failed to read file '{req.file_path}': {e}")
        else:
            raise HTTPException(status_code=400, detail=f"File '{req.file_path}' does not exist.")
    if not code_content:
        raise HTTPException(status_code=400, detail="No code content or valid file_path provided.")
    fix_res = apply_review_fix(content=code_content, file_path=file_path, fix_id=fix_id)
    if req.file_path:
        p = Path(req.file_path)
        if p.exists() and p.is_file() and fix_res.patched_code:
            if ".." in str(p):
                try:
                    _guard_path(p)
                except HTTPException:
                    raise HTTPException(status_code=400, detail=f"Invalid file path '{req.file_path}'")
            try:
                p.write_text(fix_res.patched_code, encoding="utf-8")
            except Exception:
                pass
    return fix_res


@app.post("/api/pipeline/validate", response_model=PipelineValidation)
async def post_pipeline_validate(req: PipelineValidateRequest) -> PipelineValidation:
    """Validate workflow YAML against enterprise SDET policies."""
    yaml_content = req.workflow_yaml or req.yaml_content or req.content
    if yaml_content is None:
        raise HTTPException(status_code=400, detail="Missing workflow YAML content in request.")
    return validate_workflow_yaml(yaml_content, strict=req.strict)


@app.post("/api/pipeline/run", response_model=PipelineRunResult)
async def post_pipeline_run(req: PipelineRunRequest) -> PipelineRunResult:
    """Execute simulated matrix pipeline execution."""
    yaml_content = req.workflow_yaml or req.yaml_content or req.content
    if yaml_content is None:
        raise HTTPException(status_code=400, detail="Missing workflow YAML content in request.")
    try:
        return simulate_pipeline_run(yaml_content=yaml_content, parallel=req.parallel, fail_fast=req.fail_fast, strict_validation=req.strict_validation, verbose=req.verbose)
    except ValueError as e:
        if "exceeds cap" in str(e):
            raise HTTPException(status_code=400, detail=str(e))
        raise
    except Exception as e:
        raise HTTPException(status_code=400, detail=str(e))


@app.get("/api/reports/allure", response_model=AllureSummaryResponse)
async def get_allure_report_summary() -> AllureSummaryResponse:
    """Retrieve summary of chaotic test executions with telemetry."""
    dataset = generate_chaos_dataset()
    return summarize_dataset(dataset)


@app.get("/api/reports/allure/html", response_class=HTMLResponse)
async def get_allure_html_report() -> HTMLResponse:
    """Serve interactive Allure HTML report."""
    dataset = generate_chaos_dataset()
    html_content = render_html_report_string(dataset)
    return HTMLResponse(content=html_content, status_code=200)


@app.get("/api/triage/tests", response_model=list[ChaosTestResultItem])
async def get_triage_tests(
    category: str | None = None,
    failing_only: bool = True,
    track: str | None = None,
) -> list[ChaosTestResultItem]:
    """Retrieve list of chaotic tests for triage challenge."""
    dataset = generate_chaos_dataset()
    filtered = dataset
    if category and category.lower() != "all":
        norm_cat = category.lower().replace(" ", "_")
        filtered = [t for t in filtered if t.category == norm_cat]
    if failing_only:
        filtered = [t for t in filtered if t.status in ("failed", "broken", "flaky")]
    if track:
        filtered = [t for t in filtered if t.track_id == track]
    return filtered


@app.post("/api/triage/submit", response_model=TriageResultResponse)
async def post_triage_submit(req: TriageSubmissionRequest) -> TriageResultResponse:
    """Submit root-cause hypothesis, award XP, update streak, and unlock badges."""
    return evaluate_triage_submission(req)



