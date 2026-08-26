# Theoretical Context: Single-Runner Test Suites and the Illusion of Coverage

## Production Incident: The `ubuntu-latest` Rollover (2022–2023)

GitHub's `ubuntu-latest` label is not a version — it is a moving pointer. When GitHub migrated it from Ubuntu 20.04 to 22.04, and later began the move to 24.04, pipelines that had been green for years started failing overnight on code nobody had touched. The image change brought a different default Python, a different OpenSSL, a different Node, different preinstalled packages, and stricter defaults in tooling that had previously been lenient. Teams that pinned nothing and tested on exactly one runner discovered their entire notion of "the suite passes" had been a statement about one machine image, not about their software.

The same class of failure has a quieter, more expensive variant: the suite is green on the CI runner and broken for half the engineering team. Linux filesystems are case-sensitive; macOS and Windows are not, by default. `import { UserCard } from './usercard'` resolves fine on a developer's Mac and on a case-insensitive checkout, and fails at runtime on a Linux production container. A single-runner matrix cannot see this, because the disagreement *is between runners*.

## The Underlying Mechanism

1. **A test result is a claim about an environment, not about code.** "The suite passes" is shorthand for "the suite passed on this OS, this runtime version, this locale, this filesystem semantics." A pipeline with one runner supports exactly one such claim.
2. **`strategy.matrix` is a cartesian product.** Declaring two axes — `os: [ubuntu-latest, windows-latest]` and `node-version: [20, 22]` — expands into 2 × 2 = 4 independent job instances scheduled in parallel. Each gets a clean runner; each reports separately, so a failure names the exact combination that broke.
3. **`fail-fast` controls the diagnosis.** The default `fail-fast: true` cancels every sibling instance the moment one fails, which tells you *that* something broke but not *how widely*. Setting `fail-fast: false` lets all combinations finish, converting a single red X into a coverage map: one cell red means a platform bug, all cells red means a genuine regression.
4. **Matrix axes are chosen from real risk, not from availability.** Test the runtimes you actually support, the oldest and newest of them, plus the platforms your engineers develop on. Adding axes multiplies cost linearly, so each one should correspond to an environment where a failure would matter.

```
[Anti-Pattern: One Runner, One Claim]

  jobs.e2e-tests
    runs-on: ubuntu-latest      ← a moving pointer, not a version
    node-version: 20
         │
         └──► 1 job instance ──► ✅ green
                                     │
                    "the suite passes" really means
                    "it passed on whatever ubuntu-latest
                     pointed at today, on Node 20"        ❌

[Resilient Pattern: Matrix Expansion]

  strategy:
    fail-fast: false
    matrix:
      os:           [ubuntu-latest, windows-latest]
      node-version: [20, 22]
         │
         ├──► ubuntu-latest  / node 20  ──► ✅
         ├──► ubuntu-latest  / node 22  ──► ✅
         ├──► windows-latest / node 20  ──► ✖  path separator assumption
         └──► windows-latest / node 22  ──► ✖
                                     │
                    one red column names the platform bug
                    precisely, instead of hiding it        ✅
```

Parallel matrix execution is also the cheapest wall-clock win in CI: four instances running concurrently finish in roughly the time of the slowest one, not the sum of all four.

You will now simulate this in the Crucible: run `cherenkov-lings pipeline validate` against the workflow, read the `MISSING_MATRIX_STRATEGY` finding, and expand the test job across operating systems and runtime versions until the policy score reaches 100/100.
