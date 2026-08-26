# Hints: Drill 03 - Lost Failure Artifacts

## Hint 1 (Concept)
CI runners are destroyed the instant the job ends. Every report, trace, video, and screenshot your test framework wrote to the runner's local disk dies with it — so a failure at 02:14 leaves the on-call engineer a red X and nothing to reproduce from.

The test job needs a step that copies failure evidence off the runner and into durable artefact storage before the container is torn down.

## Hint 2 (Syntax)
Artefacts are exported with a dedicated action that takes a name and a path:

```yaml
      - name: <descriptive name>
        uses: actions/upload-artifact@v4
        with:
          name: <artifact-name>
          path: <directory-or-glob>
```

There is a trap here that catches almost everyone. A step with no `if:` behaves as `if: success()`, which means **it is skipped when a previous step failed** — precisely the run where you needed the evidence. Read the condition carefully before you consider this drill finished.

## Hint 3 (Snippet)
The test step writes its report to `playwright-report/`, then the job ends and it is gone. Append an upload step that runs regardless of the test outcome:

```yaml
      - name: Upload Playwright report
        if: always()
        uses: actions/upload-artifact@v4
        with:
          name: playwright-report
          path: playwright-report/
```

`if: always()` is the load-bearing line. Without it the step inherits `if: success()`, uploads happily on green builds, and silently skips every red one.
