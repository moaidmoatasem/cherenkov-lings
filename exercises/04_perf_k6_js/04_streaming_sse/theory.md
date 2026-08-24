# Theoretical Context: Server-Sent Events (SSE) Streaming Concurrency

## Production Incident: Slack Real-Time Gateway Event Drop (2019)

In 2019, Slack experienced a major real-time messaging degradation when peak morning enterprise traffic caused widespread event delivery stalls across active desktop and mobile clients. During morning login surges, hundreds of thousands of simultaneous clients maintained open streaming connections to the edge gateway. When an unexpected backend cache invalidation caused event emission rates to surge, client streaming buffers filled faster than event parsers could process them. Downstream connection pools became exhausted, socket buffers overflowed, and persistent Server-Sent Events (SSE) connections were dropped en masse. The outage highlighted that real-time event streams cannot be tested as standard request-response endpoints; they require dedicated streaming endurance and concurrent connection lifecycle testing.

## The Underlying Mechanism

Server-Sent Events (SSE) utilize the `text/event-stream` MIME type over long-lived HTTP/1.1 or HTTP/2 connections to push real-time updates from server to client without the overhead of bidirectional WebSockets:

1. **Protocol Characteristics**: The server responds with `Content-Type: text/event-stream`, `Cache-Control: no-cache`, and `Connection: keep-alive`, holding the TCP socket open indefinitely and pushing chunks formatted as `event: ...\ndata: ...\n\n`.
2. **The Anti-Pattern in Load Testing**: Traditional HTTP load testing tools execute a single `GET` request, read the first chunk of bytes, and terminate the socket immediately. This naive check fails to test long-lived connection stability, buffer memory accumulation, heartbeat keep-alives, connection backpressure, or server socket exhaustion under sustained concurrent streams.
3. **Resilient Streaming Testing in k6**: Using k6 streaming modules or continuous response consumption loops, performance engineers open long-lived connections, parse incoming SSE frames, measure inter-event latency intervals, and assert stream integrity over time under high concurrent Virtual User (VU) counts.

```
[Anti-Pattern: Naive Request-Response Disconnect]
k6 Virtual User ──── GET /events/stream ────► Server (Opens SSE Stream)
k6 Virtual User ◄─── Chunk 1 (100 bytes) ─── Server (Keeps Socket Open)
k6 Virtual User ──── Socket Closed (Exit) ──► Server (Socket Abandoned / Leak!)

[Resilient SDET Pattern: Long-Lived Concurrent Stream Verification]
k6 Virtual User ──── GET /events/stream ────► Server (Stream Active)
k6 Stream Handler ◄── Event 1: Heartbeat ──── Server
k6 Stream Handler ◄── Event 2: Data Payload ─ Server (Assert Frame Schema)
k6 Stream Handler ◄── Event N: Message ────── Server (Measure Inter-Arrival Time)
k6 Stream Handler ─── Graceful Teardown ────► Server (Clean Connection Close)
```

Testing streaming endpoints ensures that edge load balancers, reverse proxies, and backend thread pools maintain stable memory profiles and low delivery jitter under sustained concurrency.

You will now simulate this in the Crucible: author a k6 streaming test to validate continuous Server-Sent Events from `/events/stream` under concurrent load and handle connection drops cleanly.
