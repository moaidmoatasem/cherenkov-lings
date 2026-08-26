# Theoretical Context: Open vs Closed Workload Models & Coordinated Omission

## Production Incident: The Robinhood Outage (March 2020)

On 2 March 2020, as markets opened on what became one of the highest-volume trading days of the year, Robinhood's platform went fully dark for an entire trading session, then failed again the following day. Customers could not close positions during extreme volatility, and the resulting complaints drew a $70 million FINRA penalty. The load characteristics that day were not a gradual ramp; they were a step function of users all pressing the same button in the same minute. Systems validated with gentle concurrency ramps routinely survive their own load tests and then fall over on precisely this shape of traffic.

## The Underlying Mechanism

Load generators come in two fundamentally different flavours, and choosing the wrong one produces tests that cannot reproduce production failures:

1. **Closed model (VU / concurrency based)**: A fixed population of virtual users each loop *request → wait for response → request*. Throughput is an *output*. When the server slows, VUs block waiting, offered load drops, and the server is handed an accidental recovery period it would never get in production. This is **coordinated omission**: the load generator conspires with the system under test to hide the queue.

2. **Open model (arrival rate based)**: Requests are issued on a schedule regardless of whether previous ones have returned. Throughput is an *input*. If the service degrades, the queue grows, latency compounds, and the generator reports `dropped_iterations` when it can no longer keep pace.

```
[Same service, same 300 req/s target, two models]

CLOSED (100 VUs):          OPEN (300 iterations/s):
  server slows to 2s        server slows to 2s
  → VUs block               → k6 keeps firing
  → offered load falls      → queue depth climbs
     to ~50 req/s           → p99 → 12,000ms
  → p99 looks like 2,100ms  → dropped_iterations > 0
  → test PASSES  ❌         → test FAILS  ✅
```

Checkout endpoints are the canonical case for the open model: they hold inventory locks and payment-gateway connections, so their failure mode is queue collapse under sustained arrival pressure rather than a slow, linear degradation.

You will now simulate this in the Crucible: configure a `ramping-arrival-rate` spike against `/checkout` and enforce p99 and dropped-iteration thresholds so the run fails loudly when the service cannot hold the rate.
