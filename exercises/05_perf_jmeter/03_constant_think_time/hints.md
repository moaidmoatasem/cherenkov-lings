# Hints: Drill 03 - Constant Think Time vs Gaussian Random Timer

## Hint 1 (Concept)
Real users do not click at machine speed. Between actions they read, scroll, and think. This "think time" is critical for realistic load tests. A Constant Timer with 0ms removes all think time and makes your test look like a DDoS attack -- not real user behaviour.

## Hint 2 (Pattern)
Replace Constant Timer with Gaussian Random Timer:
  - Deviation (ms): 500
  - Constant Delay Offset (ms): 1000
This produces wait times centered around 1000ms with natural variation.

## Hint 3 (Answer)
In JMeter GUI: right-click Thread Group, Add > Timer > Gaussian Random Timer.
Set Deviation = 500ms, Constant Delay Offset = 1000ms.
Remove any Constant Timer set to 0ms.
