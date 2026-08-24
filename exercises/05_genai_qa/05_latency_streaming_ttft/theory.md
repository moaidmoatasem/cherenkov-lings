# Theoretical Context: LLM Streaming Performance & Time-To-First-Token (TTFT)

## Real-World Incident Case Study
During high inference load, LLM gateway buffers can delay streaming frames. Users perceive slow TTFT as application unresponsiveness, even if token throughput is high.

## Protocol & Runtime Mechanism
Streaming responses utilize Server-Sent Events (`text/event-stream`) over HTTP/1.1 or HTTP/2 chunked transfer encoding:

$$\text{Total Duration} = \text{TTFT} + \sum_{i=1}^{N} \Delta t_{\text{token}_i}$$

```
  Client                     Gateway                     LLM Engine
    ¦                           ¦                            ¦
    +--- GET /api/llm/stream --?+--- Inference Queue -------?¦
    ¦                           ¦                            ¦ (KV-Cache Compute)
    ¦?-- 1st Token (TTFT) ------+?-- First Token Generated --¦
    ¦?-- 2nd Token -------------+?-- Next Token -------------¦
```

## You will now simulate this in the Crucible
Run `cherenkov-lings watch --track=genai-qa` and verify the streaming SLA.
