# Hints: Drill 05 - Grafana Metrics & Strict Thresholds

## Hint 1 (Architectural Nudge)
Averages hide latency spikes. A system with a 100ms average response time can have a 10,000ms p99 latency affecting thousands of users. Always define percentile thresholds (p95, p99) and custom tagged metrics for dashboard export.

## Hint 2 (API Pattern)
Use k6 `Trend` and `Rate` metric constructors from `k6/metrics` and define thresholds in `export const options = { thresholds: { ... } }`.

## Hint 3 (Code Diff)
```diff
+ export const options = {
+   thresholds: {
+     'http_req_duration{endpoint:checkout}': ['p(99)<250', 'p(95)<150'],
+     'failed_orders': ['rate<0.01'],
+   },
+ };
```
