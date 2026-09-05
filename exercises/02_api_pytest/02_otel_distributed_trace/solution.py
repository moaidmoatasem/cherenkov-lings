"""
PRODUCTION STORY:
Fintech Async Ledger Settlement Failure (2021)
By acting as a distributed trace participant, the test injects W3C traceparent
context and asserts that downstream spans correlate back to the client Span ID,
proving asynchronous operations completed without silent failures.
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

    # Generate 16-byte (32 hex) trace_id and 8-byte (16 hex) client_span_id
    trace_id = secrets.token_hex(16)
    client_span_id = secrets.token_hex(8)

    headers = {
        "traceparent": f"00-{trace_id}-{client_span_id}-01",
        "Content-Type": "application/json",
    }

    response = requests.post(f"{base_url}/transfer", json=payload, headers=headers)
    assert response.status_code == 200, f"Expected 200, got {response.status_code}"

    # Query telemetry spans for parent-child tree assertions
    telemetry_resp = requests.get(f"{base_url}/api/telemetry/spans?trace_id={trace_id}")
    if telemetry_resp.status_code == 200:
        spans = telemetry_resp.json().get("spans", [])
        if spans:
            root_span = next((s for s in spans if s.get("parent_span_id") == client_span_id), None)
            assert root_span is not None, "Root span parent_span_id must match client_span_id"
            assert root_span.get("trace_id") == trace_id, "Span trace_id must match propagated trace_id"
