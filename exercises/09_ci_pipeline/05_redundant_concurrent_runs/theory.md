# Theoretical Context: Queue Starvation and Superseded CI Runs

## Production Incident: The Self-Inflicted Queue (why `concurrency` shipped in 2021)

GitHub added top-level `concurrency` to Actions in 2021 to solve a problem every growing engineering organisation reaches independently and usually blames on the CI provider. The symptom is always the same: builds that used to start immediately now sit in a queue for twenty minutes, the runner fleet looks saturated, and the obvious remedy — buy more runners — makes the bill grow without making the queue shorter.

The cause is not capacity. It is that a large fraction of the fleet is executing work whose result nobody will ever read. An engineer pushes six times in ten minutes to the same pull request — a typo fix, a lint fix, a rebase, a review comment addressed. Without a concurrency policy each push starts a full matrix run, so six pushes produce six concurrent runs of four instances: twenty-four job instances, of which twenty are already testing commits that have been superseded. They will run to completion, report on code that no longer exists at the branch head, and hold runner slots the entire time — while a colleague's release-blocking build waits behind them.

## The Underlying Mechanism

1. **CI results are positional, not cumulative.** The only run whose verdict has any bearing on a merge decision is the run against the current head of the branch. A run against an intermediate commit is not partial information; it is *obsolete* information, and reporting it is actively misleading when it lands after a newer run.
2. **A concurrency group is a mutual-exclusion key.** `concurrency.group` is an arbitrary string; the platform allows one active run per distinct value. Keying it on `${{ github.workflow }}-${{ github.ref }}` scopes exclusion to one workflow on one branch, so unrelated branches never contend with each other.
3. **`cancel-in-progress` decides which run wins.** With `false` — the default — a new run *queues behind* the in-flight one, which is the worst of both worlds: the stale run still consumes the slot, and the run you care about is delayed by its full duration. With `true`, the in-flight run is cancelled the moment a newer one enters the group, so the fleet only ever executes current work.
4. **The exception is deployment.** For jobs with side effects — publishing, deploying, migrating — cancelling mid-flight can leave a partial rollout. Those workflows want a concurrency group with `cancel-in-progress: false`, so runs serialise rather than interrupt. Test workflows want the opposite, and this one is a test workflow.

```
[Anti-Pattern: cancel-in-progress: false]

  push #1 ──► run A ████████████████████████ 20 min  (obsolete after push #2)
  push #2 ──►        run B  … queued behind A …  ████████████  (obsolete)
  push #3 ──►                    run C … queued …  ████████████
                                                         │
   HEAD is at #3, but the fleet spends ~60 minutes of slots
   producing two verdicts nobody will read, and the verdict
   that matters arrives last                                    ❌

[Resilient Pattern: cancel-in-progress: true]

  concurrency:
    group: ${{ github.workflow }}-${{ github.ref }}
    cancel-in-progress: true

  push #1 ──► run A ██████╳ cancelled
  push #2 ──►        run B ███╳ cancelled
  push #3 ──►             run C ████████████████ 20 min ──► ✅ verdict on HEAD

   one verdict, on the only commit that matters, slots
   returned to the pool the instant they became useless        ✅
```

The counter-intuitive result is that cancelling work aggressively *increases* effective throughput: capacity spent on superseded commits is capacity not spent on anyone's release.

You will now simulate this in the Crucible: run `cherenkov-lings pipeline validate` against the workflow, read the `CONCURRENCY_CANCEL_DISABLED` finding, and make a newer push supersede the run already in flight until the policy score reaches 100/100.
