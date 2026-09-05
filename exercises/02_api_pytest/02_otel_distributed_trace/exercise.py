"""
PRODUCTION STORY:
Fintech Async Ledger Settlement Failure (2021)
An HTTP 200 response alone does not guarantee backend side-effects occurred
correctly. Downstream Kafka consumers and async database writes operate
out-of-band. A test that only checks response status code misses silent async
failures, consumer lag, and ledger balance corruption.

GOAL:
Inject W3C traceparent headers ('00-{trace_id}-{parent_id}-01') into the HTTP
request and assert distributed span status and parent-child correlation.
"""

import requests
import time

def test_distributed_trace_and_span_correlation():
    base_url = "http://localhost:8081"
    payload = {
        "from_account": "ACC-001",
        "to_account": "ACC-002",
        "amount": 100.0,
    }

    # FLAWED: Naive test without W3C traceparent context propagation
    # and relying on brittle sleep without verifying telemetry spans.
    response = requests.post(f"{base_url}/transfer", json=payload)
    time.sleep(1)
    # TODO: Generate trace_id (32 hex chars) and client_span_id (16 hex chars).
    # TODO: Inject 'traceparent': f'00-{trace_id}-{client_span_id}-01'.
    # TODO: Query /api/telemetry/spans?trace_id={trace_id} and assert parent-child correlation.
    assert response.status_code == 200
