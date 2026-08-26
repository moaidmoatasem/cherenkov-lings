# Theoretical Context: Unbounded Jobs, Hung Runners, and CI as an Attack Surface

## Production Incident: Cryptomining Abuse Forces the Free-CI Reckoning (2020–2021)

Between 2020 and 2021 the major CI providers discovered the same thing at roughly the same time: free, generously-timed build minutes are an excellent way to mine cryptocurrency on someone else's electricity bill. Attackers automated the pattern — fork a public repository, open a pull request that triggers the workflow, and have the job run a miner for as long as the platform allowed. GitHub, GitLab, Docker Hub, and Travis CI all tightened free-tier CI in response, restricting who could trigger workflows from forks, throttling anonymous builds, and in Travis's case ending the open-source free tier substantially because of the abuse.

What made the abuse economical was not a clever exploit. It was the default: a job that declares no time limit runs until the platform's ceiling, and on GitHub Actions **that ceiling is 360 minutes — six hours — per job.** The same default hurts honest pipelines just as reliably. A Playwright worker deadlocks on a browser handle, the job stops producing output, and nobody notices until six hours of matrix instances have burned through the month's budget and every other team's builds have been queued behind them.

## The Underlying Mechanism

1. **The default is six hours, and it is not a safety net.** `timeout-minutes` on a job defaults to 360. That number was chosen to accommodate the slowest legitimate build on the platform, not to protect any particular repository. A suite that normally finishes in eight minutes has, by default, fifty-nine minutes of slack for every one minute of work.
2. **A hung job is worse than a failed one.** A failure returns a signal and frees the runner. A hang consumes a concurrency slot for the full ceiling while producing nothing, and under a matrix it consumes one slot *per combination* — four instances hanging for six hours each is a full day of runner time spent on zero information.
3. **The budget is the correct unit of reasoning.** Multiply: minutes × matrix instances × runs per day × price per minute. A four-instance matrix on a six-hour ceiling can cost more in a single stuck night than the entire pipeline does in a normal month, and on self-hosted fleets the cost is paid in queue depth for everyone else.
4. **The bound should be derived, not guessed.** Take the observed p95 duration of the job and allow modest headroom — typically 2–3×. That is tight enough to catch a hang within minutes and loose enough to survive a slow runner or a cold cache. Deleting the key is *not* a fix: absence means 360.

```
[Anti-Pattern: Effectively Unbounded]

  timeout-minutes: 360        (or omitted — same thing)
  p95 actual duration: ~8 min

  browser handle deadlocks at 00:03
  │
  ├─ 00:03 ──────────────────────────────────── 06:03  runner held, no output
  ├─ 00:03 ──────────────────────────────────── 06:03  ×4 matrix instances
  │
  └──► 24 runner-hours burned, zero information gained
       every other team's builds queued behind it        ❌

[Resilient Pattern: Bounded by Observed Behaviour]

  timeout-minutes: 30         (p95 ≈ 8 min, ~3× headroom)

  browser handle deadlocks at 00:03
  │
  ├─ 00:03 ─────────► 00:30  job killed, marked failed
  │
  └──► 27 runner-minutes spent, slot returned to the pool,
       red build visible on the dashboard immediately     ✅
```

Fail-fast is a cost control and an observability control at once: the sooner a hang is converted into a failure, the sooner it becomes a thing a human can see and act on.

You will now simulate this in the Crucible: run `cherenkov-lings pipeline validate` against the workflow, read the `EXCESSIVE_TIMEOUT` finding, and bound the job to something derived from its real runtime until the policy score reaches 100/100.
