"""
PRODUCTION STORY:
Fintech Async Ledger Settlement Failure (2021)
An HTTP 200 response alone does not guarantee backend side-effects occurred
correctly. Downstream Kafka consumers and async database writes operate
out-of-band. A test that only checks response status code misses silent async
failures, consumer lag, and ledger balance corruption.

GOAL:
Inject a W3C traceparent header ('00-{trace_id}-{parent_id}-01') into the HTTP
request so the server's span is correlated back to this client, then query
/api/telemetry/spans to prove the correlation actually happened -- not just
that the request returned 200.
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

    trace_id = secrets.token_hex(16)
    client_span_id = secrets.token_hex(8)

    # TODO: This trace_id and client_span_id are generated but never sent
    # anywhere -- the request below carries no 'traceparent' header, so the
    # server has nothing of the client's to correlate against. It records a
    # span under a trace_id of its own choosing, not this one.
    response = requests.post(f"{base_url}/transfer", json=payload)
    assert response.status_code == 200

    telemetry_resp = requests.get(f"{base_url}/api/telemetry/spans?trace_id={trace_id}")
    assert telemetry_resp.status_code == 200
    spans = telemetry_resp.json()["spans"]
    root_span = next((s for s in spans if s.get("parent_span_id") == client_span_id), None)
    assert root_span is not None, (
        f"No span under trace_id={trace_id} has parent_span_id={client_span_id} -- "
        "the server never saw this trace_id because it was never sent."
    )
