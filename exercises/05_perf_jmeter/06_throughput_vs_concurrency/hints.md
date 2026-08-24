# Hints: Drill 06 - Throughput vs Concurrency

## Hint 1 (Concept)
Most beginners set Thread Count = desired RPS. This is wrong. If your server takes 500ms to respond, 1 thread can only generate 2 requests/second. To get 100 RPS you need 50 threads -- not 100. Thread count depends on response time. Use the Throughput Shaping Timer plugin to specify RPS directly and let JMeter calculate threads automatically.

## Hint 2 (Pattern)
Install the Throughput Shaping Timer via JMeter Plugin Manager.
Configure it with a ramp schedule:
  Start: 10 RPS, Duration: 60s
  Peak:  100 RPS, Duration: 120s
  End:   10 RPS, Duration: 30s
JMeter will automatically adjust thread counts to hit these targets.

## Hint 3 (Answer)
Formula: Required Threads = Target RPS x Average Response Time (seconds)
If avg response = 200ms (0.2s) and target RPS = 100:
  Threads needed = 100 x 0.2 = 20 threads
Monitor "Active Threads" in your dashboard to verify.
