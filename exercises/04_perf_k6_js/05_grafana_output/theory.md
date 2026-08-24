# Theoretical Context: Real-Time Observability and Percentile Telemetry Export

## Production Incident: DoorDash Super Bowl Latency Blindspot (2021)

During the 2021 Super Bowl broadcast, on-demand food delivery giant DoorDash experienced severe localized ordering degradations in major metropolitan areas despite automated performance test suites reporting green status in CI. The test suites had validated average response times across simulated checkout journeys, observing an apparently healthy mean latency of 120ms. However, unmonitored tail latency had suffered severe regression: the 99th percentile (p99) latency had exploded to over 14,000ms due to database lock contention on merchant inventory tables. Because the load testing pipeline relied only on aggregated terminal stdout summaries rather than real-time percentile telemetry streaming to Grafana, engineering leadership had no visibility into the tail latency catastrophe until live customer orders began failing.

## The Underlying Mechanism

High-concurrency systems exhibit non-linear queueing behaviors described by Universal Scalability Law and Kingman's formula for waiting times:

1. **The Masking Effect of Averages**: In a dataset of 10,000 requests where 9,900 complete in 50ms and 100 take 15,000ms, the arithmetic mean is approximately 199.5ms—seemingly well within acceptable limits. Yet for a high-volume platform, that 1% represents thousands of abandoned shopping carts and failed transactions per minute.
2. **Custom Metrics and Tagging in k6**: k6 provides granular metric constructs:
   - `Trend`: Measures distribution values (p90, p95, p99, min, max, avg).
   - `Rate`: Tracks fractional success/failure ratios.
   - `Counter`: Records cumulative totals.
   - Custom Tags: Segment metrics by endpoint, HTTP status, or payload size.
3. **Telemetry Streaming to InfluxDB / Prometheus / Grafana**: Instead of waiting for post-test stdout logs, modern SDETs stream time-series metrics directly to time-series backends (such as InfluxDB, Prometheus, or Datadog) using k6 output extensions. This enables live visualization of latency distribution changes as Virtual Users scale.

```
[Anti-Pattern: Static Summary Log with Hidden p99 Tail Spike]
k6 Load Test ──> [Aggregated stdout Log] ──> Mean: 120ms (Pass) ──> Blind to Tail! ❌
                                             (Actual p99: 14,000ms)

[Resilient SDET Pattern: Continuous Percentile Telemetry to Grafana]
k6 Load Test ──┬── Custom Trend('order_latency') ──► InfluxDB / Prometheus
               ├── Custom Rate('failed_orders')   ──► InfluxDB / Prometheus
               └── Tag: { endpoint: 'checkout' }   ──► InfluxDB / Prometheus
                                                               │
                                                               ▼
                                                  [Grafana Live Dashboard]
                                                  p99 Alert Triggered! ✅
```

Exporting high-resolution time-series metrics ensures that performance regressions are detected and analyzed across all percentile dimensions during automated load validation.

You will now simulate this in the Crucible: configure k6 custom Trend and Rate metrics with strict percentile thresholds and structured telemetry output formatting.
