import json
import time
from fastapi.testclient import TestClient
from crucible.backend.app import app
from crucible.backend.chaos import parse_chaos_header, parse_duration_ms

client = TestClient(app)

def test_parse_duration_ms():
    assert parse_duration_ms("500ms") == 500.0
    assert parse_duration_ms("1.5s") == 1500.0
    assert parse_duration_ms("2s") == 2000.0
    assert parse_duration_ms("300") == 300.0

def test_parse_chaos_header_compound():
    header = "delay=500ms;jitter=75ms;stale_dom=true;token_expire=immediate;kafka_lag=1000ms;drop_partial=true;drop_after=3"
    parsed = parse_chaos_header(header)
    assert parsed["delay"] == 500.0
    assert parsed["jitter"] == 75.0
    assert parsed["stale_dom"] is True
    assert parsed["token_expire"] == "immediate"
    assert parsed["kafka_lag"] == 1000.0
    assert parsed["drop_partial"] is True
    assert parsed["drop_after"] == 3

def test_parse_chaos_header_malformed():
    parsed = parse_chaos_header("invalid;;;foo=bar;delay=notanumber")
    assert parsed.get("delay") == 0.0
    assert parsed.get("foo") == "bar"

def test_chaos_delay_header():
    # Verify >= 500ms latency
    start = time.perf_counter()
    resp = client.get("/health", headers={"X-Chaos": "delay=500ms"})
    elapsed = time.perf_counter() - start

    assert resp.status_code == 200
    assert resp.json()["status"] == "ok"
    assert elapsed >= 0.48, f"Expected elapsed >= 0.48s, got {elapsed:.3f}s"

def test_chaos_stale_dom_header():
    # Verify stale_dom=true injects header
    resp = client.get("/checkout", headers={"X-Chaos": "stale_dom=true"})
    assert resp.status_code == 200
    assert resp.headers.get("x-chaos-stale-dom") == "true" or resp.headers.get("X-Chaos-Stale-DOM") == "true"

    # Verify normal request does NOT have stale_dom header
    resp_normal = client.get("/checkout")
    assert "x-chaos-stale-dom" not in resp_normal.headers
    assert "X-Chaos-Stale-DOM" not in resp_normal.headers

def test_chaos_token_expire_immediate():
    # 1. Normal login returns valid token that works on /auth/me
    login_resp = client.post("/auth/login", json={"username": "alice", "password": "password123"})
    assert login_resp.status_code == 200
    normal_token = login_resp.json()["access_token"]

    me_resp = client.get("/auth/me", headers={"Authorization": f"Bearer {normal_token}"})
    assert me_resp.status_code == 200
    assert me_resp.json()["username"] == "alice"

    # 2. Login with token_expire=immediate creates immediately expired token
    chaos_login_resp = client.post(
        "/auth/login",
        json={"username": "bob", "password": "password123"},
        headers={"X-Chaos": "token_expire=immediate"}
    )
    assert chaos_login_resp.status_code == 200
    chaos_token = chaos_login_resp.json()["access_token"]
    assert chaos_login_resp.json()["expires_in"] == 0

    # 3. Accessing /auth/me with expired token results in 401 Unauthorized
    chaos_me_resp = client.get("/auth/me", headers={"Authorization": f"Bearer {chaos_token}"})
    assert chaos_me_resp.status_code == 401
    assert "expired" in chaos_me_resp.json()["detail"].lower()

def test_chaos_kafka_lag():
    # Reset state
    reset_resp = client.post("/reset")
    assert reset_resp.status_code == 200

    # Verify initial balance
    bal1 = client.get("/balance?account_id=ACC-001").json()
    assert bal1["balance"] == 1000.00
    assert bal1["pending_count"] == 0

    # Send transfer of $250 with 1000ms Kafka lag
    tx_resp = client.post(
        "/transfer",
        json={"from_account": "ACC-001", "to_account": "ACC-002", "amount": 250.00},
        headers={"X-Chaos": "kafka_lag=1000ms"}
    )
    assert tx_resp.status_code == 200
    assert tx_resp.json()["status"] == "QUEUED_LEDGER"
    assert tx_resp.json()["lag_ms"] == 1000.0

    # Immediately check balance: should still be 1000.00 and pending_count == 1
    bal_immediate = client.get("/balance?account_id=ACC-001").json()
    assert bal_immediate["balance"] == 1000.00, f"Expected balance 1000.00 before lag elapses, got {bal_immediate['balance']}"
    assert bal_immediate["pending_count"] == 1

    # Wait for lag to elapse (> 1000ms)
    time.sleep(1.1)

    # Balance should now be settled to 750.00 and pending_count == 0
    bal_settled = client.get("/balance?account_id=ACC-001").json()
    assert bal_settled["balance"] == 750.00, f"Expected settled balance 750.00, got {bal_settled['balance']}"
    assert bal_settled["pending_count"] == 0

    bal_dest = client.get("/balance?account_id=ACC-002").json()
    assert bal_dest["balance"] == 750.00

def test_chaos_search_inverted_latency():
    # Short query (len <= 2) should take >= 800ms
    start_short = time.perf_counter()
    resp_short = client.get("/search?q=p")
    elapsed_short = time.perf_counter() - start_short
    assert resp_short.status_code == 200
    assert elapsed_short >= 0.78
    assert "Python" in resp_short.json()["results"]

    # Long query (len > 2) should take < 200ms
    start_long = time.perf_counter()
    resp_long = client.get("/search?q=playwright")
    elapsed_long = time.perf_counter() - start_long
    assert resp_long.status_code == 200
    assert elapsed_long < 0.3
    assert "Playwright TypeScript" in resp_long.json()["results"]

def test_chaos_jitter_does_not_crash_negative_delay():
    # delay=10ms, jitter=50ms could produce negative without max(0.0, ...)
    start = time.perf_counter()
    resp = client.get("/health", headers={"X-Chaos": "delay=10ms;jitter=50ms"})
    elapsed = time.perf_counter() - start
    assert resp.status_code == 200
    assert elapsed >= 0.0

def test_multiple_concurrent_transfers_with_different_lags():
    client.post("/reset")
    # Transfer 1: 100 with 500ms lag
    client.post(
        "/transfer",
        json={"from_account": "ACC-001", "to_account": "ACC-002", "amount": 100.00},
        headers={"X-Chaos": "kafka_lag=500ms"}
    )
    # Transfer 2: 200 with 1500ms lag
    client.post(
        "/transfer",
        json={"from_account": "ACC-001", "to_account": "ACC-002", "amount": 200.00},
        headers={"X-Chaos": "kafka_lag=1500ms"}
    )

    # Immediately: 2 pending, balance = 1000
    b0 = client.get("/balance?account_id=ACC-001").json()
    assert b0["balance"] == 1000.00
    assert b0["pending_count"] == 2

    # After 700ms: first transfer settled (balance = 900), second still pending (pending_count = 1)
    time.sleep(0.7)
    b1 = client.get("/balance?account_id=ACC-001").json()
    assert b1["balance"] == 900.00
    assert b1["pending_count"] == 1

    # After another 1000ms: second transfer settled (balance = 700), pending_count = 0
    time.sleep(1.0)
    b2 = client.get("/balance?account_id=ACC-001").json()
    assert b2["balance"] == 700.00
    assert b2["pending_count"] == 0


def test_chaos_idempotency_conflict():
    # 1. Normal checkout returns 200 and success
    resp_normal = client.post(
        "/checkout",
        json={"item_id": "item-1", "customer_name": "QA Student", "payment_method": "credit_card"},
        headers={"Idempotency-Key": "key-123"}
    )
    assert resp_normal.status_code == 200
    assert resp_normal.json()["status"] == "success"
    assert resp_normal.json()["order_id"].startswith("ORD-")

    # 2. Checkout with idempotency_conflict returns 409 Conflict with expected payload
    resp_conflict = client.post(
        "/checkout",
        json={"item_id": "item-1", "customer_name": "QA Student", "payment_method": "credit_card"},
        headers={"Idempotency-Key": "key-123", "X-Chaos": "idempotency_conflict=true"}
    )
    assert resp_conflict.status_code == 409
    data = resp_conflict.json()
    assert data["status"] == "conflict"
    assert data["error"] == "IDEMPOTENCY_CONFLICT"
    assert data["message"] == "Order conflict: duplicate transaction detected with idempotency key"
    assert data["order_id"] == "ORD-4821"


def test_rag_endpoint_valid_query():
    """Verify /api/rag returns grounded response with matched source facts."""
    resp = client.get("/api/rag?query=Cherenkov")
    assert resp.status_code == 200
    data = resp.json()
    assert data["grounded"] is True
    assert data["document_title"] == "Cherenkov Radiation Primer"
    assert len(data["source_facts"]) > 0
    assert any("cherenkov" in f.lower() for f in data["source_facts"])
    assert "Cherenkov Radiation Primer" in data["answer"]


def test_rag_endpoint_empty_query():
    """Verify /api/rag handles empty query by returning ungrounded response."""
    resp = client.get("/api/rag?query=")
    assert resp.status_code == 200
    data = resp.json()
    assert data["grounded"] is False
    assert data["source_facts"] == []
    assert "Please provide a search query" in data["answer"]


def test_rag_endpoint_unmatched_query_fallback():
    """Verify /api/rag falls back gracefully to default facts for unmatched queries."""
    resp = client.get("/api/rag?query=completely_unrelated_xyz_topic")
    assert resp.status_code == 200
    data = resp.json()
    assert data["grounded"] is True
    assert len(data["source_facts"]) == 2


def test_llm_endpoint_structured_invariance():
    """Verify /api/llm returns stable intent and entity classifications."""
    resp = client.get("/api/llm?prompt=What+is+my+transfer+status")
    assert resp.status_code == 200
    data = resp.json()
    assert data["intent"] == "transfer_status_inquiry"
    assert data["entities"]["action"] == "transfer"
    assert data["entities"]["status"] == "pending"
    assert data["entities"]["domain"] == "ledger"
    assert data["confidence"] >= 0.8
    assert data["model"] == "mock-llm-v1"


def test_llm_endpoint_raw_text_variance():
    """Verify /api/llm cycles through phrasing variants across repeated calls."""
    variants_seen = set()
    for _ in range(8):
        resp = client.get("/api/llm?prompt=Check+status")
        assert resp.status_code == 200
        variants_seen.add(resp.json()["raw_text"])
    assert len(variants_seen) > 1, "Expected raw_text to vary across calls"


def test_genai_endpoints_chaos_delay():
    """Verify GenAI mock endpoints respect X-Chaos latency injection."""
    start = time.perf_counter()
    resp = client.get("/api/rag?query=Cherenkov", headers={"X-Chaos": "delay=200ms"})
    elapsed = time.perf_counter() - start
    assert resp.status_code == 200
    assert elapsed >= 0.18


# ---------------------------------------------------------------------------
# R4 Crucible Backend Expansion Tests
# ---------------------------------------------------------------------------


def test_upload_endpoint_success():
    """Verify multipart file upload returns proper metadata."""
    file_bytes = b"Sample upload content for Micro-Crucible."
    resp = client.post(
        "/upload",
        files={"file": ("test_file.txt", file_bytes, "text/plain")},
    )
    assert resp.status_code == 200
    data = resp.json()
    assert data["filename"] == "test_file.txt"
    assert data["content_type"] == "text/plain"
    assert data["size_bytes"] == len(file_bytes)
    assert data["status"] == "uploaded"
    assert data["message"] == "File uploaded successfully"


def test_upload_endpoint_chaos_drop_partial():
    """Verify drop_partial chaos header aborts upload with 400 error."""
    file_bytes = b"Partial upload content simulation."
    resp = client.post(
        "/upload",
        files={"file": ("partial_file.dat", file_bytes, "application/octet-stream")},
        headers={"X-Chaos": "drop_partial=true"},
    )
    assert resp.status_code == 400
    data = resp.json()
    assert data["status"] == "error"
    assert data["error"] == "PARTIAL_UPLOAD_DROPPED"
    assert "dropped" in data["message"].lower() or "aborted" in data["message"].lower()
    assert data["bytes_received"] == 0


def test_products_pagination_navigation():
    """Verify paginated product list navigation across pages."""
    resp1 = client.get("/products?page=1&per_page=5")
    assert resp1.status_code == 200
    data1 = resp1.json()
    assert data1["total"] == 25
    assert data1["page"] == 1
    assert data1["per_page"] == 5
    assert data1["total_pages"] == 5
    assert len(data1["products"]) == 5
    assert data1["products"][0]["id"] == "prod-001"

    resp2 = client.get("/products?page=2&per_page=5")
    assert resp2.status_code == 200
    data2 = resp2.json()
    assert data2["page"] == 2
    assert len(data2["products"]) == 5
    assert data2["products"][0]["id"] == "prod-006"
    assert data2["products"][0]["id"] != data1["products"][0]["id"]


def test_products_pagination_defaults_and_out_of_bounds():
    """Verify default query parameters and out-of-bounds page handling."""
    resp_default = client.get("/products")
    assert resp_default.status_code == 200
    data_default = resp_default.json()
    assert data_default["page"] == 1
    assert data_default["per_page"] == 10
    assert len(data_default["products"]) == 10
    assert data_default["total"] == 25
    assert data_default["total_pages"] == 3

    resp_oob = client.get("/products?page=999&per_page=10")
    assert resp_oob.status_code == 200
    data_oob = resp_oob.json()
    assert data_oob["page"] == 999
    assert data_oob["products"] == []
    assert data_oob["total"] == 25


def test_events_stream_sse_protocol():
    """Verify Server-Sent Events stream emits formatted event payloads."""
    with client.stream("GET", "/events/stream", headers={"X-Chaos": "drop_after=1"}) as resp:
        assert resp.status_code == 200
        assert "text/event-stream" in resp.headers.get("content-type", "")
        lines = [line for line in resp.iter_lines() if line]
        assert any(line_item.startswith("id: 1") for line_item in lines)
        assert any(line_item.startswith("event: message") for line_item in lines)
        data_lines = [line_item for line_item in lines if line_item.startswith("data: ")]
        assert len(data_lines) >= 1
        data_json = json.loads(data_lines[0][len("data: "):])
        assert data_json["id"] == 1
        assert "telemetry" in data_json["data"].lower()


def test_events_stream_chaos_drop_after():
    """Verify drop_after chaos header disconnects SSE stream after N events."""
    with client.stream("GET", "/events/stream", headers={"X-Chaos": "drop_after=2"}) as resp:
        assert resp.status_code == 200
        lines = [line for line in resp.iter_lines() if line]
        id_lines = [line_item for line_item in lines if line_item.startswith("id: ")]
        assert len(id_lines) == 2
        assert id_lines[0] == "id: 1"
        assert id_lines[1] == "id: 2"


def test_graphql_query_user_direct():
    """Verify direct GraphQL query resolves user entity fields."""
    query = "{ user { id name email role status } }"
    resp = client.post("/graphql", json={"query": query})
    assert resp.status_code == 200
    res_data = resp.json()
    assert "data" in res_data
    assert "user" in res_data["data"]
    user = res_data["data"]["user"]
    assert user["id"] == "usr-4819"
    assert user["name"] == "sdet_student"
    assert user["email"] == "student@cherenkov.qa"
    assert user["role"] == "sdet_engineer"
    assert user["status"] == "active"


def test_graphql_aliased_query_me():
    """Verify aliased GraphQL query resolves correctly with custom target key."""
    query = "{ me: user { id name role } }"
    resp = client.post("/graphql", json={"query": query})
    assert resp.status_code == 200
    res_data = resp.json()
    assert "data" in res_data
    assert "me" in res_data["data"]
    assert "user" not in res_data["data"]
    me = res_data["data"]["me"]
    assert me["id"] == "usr-4819"
    assert me["name"] == "sdet_student"
    assert me["role"] == "sdet_engineer"


def test_graphql_query_with_named_operation():
    """Verify query with 'query OperationName' prefix resolves correctly."""
    query = """
    query GetUserProfile {
        profile: user {
            id
            name
        }
    }
    """
    resp = client.post("/graphql", json={"query": query})
    assert resp.status_code == 200
    res_data = resp.json()
    assert "profile" in res_data["data"]
    assert res_data["data"]["profile"]["name"] == "sdet_student"


def test_graphql_errors_and_invalid_queries():
    """Verify unknown fields and invalid GraphQL queries return 400 with errors."""
    # Unknown field on Query
    resp_unknown = client.post("/graphql", json={"query": "{ unknown_field { id } }"})
    assert resp_unknown.status_code == 400
    data_unknown = resp_unknown.json()
    assert "errors" in data_unknown
    assert "unknown_field" in data_unknown["errors"][0]["message"]

    # Malformed syntax
    resp_syntax = client.post("/graphql", json={"query": "this is not valid graphql"})
    assert resp_syntax.status_code == 400
    data_syntax = resp_syntax.json()
    assert "errors" in data_syntax



