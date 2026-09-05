# Hints: Drill 02 - OpenTelemetry Distributed Trace & Span ID Correlation

## Hint 1 (Architectural Nudge)
In modern distributed microservice architectures, an HTTP 200 response alone does not guarantee backend side-effects occurred correctly. Downstream queue lag, async database writes, and background message broker workers operate out-of-band. A test that only checks `response.status_code == 200` suffers from the vacuous assertion anti-pattern: it misses async failures, silent downstream drops, and ledger corruption.

To verify end-to-end consistency without fragile, non-deterministic sleeps (`time.sleep`), your test must act as a distributed trace participant. Every request initiated by your test must propagate a W3C trace context header (`traceparent`) across service boundaries so all downstream spans share the exact same `trace_id`. Naive tests that issue disconnected requests break trace context propagation, rendering downstream async execution invisible to test verification.

## Hint 2 (API Pattern)
Follow the W3C Trace Context specification (`traceparent: version-trace_id-parent_id-trace_flags`):
- `version`: `00` (current W3C specification version)
- `trace_id`: 32-character hexadecimal string (16 bytes, globally unique trace identifier)
- `parent_id` (Span ID): 16-character hexadecimal string (8 bytes, caller/client span identifier)
- `trace_flags`: `01` (recorded/sampled flag enabled)

Format the header in Python:
```python
import secrets

trace_id = secrets.token_hex(16)
client_span_id = secrets.token_hex(8)
headers = {
    "traceparent": f"00-{trace_id}-{client_span_id}-01",
    "Content-Type": "application/json",
}
response = client.post("/transfer", json=payload, headers=headers)
```

To assert parent-child span tree correlation and distributed trace execution:
1. Query the OpenTelemetry collector or Crucible telemetry endpoint (`/api/telemetry/spans?trace_id={trace_id}`).
2. Verify that a root server span exists whose `parent_span_id` equals your injected `client_span_id`.
3. Assert that downstream asynchronous spans (e.g. `kafka.ledger.settle`, `db.persist`) share the exact same `trace_id` and have an `OK` status code.

## Hint 3 (Code Diff)
Replace the naive sleep and status-only check with W3C traceparent propagation and distributed span tree assertions:

```diff
- # Old brittle check: sleep and assume background Kafka processed the event
- response = client.post("/transfer", json=payload)
- time.sleep(2)
- assert response.status_code == 200

+ # Resilient OTel Assertion: Propagate W3C traceparent and assert span tree correlation
+ import secrets
+ trace_id = secrets.token_hex(16)
+ client_span_id = secrets.token_hex(8)
+ headers = {
+     "traceparent": f"00-{trace_id}-{client_span_id}-01",
+     "Content-Type": "application/json",
+ }
+ response = client.post("/transfer", json=payload, headers=headers)
+ assert response.status_code == 200
+
+ # Query Crucible telemetry / OTel collector for distributed span correlation
+ spans = wait_for_spans(trace_id=trace_id, timeout_sec=5.0)
+ root_span = next(s for s in spans if s.get("parent_span_id") == client_span_id)
+ assert root_span["parent_span_id"] == client_span_id, "W3C traceparent client Span ID not correlated"
+ assert any(s["name"] == "kafka.ledger.settle" for s in spans), "Missing downstream Kafka event span"
+ assert all(s.get("status", {}).get("code") != "ERROR" for s in spans), "Distributed span reported error"
```
