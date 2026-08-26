# Hints: Drill 04 - Runaway Job Timeout

## Hint 1 (Concept)
This job declares a six-hour ceiling for a suite that finishes in minutes. A deadlocked browser handle will therefore hold a runner — and one runner *per matrix combination* — for the full six hours while producing no output at all.

Note the trap: deleting `timeout-minutes` entirely is not a fix. The platform default is 360 minutes, which is exactly the value already there. You need a real bound, not the absence of one.

## Hint 2 (Syntax)
The bound is a job-level key, a plain integer in minutes:

```yaml
  <job-id>:
    runs-on: ...
    timeout-minutes: <integer>
```

Derive the number rather than guessing it: take the job's observed p95 duration and allow roughly 2–3× headroom. That is tight enough to convert a hang into a visible failure within minutes, and loose enough to tolerate a cold cache or a slow runner.

The policy flags anything above 120 minutes as excessive, and flags the key's absence separately.

## Hint 3 (Snippet)
This suite's steps — checkout, setup, `npm ci`, a Playwright run — land around eight minutes on a warm runner. Replace:

```yaml
  e2e-tests:
    name: Playwright E2E Suite
    runs-on: ${{ matrix.os }}
    timeout-minutes: 360
```

with a bound derived from that:

```yaml
  e2e-tests:
    name: Playwright E2E Suite
    runs-on: ${{ matrix.os }}
    timeout-minutes: 30
```

Anything in the 15–30 range is defensible here. The point is that the number comes from how long the job actually takes, not from the platform's maximum.
