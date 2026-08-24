# Hints: Drill 02 - Spike Profile and p99 Latency

## Hint 1 (Architectural Nudge)
Averages lie. A test reporting avg=200ms can hide the fact that 1% of users experience 5-second timeouts. Production SLAs are measured at tail latency (p99). Without a k6 Trend metric and a threshold, your test never fails even if p99 explodes.

## Hint 2 (API Pattern)
Define a custom Trend metric with percentile reporting: const myDuration = new Trend('custom_duration', true). Attach a threshold: 'custom_duration': ['p(99)<500']. Call myDuration.add(res.timings.duration) in your default function.

## Hint 3 (Code Diff)
Add: const searchDuration = new Trend('search_response_time', true). Add thresholds: 'search_response_time': ['p(99)<5000'], 'search_errors': ['rate<0.05']. Call searchDuration.add(res.timings.duration) after each request.
