# Hints: Drill 03 - Chaos SLA Assertion

## Hint 1 (Architectural Nudge)
When testing through a chaos proxy, a test that passes silently despite a 50% error rate is worse than no test at all. You need an observable metric that can fail the build.

## Hint 2 (API Pattern)
Use a k6 Rate metric: const errorRate = new Rate('chaos_errors'). Set a threshold: 'chaos_errors': ['rate<0.05']. Track failures: errorRate.add(res.status >= 400 ? 1 : 0).

## Hint 3 (Code Diff)
Add: const errorRate = new Rate('chaos_errors'). Add thresholds: 'chaos_errors': ['rate<0.05'], 'http_req_duration': ['p(95)<3000']. Add timeout: '10s' to the http.get options. Track: errorRate.add(res.status < 200 || res.status >= 300 ? 1 : 0).
