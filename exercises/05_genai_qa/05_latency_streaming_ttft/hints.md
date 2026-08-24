## Hint 1 (Architectural Nudge)
In streaming LLM architectures, Time-To-First-Token (TTFT) is the primary user-perceived performance metric.

## Hint 2 (API Pattern)
Verify `Content-Type: text/event-stream` and assert on response headers and initial chunk latency.

## Hint 3 (Code Diff)
```diff
- expect(body.length).toBeGreaterThan(0);
+ expect(res.headers()['content-type']).toContain('text/event-stream');
+ expect(ttft).toBeLessThan(1500);
```
