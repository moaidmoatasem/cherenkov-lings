# Hints: Drill 02 - Missing Matrix Strategy

## Hint 1 (Concept)
A green suite is a claim about one environment. This job pins itself to a single runner image and a single Node version, so it can only ever tell you that the code works *there* — it is structurally blind to platform-specific bugs like path separators, case-sensitive imports, or runtime API changes.

The validator treats any job it recognises as a testing job (by name, or by the commands it runs) as requiring real parallel coverage.

## Hint 2 (Syntax)
A matrix is declared under the job's `strategy` key and expands into one job instance per combination:

```yaml
    strategy:
      matrix:
        <axis-name>: [value1, value2]
        <other-axis>: [value1, value2]
```

Two axes with two values each produce four parallel instances. Reference a matrix value elsewhere in the job with `${{ matrix.<axis-name> }}` — including in `runs-on`, which is how a single job definition targets several operating systems.

Add `fail-fast: false` alongside `matrix` so one failing combination does not cancel the others; you want the full coverage map, not the first casualty.

## Hint 3 (Snippet)
Replace the hardcoded runner and Node version:

```yaml
  e2e-tests:
    name: Playwright E2E Suite
    runs-on: ubuntu-latest
    timeout-minutes: 30
    steps:
      - uses: actions/setup-node@v4
        with:
          node-version: 20
```

with a matrix-driven job:

```yaml
  e2e-tests:
    name: Playwright E2E Suite
    runs-on: ${{ matrix.os }}
    timeout-minutes: 30
    strategy:
      fail-fast: false
      matrix:
        os: [ubuntu-latest, windows-latest]
        node-version: [20, 22]
    steps:
      - uses: actions/setup-node@v4
        with:
          node-version: ${{ matrix.node-version }}
```

Note that a matrix with only one effective combination still fails the policy — the point is coverage, not the presence of the keyword.
