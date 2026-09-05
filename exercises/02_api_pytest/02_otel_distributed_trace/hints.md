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

To assert span correlation, not just a status code:
1. Query the Crucible telemetry endpoint: `GET /api/telemetry/spans?trace_id={trace_id}`. It returns `{"spans": [...]}`.
2. Verify that a span exists in that list whose `parent_span_id` equals the `client_span_id` you generated and sent.
3. Assert this directly -- no `if response.status_code == 200:` guard around it. A guard like that turns "the check failed" into "the check silently never ran," which is the same vacuous-test failure mode as the missing assertion in Hint 1, just one layer deeper.

## Hint 3 (Code Diff)
Replace the disconnected trace_id (generated but never sent) and the missing correlation check with real propagation and a direct assertion:

```diff
+ import secrets
  import requests

  def test_distributed_trace_and_span_correlation():
      base_url = "http://localhost:8081"
      payload = {"from_account": "ACC-001", "to_account": "ACC-002", "amount": 100.0}

      trace_id = secrets.token_hex(16)
      client_span_id = secrets.token_hex(8)

-     response = requests.post(f"{base_url}/transfer", json=payload)
+     headers = {
+         "traceparent": f"00-{trace_id}-{client_span_id}-01",
+         "Content-Type": "application/json",
+     }
+     response = requests.post(f"{base_url}/transfer", json=payload, headers=headers)
      assert response.status_code == 200

      telemetry_resp = requests.get(f"{base_url}/api/telemetry/spans?trace_id={trace_id}")
      assert telemetry_resp.status_code == 200
      spans = telemetry_resp.json()["spans"]
      root_span = next((s for s in spans if s.get("parent_span_id") == client_span_id), None)
      assert root_span is not None, "No span correlates to the client_span_id we sent"
```
