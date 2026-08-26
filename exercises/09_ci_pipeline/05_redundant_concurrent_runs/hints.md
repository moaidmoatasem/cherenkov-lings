# Hints: Drill 05 - Redundant Concurrent Runs

## Hint 1 (Concept)
This workflow already has a concurrency group, so the grouping key is not the problem — read the setting underneath it.

The question to answer is: when a second push lands on the same branch while a run is still in flight, what should happen to the first run? Its verdict concerns a commit that is no longer the branch head, so it is not partial information — it is obsolete information, and it is holding runner slots while it produces it.

## Hint 2 (Syntax)
The top-level concurrency block has two keys:

```yaml
concurrency:
  group: <mutual-exclusion-key>
  cancel-in-progress: <boolean>
```

`group` scopes the exclusion — keying it on workflow plus ref, as this file already does, means one active run per workflow per branch. `cancel-in-progress` decides the outcome of a collision: `false` queues the newcomer behind the stale run, `true` terminates the stale run immediately.

Be aware of the inverse case for real pipelines: deployment workflows want `false`, because cancelling a half-finished rollout is worse than waiting. This is a test workflow.

## Hint 3 (Snippet)
Change the single boolean:

```yaml
concurrency:
  group: ${{ github.workflow }}-${{ github.ref }}
  cancel-in-progress: false
```

to:

```yaml
concurrency:
  group: ${{ github.workflow }}-${{ github.ref }}
  cancel-in-progress: true
```

One word, and the fleet stops spending capacity on commits nobody will merge.
