# Handover: Cherenkov-Lings Remediation

Written 2026-09-04 by a Claude Code session that ran out of budget mid-implementation.
This file is the single source of truth for what's done, what's in flight, and what's
left. It inlines the full approved plan so it's self-contained — no dependency on any
file outside this repo.

**Read this file fully before touching anything.** Then re-run
`git log --oneline -5` and `git status` to confirm the state described below still
matches reality (another agent may have picked this up since).

---

## How this happened

A prior session ran a 3-part audit (curriculum, frontend UX, documentation) via
background agents, published the findings as an Artifact, then turned the findings
into a 7-phase remediation plan (approved by the user via plan mode). Implementation
started phase by phase. The session was interrupted mid-Phase-1 to hand off before
running out of context budget. **Nothing has been committed or pushed yet** — that's
the first thing the next agent should do.

---

## STATUS LEDGER — read this first

### ✅ DONE (verified applied, uncommitted in working tree)

**Phase 0 — Canonical numbers (68 drills / 13 tracks, was 65/12 or 60/11 in places):**
- `README.md` — curriculum table headline + rows fixed to 68/13, added missing
  `api-pytest` row, fixed `maestro-mobile` (5→6) and `k6-js` (5→6) drill counts.
- `CHEATSHEET.md` — `watch --track=` list expanded to all 13 real tracks (was 7) plus
  a `getting-started` line for the **not-yet-created** Phase 4 track (see warning
  below); added the 4 missing top-level commands (`review`, `pipeline run`, `triage`,
  `report`) with verified-correct flags (checked against `src/main.rs`'s actual
  `clap` arg definitions, not guessed).
- `TEST_READY.md` — curriculum table reconciled to 68/13 (added `api-pytest` and
  `ci-pipeline` rows).
- `crucible/frontend/src/learn/useLearnerProgress.ts` — `SEEDED_TOTAL_MODULES`/`_TRACKS`
  bumped 65/12 → 68/13.
- `crucible/frontend/src/pages/MissionControlPage.tsx` — line ~53 badge description
  and line ~205 marketing copy no longer hardcode "60 drills / 11 tracks".

**Phase 1 — Quick fixes:**
- 1.1 **CHEATSHEET.md Locator Quality table** — rewritten to match the real 5-tier
  engine in `src/feedback.rs:70-78` (`getByRole=100`, text/label/placeholder/altText/
  title=90, `getByTestId=85`, CSS=40, XPath=0). Previously had 7 invented tiers with
  `getByText` scored *below* `getByTestId`, which is backwards vs. the real engine.
- 1.2 **Mojibake re-encoding** — done in `CHEATSHEET.md` (was 35 literal `?` bytes),
  `CONTRIBUTING.md` (10), `AI_AGENT_INSTRUCTIONS.md` (6), `crucible/start.sh` (15,
  including 3 lines with `�` replacement-character corruption, not just `?`). All
  re-encoded with intentional emoji restored.
- 1.5 **`src/main.rs` `Commands::Watch` fallback** — added the same
  `.unwrap_or_else(|_| gamification::embedded_config())` pattern already used by
  `Dashboard`/`Audit`/`NewDrill`, so `watch` now works from outside the repo root
  (previously hard-errored via a bare `?` on `config::load_config("lings.toml")`).
  **This changes Rust code — `cargo build`/`cargo test` have NOT been run to confirm
  it compiles.** Do that first.
- 1.6 **`lings.toml` drill reordering** — `playwright-ts` track: `04_first_playwright_test`
  moved to position 1 (was 4th, after 3 advanced drills). `maestro-mobile` track:
  `06_login_flow` moved to position 1 (was 6th/last). No directory renames — confirmed
  safe because drill presentation order comes purely from TOML array order
  (`src/config.rs`, `crucible/backend/curriculum.py` both preserve array order) and
  `tests/curriculum_manifest_tests.rs` never checks name-vs-position. **Not yet
  verified with `cargo test --all` — do that first**, specifically
  `tests/curriculum_manifest_tests.rs`.

### 🔶 IN PROGRESS — stopped here, mid-investigation

**Phase 1.3 — Rewrite the two weak DevSecOps drills**
(`exercises/07_cloud_devsecops/01_insecure_docker_mount` and `02_jwt_weak_signing_key`).

What's confirmed:
- Both `hints.md` files are missing `## Hint 3` (only have Hint 1 + Hint 2).
- Both `exercise.py`/`solution.py` assert against a Python dict literal defined in the
  same file — never touch the running Crucible. This is the weakest pattern in the
  repo; drills 03/04/05 in the same track (SQLi, SSRF, CORS) are the model to copy —
  `exercise.py` uses `requests.post("http://localhost:8081/...", ...)`, `solution.py`
  uses `TestClient(app)` from `fastapi.testclient` against the same endpoint. See
  `exercises/07_cloud_devsecops/04_ssrf_metadata_service/{exercise,solution,hints}.py`
  as the exact template — already read in full this session, pattern confirmed clean.
- **Blocker found**: unlike SQLi/SSRF/CORS, there is **no existing backend endpoint**
  for either docker-mount config or JWT-algorithm-none bypass. Grepped
  `crucible/backend/app.py` for `docker|jwt|/api/security` — only 3 security endpoints
  exist: `GET /api/security/user-lookup`, `POST /api/security/fetch-url` (used by SSRF
  drill), `GET /api/security/cors-sensitive` (used by CORS drill). JWT *login itself*
  exists (`app.py:457-495`, uses `jwt.encode`/`jwt.decode` with a `SECRET_KEY`), which
  is promising for drill 02 — but was not yet confirmed to reject `alg=none` tokens.
  Docker-mount (drill 01) has **no runtime HTTP surface at all** — it's fundamentally
  a static deployment-config concern (Tesla's 2018 incident was about a K8s manifest),
  not something the FastAPI backend can expose as a request/response check.

**Recommended resolution (not yet implemented):**
- **Drill 02 (JWT)**: read `app.py:440-500` in full, confirm whether `jwt.decode(...,
  algorithms=[ALGORITHM])` already rejects an attacker-forged `alg: none` token (PyJWT
  does by default when `algorithms=` is a fixed allowlist — likely already secure,
  which would make this a genuine "write a test proving the fix already holds" drill,
  a legitimate and common real-world pattern). Write `exercise.py` using `requests` to
  POST a hand-crafted `alg: none` JWT to whichever endpoint accepts `Authorization:
  Bearer <token>` (find via `get_current_user`, referenced at `app.py:483`), asserting
  it's rejected (401/403). Mirror the SSRF drill's `solution.py` structure with
  `TestClient`.
- **Drill 01 (Docker mount)**: since no live endpoint exists, either (a) add a small
  new endpoint to `crucible/backend/app.py` — e.g. `POST /api/security/validate-deploy-config`
  that accepts a container/volume config JSON and returns 403 if it mounts
  `/var/run/docker.sock` (consistent with `AI_AGENT_INSTRUCTIONS.md`'s explicit
  allowance: *"You may add new endpoints or new failure mechanisms"* to the Crucible),
  or (b) accept that this one drill stays a static-analysis-style test (not every
  DevSecOps drill needs a live HTTP call) but still strengthen it beyond a same-file
  dict literal — e.g. have it parse a real `docker-compose.yml`-style fixture file
  checked into the drill directory. (a) is more consistent with the rest of the track;
  recommend it if time allows.
- Both: add `## Hint 3 (Code Diff)` to both `hints.md`, matching the SSRF drill's
  hints.md format exactly (Hint 1 = architectural nudge, Hint 2 = API pattern/what to
  assert, Hint 3 = literal diff).
- Verify with `cargo run -- audit` (checks 4-file contract + hint count) and
  `python -m pytest` on both exercise/solution pairs against a running Crucible.

### 🔲 NOT STARTED

**Phase 1.4 — README Learn UI documentation**
Add a section to `README.md` documenting the Learn UI (routes `/`, `/learn`,
`/sandbox`, rendered by `crucible/frontend/src/learn/LearnApp.tsx`) — currently
undocumented anywhere despite being the actual landing experience at `localhost:8080`.
Also correct README's existing `:8080` description (in the "Quick Start" section),
which still describes only the old pathology-demo hub (now at `/sandbox` inside the
Learn shell, not at the bare root — see `crucible/frontend/src/App.tsx:105-118`).

**Phase 2 — Wire the three flagship tabs to their real backends**
Endpoints confirmed live and tested in `crucible/backend/app.py:1025-1147`:
- `POST /api/review` `{code, file_path}` → `ReviewReport`; `POST /api/review/fix`.
- `POST /api/pipeline/validate` and `POST /api/pipeline/run`, both take
  `{workflow_yaml | yaml_content | content, ...}`.
- `GET /api/triage/tests?category=&failing_only=&track=`; `POST /api/triage/submit`.
- `GET /api/reports/allure`, `GET /api/reports/allure/html`.

Frontend changes needed (none started):
- `crucible/frontend/src/pages/CodeReviewPage.tsx` — `handleRunASTReview` (line 394)
  currently regex-scans hardcoded `CODE_TEMPLATES` (line 29) client-side and never
  calls the backend. Replace with `POST /api/review`. Wire "Apply Fix" to
  `POST /api/review/fix`. Use `apiUrl()` from `crucible/frontend/src/lib/api.ts`
  (already used elsewhere, e.g. `useLearnerProgress.ts`) — do not hardcode the base URL.
  Also: `crucible/frontend/src/components/StreamViewer.tsx` renders a `LIVE` badge
  (`role="status" aria-live="polite"`) over text admitting it's mocked — relabel or drop.
- `crucible/frontend/src/pages/PipelineBuilderPage.tsx` — `handleStartSimulation`
  (line 371) uses `window.setInterval` (line 415) to fabricate runner logs including
  invented lines like `"✓ 15 tests passed on Shard ${shard}"` (line 432). Replace with
  `POST /api/pipeline/validate` then `POST /api/pipeline/run`, driving the same
  progressive-reveal UI off the real `PipelineRunResult.jobs[].steps`/logs.
- `crucible/frontend/src/pages/AllureTriagePage.tsx` — hardcoded `CHAOS_TEST_CASES`
  (line 24) → `GET /api/triage/tests`. `handleEvaluateTriage` (line 270, client-side
  keyword heuristic) → `POST /api/triage/submit`. Drop local `earnedXP` state (starts
  at literal `350`, line 218) — use the XP the backend returns (persists to
  `.cherenkov-progress.json`, so it will actually show up on Mission Control instead of
  resetting on refresh). Hardcoded donut/KPI constants (lines 236-257, labeled
  `// Simulated full enterprise suite`) → `GET /api/reports/allure`.

**Phase 3 — Make the landing screen honest**
- `crucible/frontend/src/learn/TodayScreen.tsx` / `RecordScreen.tsx` — extend
  `useLearnerProgress` (already fetches `/api/progress` + `/api/curriculum`, see
  `crucible/frontend/src/learn/useLearnerProgress.ts`) to override more than just
  `points`/`modulesBuilt`. The "of 60, across four tracks" caption
  (`crucible/frontend/src/learn/content.ts:408`, `KPIS[0].sub`) needs to interpolate
  the live `modulesTotal`/`tracksTotal` the hook already fetches (lines 100-107 of
  `useLearnerProgress.ts`) but doesn't currently apply to this caption string.
- `Kept sessions` (86%) / `Time spent` (9h 40m) KPIs have no backend concept to back
  them — recommend relabeling as illustrative rather than wiring fake precision.
- 4 dead buttons with no click handler: `Reschedule`/`Start recall`
  (`TodayScreen.tsx:71-73,134-136`), `Preview it`/`Share settings`
  (`RecordScreen.tsx:78-83`) — remove or disable with a "coming soon" state.
- `crucible/frontend/src/pages/HomePage.tsx` — only links 4 of 13 sandbox pages
  (Checkout, Shadow DOM, Search, Transfer + a Mission Control button). Add links for
  the 8 it omits: Catalog, Dashboard, Payment, Profile, Mobile Test, Code Review,
  Pipeline Builder, Allure Triage (all already exist as routes in
  `crucible/frontend/src/App.tsx`).
- Accessibility: `aria-label` missing on the modal close button
  (`MissionControlPage.tsx` around line 530) and the pipeline stage-enable toggle
  (`PipelineBuilderPage.tsx` around line 624).

**Phase 4 — New `getting-started` on-ramp track (not created yet)**
⚠️ **`CHEATSHEET.md` already references `cherenkov-lings watch --track=getting-started`**
(added in the Phase 0 pass above) **for a track that does not exist yet.** Either
build this track next, or remove that line from CHEATSHEET.md if it won't be built
this pass — don't leave it dangling.

Plan: new track id `getting-started`, directory `exercises/00_getting_started`, added
as the **first** `[[tracks]]` entry in `lings.toml` (ahead of `foundations`). Reuses
the Python/pytest runner (same `command` line as the `foundations` track). Directory
prefix `00_` duplicating `00_foundations` is fine — `02_`/`09_` prefixes are already
shared by two tracks each, confirmed precedent.

4-5 checkpoints, each using the standard 4-file contract (`exercise.py`, `solution.py`,
`hints.md` with exactly 3 hints, `theory.md` ≥150 words) so
`tests/curriculum_manifest_tests.rs` and every manifest consumer treat it like any
other track:
1. Environment setup checkpoint (Python venv, running a script, interpreting output).
2. Git basics checkpoint (clone/commit/branch vocabulary, reading a diff).
3. Reading a stack trace / distinguishing a real failure from a flaky one.
4. Translating a manual test case into a single automated assertion — the actual
   manual-QA → automation bridge skill; this is the one most worth getting right.

Framed as walkthroughs with a deliberately low correctness bar, not testing-skill
challenges — the closest fit to "ungraded" the current pass/fail-scoring engine
supports without an engine change (user explicitly confirmed this framing when the
plan was approved). After adding: update `lings.toml`'s `foundations` track
description and `README.md`'s track table to note `getting-started` as the true
starting point, and add `## 0. Getting Started` as the new first row in README's
curriculum table (currently starts at "0. Foundations").

**Phase 5 — Tool fidelity: Contract-Pact and A11y-Axe (user explicitly chose "wire in the real libraries")**
- `exercises/09_contract_pact/*` (3 drills) — none import `pact-python` today; all use
  plain `requests`/`TestClient` JSON-shape assertions dressed in Pact vocabulary while
  `theory.md` in each accurately teaches real Pact mechanics. Add `pact-python` to the
  pinned `pip install` lines in `.github/workflows/ci.yml` (3 occurrences — search for
  `pip install` in that file; no `requirements.txt` covers exercises, deps are declared
  inline there, matching the existing pattern for `pytest`/`requests`/`httpx`). Rewrite
  all 3 solutions to use real `Consumer`/`Provider`/`Verifier` from `pact-python`.
- `exercises/10_a11y_axe/*` (3 drills) — none use `axe-core`; all use manual
  `getByRole`/`toBeFocused()` Playwright assertions. Add `@axe-core/playwright` to the
  root `package.json` devDependencies (currently only has `@playwright/test`). Rewrite
  all 3 solutions to use `AxeBuilder(...).analyze()` and assert on the violations array.
- Spot-check both tracks' `theory.md` afterward only if a code snippet needs to match
  new solutions — content quality there was already confirmed strong by the audit.

**Phase 6 — Visualization & remaining doc polish**
- Add 2 ASCII diagrams to README (style-matched to the existing 4D scorecard / XP
  ladder ASCII art): the learner user-journey (Today → Module [Read/Watch/Practice/
  Build] → Lab → Record, per `crucible/frontend/src/learn/LearnApp.tsx`), and a
  track/dependency map (13 tracks, tiers, explaining the duplicated `02_`/`09_`
  directory prefixes).
- Reconcile `TEST_INFRA.md` vs `TEST_READY.md` — both are now numerically consistent
  (68/13, since Phase 0 fixed `TEST_READY.md`), but they're still two separately-
  maintained "verification" snapshots with no indication which is canonical. Pick one,
  or date/label both.
- `PROJECT.md` — Code Layout section lists a `src/cli/` directory that doesn't exist;
  clap parsing lives directly in `src/main.rs`. Fix the listed tree.
- `architecture_notes.md` (lines ~187-190) — describes locator scoring as a
  point-deduction model (`Hardcoded sleeps → -40 pts`, etc.); the real engine
  (`src/feedback.rs:70-78`) assigns an absolute score per locator kind, and this doc
  also mislabels `getByTestId` as "+100" when it's actually 85. Fix to match Phase 1.1's
  corrected table.

---

## Verification checklist (none of this has been run yet this session)

Run in this order once Phase 1 code changes settle:
1. `cargo build --release` — confirm the `src/main.rs` Watch-command edit compiles.
2. `cargo test --all` — especially `tests/curriculum_manifest_tests.rs`, to confirm the
   `lings.toml` reordering didn't break anything.
3. `cargo clippy --all-targets -- -D warnings`
4. `cargo run -- audit` — validates the 4-file contract + hint counts across all drills;
   will still flag the 2 devsecops `hints.md` files until Phase 1.3 is finished.
5. `python -m pytest tests/test_micro_crucible_chaos.py`
6. Manually: `cherenkov-lings watch --track=playwright-ts` and
   `--track=maestro-mobile` from outside the repo root, to confirm both the reorder
   and the fallback fix.
7. Once Phase 2/3 land: start the Crucible (`crucible/start.bat` or
   `docker compose up`), open `http://localhost:8080`, exercise the 3 flagship tabs,
   confirm real `:8081` network requests (not fabricated timers) in devtools.

This is the same protocol `AI_AGENT_INSTRUCTIONS.md` already documents — follow it
before reporting any phase complete.

---

## Context this session already has (don't re-derive it)

An HTML audit artifact was published earlier this session covering all findings above
in more narrative form, with severity ranking — ask the user for the link
(`claude.ai/code/artifact/...`) if useful, it wasn't saved as a repo file. The approved
plan this handover is based on is otherwise fully reproduced above; nothing was
omitted or summarized away.
