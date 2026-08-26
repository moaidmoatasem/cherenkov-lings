# Theoretical Context: Ephemeral Runners and the Evidence That Never Survived

## Production Incident: GitLab's Database Deletion and the Five Broken Backups (31 January 2017)

While fighting a spam-driven load spike, a GitLab engineer intended to clear a replica's data directory and ran the removal against the wrong host, deleting roughly 300 GB from the primary database. That was the accident. The incident was what came next: GitLab had five separate backup and recovery mechanisms, and in the moment they were needed, none of them worked as believed — pg_dump was silently failing against a version mismatch, disk snapshots were not enabled on the relevant volume, and the S3 backup bucket was empty. Recovery came from a staging snapshot that happened to have been taken six hours earlier by chance. Roughly six hours of data was lost permanently.

GitLab published the whole thing, live and in public, and the durable lesson is a testing lesson: **evidence you have not verified you can retrieve is evidence you do not have.** A CI pipeline reproduces this failure in miniature every night. A flaky test fails at 02:14 on the Windows runner; the trace file, the video, the screenshots, and the HTML report are all written to the runner's local disk; the job ends; the container is destroyed; and the only artefact that survives to morning is a red X and a truncated log tail. The engineer who picks it up cannot reproduce it — not because the bug is hard, but because the evidence was deleted by design.

## The Underlying Mechanism

1. **Runners are ephemeral by construction.** A hosted runner is a fresh VM or container per job. Its filesystem exists for the duration of the job and is destroyed immediately afterwards. Anything written to `playwright-report/`, `test-results/`, `allure-results/`, or `target/surefire-reports/` is gone the moment the job terminates.
2. **The log is not the evidence.** Console output is truncated, interleaved across parallel matrix instances, stripped of colour and structure, and cannot carry a screenshot, a video, a DOM snapshot, a HAR file, or a Playwright trace. Modern test tooling produces rich failure artefacts precisely because a stack trace alone is rarely sufficient for a timing bug.
3. **Artefact upload must be unconditional.** This is the subtle part. A step with no condition inherits an implicit `if: success()`, so an upload step placed after the test step **does not run when the tests fail** — exactly the case you needed it for. `if: always()` overrides that, running the upload on failure, on success, and on cancellation.
4. **Retention is a policy decision.** Artefacts expire. Setting a retention window deliberately is what makes flakiness analysis possible across weeks rather than hours, and it is also what keeps storage costs bounded.

```
[Anti-Pattern: Evidence Dies With the Runner]

  runner VM (ephemeral)
  ┌──────────────────────────────────────────┐
  │ npx playwright test          ──► ✖ FAIL  │
  │   playwright-report/index.html           │
  │   test-results/…/trace.zip               │
  │   test-results/…/video.webm              │
  └──────────────────┬───────────────────────┘
                     │ job ends
                     ▼
              VM destroyed
                     │
                     └──► all that survives: a red X   ❌
                          "cannot reproduce, closing"

[Resilient Pattern: Unconditional Artefact Export]

  runner VM (ephemeral)
  ┌──────────────────────────────────────────┐
  │ npx playwright test          ──► ✖ FAIL  │
  │                                          │
  │ upload-artifact  (if: always())          │
  │   └── archives report + traces + video ──┼──► artefact store
  └──────────────────┬───────────────────────┘         │
                     │ job ends                        │
                     ▼                                 ▼
              VM destroyed              trace.zip downloadable,
                                        replayable in the Playwright
                                        trace viewer, days later    ✅
```

Without `if: always()` the upload step is decorative: it runs on the green builds where you do not need it, and skips the red ones where you do.

You will now simulate this in the Crucible: run `cherenkov-lings pipeline validate` against the workflow, read the `MISSING_ARTIFACT_UPLOAD` finding, and make failure evidence outlive the runner until the policy score reaches 100/100.
