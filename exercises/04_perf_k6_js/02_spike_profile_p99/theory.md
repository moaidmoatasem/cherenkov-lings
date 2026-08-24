# Theoretical Context: Spike Profiles & Tail Latency (p99 / p99.9)

## Production Incident: Ticketmaster Taylor Swift "Eras Tour" Onsale Surge (2022)

In November 2022, Ticketmaster opened the verified fan presale for Taylor Swift's Eras Tour, expecting 1.5 million verified customers. Instead, the ticketing platform was inundated with over 3.5 billion system requests, including automated bot swarms and unregistered users hitting the queue simultaneously. The sudden step-function traffic spike overwhelmed queue-routing microservices and inventory reservation locks. While average response times initially appeared manageable in high-level telemetry, p99 tail latencies skyrocketed past 60,000ms. Thousands of fan checkout sessions timed out mid-transaction, inventory locks became deadlocked, and Ticketmaster was forced to cancel the subsequent public ticket sales entirely.

## The Underlying Mechanism

Standard load tests often ramp traffic gradually (e.g., over 10 minutes), allowing auto-scaling groups, JIT compilers, and connection pools to warm up. In contrast, real production traffic frequently arrives as an instantaneous step-function (Spike Profile):

1. **The Flaw of Average (Mean) Metrics**: Average response time hides severe degradation. If 95% of users receive responses in 50ms, but 5% of users experience 25,000ms timeouts, the average might look like a benign 1,297ms, completely masking catastrophic tail failure.
2. **Percentiles (p90, p95, p99, p99.9)**:
   - $p95$: 95% of requests completed faster than this threshold.
   - $p99$: The slowest 1% of transactions (typically complex database transactions, lock contentions, or garbage collection pauses).
3. **k6 Spike Execution**: k6 allows defining multi-stage execution profiles that jump from 0 to 1,000 VUs in seconds to test cold-start resilience, autoscaler lag, and p99 SLA enforcement.

```
[Spike Traffic Profile & Tail Latency Distribution]
VUs
 │       ┌──────────────────┐ (Spike: 1000 VUs in 5s)
 │       │                  │
 │       │                  │
 └───────┘                  └───────> Time

Latency Distribution:
  ├── 50% (Median / p50):   45ms   (Fast static cache)
  ├── 90% (p90):            120ms  (Normal query execution)
  ├── 95% (p95):            280ms  (Minor queue contention)
  └── 99% (p99 Tail):     8,500ms  (Lock contention & timeout!) ❌
```

Enforcing strict percentile thresholds (`http_req_duration: ['p(99)<500']`) ensures systems withstand sudden traffic bursts without leaving tail users stranded.

You will now simulate this in the Crucible: configure a rapid spike load test and enforce strict p99 percentile thresholds using k6.
