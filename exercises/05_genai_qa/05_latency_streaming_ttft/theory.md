# Theoretical Context: LLM Streaming Performance & Time-To-First-Token (TTFT)

## Real-World Incident Case Study
In early 2024, a major AI-powered customer support platform experienced a 3x increase in support ticket escalation rates after deploying a new LLM inference gateway. User telemetry revealed that the median Time-To-First-Token (TTFT) had increased from 800ms to 2.4 seconds under peak load. The root cause was a batch scheduling policy that accumulated requests into 512-token batches before initiating inference. While this improved GPU utilization by 40%, it introduced a queuing delay that users perceived as application unresponsiveness. Reverting to streaming-first scheduling restored TTFT to under 1 second while maintaining acceptable throughput.

## Protocol & Runtime Mechanism
Streaming responses utilize Server-Sent Events (`text/event-stream`) over HTTP/1.1 or HTTP/2 chunked transfer encoding. The total user-perceived latency decomposes into two phases:

```
  Total Duration = TTFT + Σ(Δt_token_i) for i=1 to N

  Client                     Gateway                     LLM Engine
    │                           │                            │
    ├── GET /api/llm/stream ──→├── Inference Queue ────────→│
    │                           │                            │ (KV-Cache Compute)
    │←── 1st Token (TTFT) ─────┤←── First Token Generated ─┤
    │←── 2nd Token ────────────┤←── Next Token ────────────┤
    │←── ...                    │←── ...                     │
```

**TTFT** measures the interval from request initiation to the first byte of the streaming response. It encompasses network latency, queue wait time, prompt processing (prefill), and the first forward pass through the decoder. For interactive applications, TTFT directly impacts perceived responsiveness because users see a blank screen until the first token arrives.

**Inter-token latency** (the delta between successive tokens) determines the streaming throughput. Users read at roughly 200-400 words per minute, so inter-token latency below 100ms feels instantaneous while above 500ms feels sluggish.

## Testing TTFT in Practice
Measuring TTFT requires instrumenting the HTTP client to record the timestamp when the first SSE frame arrives, not when the request is sent. Common pitfalls:
- Measuring from request sent (includes network RTT) instead of first byte received
- Using wall-clock time without accounting for DNS resolution and TLS handshake overhead
- Testing against a warm cache instead of cold-start conditions

k6 and JMeter both support streaming response measurement, but require custom scripting beyond their default HTTP request patterns. Playwright's `page.route` can intercept SSE streams for browser-based TTFT validation.

## You will now simulate this in the Crucible
Run `cherenkov-lings watch --track=genai-qa` and verify the streaming SLA by measuring TTFT against the Crucible's LLM mock endpoint.
