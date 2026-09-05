"""
PRODUCTION STORY:
Fintech Async Ledger Settlement Failure (2021)
By acting as a distributed trace participant, the test injects W3C traceparent
context and asserts that the server's span correlates back to the client's
own span ID, proving the request was actually observable end to end -- not
just that it returned 200.
"""

import secrets
import requests

def test_distributed_trace_and_span_correlation():
    base_url = "http://localhost:8081"
    payload = {
        "from_account": "ACC-001",
        "to_account": "ACC-002",
        "amount": 100.0,
    }

    # 32 hex chars (16 bytes) trace_id, 16 hex chars (8 bytes) client_span_id,
    # per the W3C Trace Context spec: version-trace_id-parent_id-flags.
    trace_id = secrets.token_hex(16)
    client_span_id = secrets.token_hex(8)

    headers = {
        "traceparent": f"00-{trace_id}-{client_span_id}-01",
        "Content-Type": "application/json",
    }

    response = requests.post(f"{base_url}/transfer", json=payload, headers=headers)
    assert response.status_code == 200, f"Expected 200, got {response.status_code}"

    telemetry_resp = requests.get(f"{base_url}/api/telemetry/spans?trace_id={trace_id}")
    assert telemetry_resp.status_code == 200, (
        f"Expected 200 from telemetry query, got {telemetry_resp.status_code}"
    )
    spans = telemetry_resp.json()["spans"]
    root_span = next((s for s in spans if s.get("parent_span_id") == client_span_id), None)
    assert root_span is not None, "Root span parent_span_id must match client_span_id"
    assert root_span.get("trace_id") == trace_id, "Span trace_id must match propagated trace_id"
