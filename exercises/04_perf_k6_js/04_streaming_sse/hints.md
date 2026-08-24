# Hints: Drill 04 - Streaming SSE Testing

## Hint 1 (Architectural Nudge)
Server-Sent Events (SSE) hold open persistent HTTP connections. Load tests must explicitly assert on `Content-Type: text/event-stream` and parse stream chunks (`event: message`, `data: {...}`) rather than expecting standard JSON bodies.

## Hint 2 (API Pattern)
Set request headers `Accept: text/event-stream` and use k6 `check` to verify stream framing and custom `Rate`/`Trend` metrics to monitor streaming health.

## Hint 3 (Code Diff)
```diff
  const res = http.get('http://127.0.0.1:8081/events/stream');
- check(res, { 'status is 200': (r) => r.status === 200 });
+ check(res, {
+   'status is 200': (r) => r.status === 200,
+   'content-type is text/event-stream': (r) => r.headers['Content-Type'].includes('text/event-stream'),
+   'body contains event data': (r) => r.body.includes('data:'),
+ });
```
