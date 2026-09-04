# Original User Request

## 2026-08-23T14:38:18Z

**cherenkov-lings** is an open-source, local-first, interactive learning platform for QA Engineers and SDETs that teaches modern test automation (frontend, backend, performance) through a Rustlings-style watcher loop, an embedded pathological sandbox (the Micro-Crucible), and AI-powered root-cause feedback — all with zero cloud, zero GPU, and zero vendor lock-in.

Working directory: C:\Users\moaid\Documents\antigravity\wonderful-raman

Integrity mode: development

---

## Context: What Has Already Been Built (Sprint 0 — Complete)

The Rust CLI skeleton is fully compiled and verified on `stable-x86_64-pc-windows-msvc`. The following files exist and are correct:

- `Cargo.toml` — deps: clap 4.5, tokio 1.37, notify 6.1.1, toml 0.8.23, serde 1.0
- `lings.toml` — platform config: tracks `playwright-ts`, `restassured-java`, `k6-js`
- `src/main.rs` — CLI: `init`, `watch --track=<id>`, `diagnose` commands
- `src/watcher.rs` — 50ms debounced file watcher using `notify` + `spawn_blocking` + `recv_timeout`
- `src/config.rs` — typed `lings.toml` deserializer
- `crucible/frontend/` — Vite/React scaffold (bare, needs content)
- `README.md` — project identity and persona targeting

`cargo run -- --help` produces:
```
Interactive Quality Engineering & SDET Learning Platform
Commands: init | watch | diagnose | help
```

---

## Requirements

### R1. Micro-Crucible Backend (FastAPI, Port 8081)

Build `crucible/backend/app.py` — a FastAPI server that serves as the intentionally broken, chaos-capable target application for all exercises. It must:

- Expose realistic API endpoints (e.g., `/checkout`, `/transfer`, `/search`) that exercises call against
- Parse in-band `X-Chaos` request headers to dynamically inject failure modes:
  - `X-Chaos: delay=<ms>` — inject artificial latency
  - `X-Chaos: stale_dom=true` — signal to the frontend to replace DOM nodes after render
  - `X-Chaos: token_expire=immediate` — invalidate JWT mid-session
  - `X-Chaos: kafka_lag=<ms>` — simulate async ledger update delay
- Include a `requirements.txt` and a startup script (`crucible/start.bat`) that installs deps and starts both the backend (port 8081) and serves the frontend (port 8080)

### R2. Micro-Crucible Frontend (React, Port 8080)

Build out `crucible/frontend/src/` with realistic UI pages that the Playwright exercises will automate against:

- `/checkout` — a checkout form with a React 19 hydration delay trap (button is rendered but not interactive until `data-hydrated="true"` is set)
- `/transfer` — a bank transfer form whose balance update is delayed (simulating Kafka lag) — balance only reflects after polling
- `/search` — a debounced autocomplete that returns out-of-order API responses
- The app must contain at least one closed Shadow DOM component and one cross-origin iframe placeholder

### R3. Exercise File Contract & First 3 Playwright Drills

Define and implement the standard exercise structure. Each drill lives at `exercises/01_web_playwright_ts/<NN>_<name>/` and contains:

- `exercise.ts` — the intentionally broken or incomplete starting code with a `// TODO:` comment marking the anti-pattern to fix
- `solution.ts` — the correct, flakiness-resistant reference implementation
- `hints.md` — exactly 3 progressive hints: (1) architectural nudge, (2) API pattern, (3) code diff

Implement the first **3 drills** end-to-end:
1. `01_hydration_timing` — fix click drops caused by React hydration delay (anti-pattern: `waitForTimeout`)
2. `02_shadow_dom_v2` — pierce a nested closed shadow root without XPath
3. `03_debounce_race_condition` — handle out-of-order autocomplete API responses

Each exercise must be runnable standalone with `npx playwright test exercise.ts` against the running Micro-Crucible.

### R4. Node.js IPC Worker & Rust Feedback Integration

Implement the pre-warmed Node.js worker that enables the sub-100ms feedback loop:

- `workers/node_worker.js` — a Node.js process that listens on a named pipe / IPC socket, receives a file path, runs `npx playwright test <file> --reporter=json`, and returns structured JSON results
- `src/runner.rs` — a Rust module that spawns the worker on startup and communicates with it over IPC to trigger test runs on file-save events
- Wire the watcher → runner → feedback output pipeline in `src/main.rs`'s `watch` command so that saving an exercise file prints real test results to the terminal

### R5. 4D Feedback Matrix (`src/feedback.rs`)

Implement the feedback scoring logic:

```
Total Score = (0.35 × Correctness) + (0.35 × Flakiness) + (0.15 × LocatorQuality) + (0.15 × Speed)
```

- **Correctness**: parse Playwright JSON reporter output for pass/fail
- **Flakiness Resistance**: run the test 5 consecutive times under injected chaos (use `X-Chaos: delay=200ms;jitter=75ms`) — score = passed_runs / 5 × 100; flag if `waitForTimeout` is detected in the exercise AST
- **Locator Quality**: static AST analysis of the exercise `.ts` file — score `getByRole` = 100, `getByTestId` = 85, CSS class = 40, absolute XPath = 0
- **Execution Speed**: wall-clock time vs. baseline

Print a clean, colored terminal scorecard after each run.

---

## Acceptance Criteria

### CLI runs cleanly
- [ ] `cargo build --release` succeeds with zero errors and zero warnings
- [ ] `cherenkov-lings init` prints a welcome message and creates the exercise directory structure
- [ ] `cherenkov-lings watch --track=playwright-ts` starts the watcher, detects file saves within 100ms, and triggers the feedback loop

### Micro-Crucible is live and functional
- [ ] `crucible/start.bat` (or equivalent) starts both the backend and frontend without errors
- [ ] `curl http://localhost:8081/checkout` returns a valid JSON response
- [ ] `curl -H "X-Chaos: delay=500ms" http://localhost:8081/checkout` takes >= 500ms to respond
- [ ] `http://localhost:8080/checkout` renders the hydration-trap checkout page in the browser

### Exercises run standalone
- [ ] `npx playwright test exercises/01_web_playwright_ts/01_hydration_timing/exercise.ts` FAILS (the anti-pattern version should fail against a running Crucible)
- [ ] `npx playwright test exercises/01_web_playwright_ts/01_hydration_timing/solution.ts` PASSES all 5 chaos stress runs
- [ ] Same pass/fail contract holds for drills 02 and 03

### End-to-end watcher loop works
- [ ] Saving `exercise.ts` (anti-pattern version) while `cherenkov-lings watch` is running produces a terminal scorecard showing < 85 total score with a flakiness failure message
- [ ] Saving `solution.ts` content into `exercise.ts` produces a scorecard >= 85 with a 5/5 flakiness pass

### Code quality
- [ ] All Rust code passes `cargo clippy -- -D warnings`
- [ ] All Python code passes `ruff check crucible/`
- [ ] All TypeScript exercises pass `tsc --noEmit`

## 2026-09-04T16:20:36Z

```markdown
# Teamwork Project: Market Analysis & Strategic Competitive Landscape

Conduct an exhaustive market analysis and competitive landscape evaluation for `cherenkov-lings`—the local-first, zero-cloud, chaos-driven experiential learning platform for QA Engineers and SDETs. Deliver a comprehensive strategic market report (`market_analysis.md`) covering competitive benchmarking, market sizing, target personas, differentiation vectors, and growth/adoption strategies.

Working directory: C:\Users\moaid\Documents\antigravity\wonderful-raman
Integrity mode: development

## Requirements

### R1. Competitive Landscape & Benchmarking Matrix
Analyze existing alternatives across four key market segments:
1. Open-source "-lings" and developer drills (Rustlings, Ziglings, Exercism, Gitlings).
2. Dedicated QA / Test Automation learning platforms (Test Automation University, Ministry of Testing, Guru99, Toolsqa).
3. Cloud-hosted interactive sandbox & lab platforms (Katacoda/Killercoda, KodeKloud, LeetCode, HackerRank).
4. Synthetic practice targets and demo sandboxes (The Internet / Herokuapp, SauceDemo, Restful-Booker).
Produce a feature-by-feature scoring matrix comparing execution model (local vs cloud), feedback speed, flakiness handling, curriculum depth (UI, API, Perf, Sec, GenAI), and total cost of ownership.

### R2. Value Proposition & Moat Analysis
Evaluate `cherenkov-lings`'s distinct technological and pedagogical moats:
- Local-first architecture (zero cloud cost, zero latency, zero vendor lock-in).
- 4D evaluation matrix (Correctness, Chaos Resilience, Locator Quality, Execution Speed).
- Integrated Micro-Crucible chaos target and L4/L7 proxy.
- Native Model Context Protocol (MCP) integration for AI-assisted IDE learning.
- Enterprise SDET capabilities (CI simulator, AI code review, Allure triage).

### R3. Market Sizing, Personas & Curriculum Gap Analysis
Define the addressable market (TAM / SAM / SOM) for QA/SDET upskilling. Profile 3 distinct learner personas:
1. Manual QA Engineer transitioning to Automation/SDET.
2. Mid-level Automation Engineer expanding into Performance, DevSecOps, and GenAI QA.
3. Enterprise QA Lead / Architect standardizing team practices and evaluating candidate competencies.
Map each persona's pain points against the current 11-track, 60-drill curriculum to identify uncovered market niches.

### R4. Strategic Recommendations & Monetization / Distribution Pathways
Provide actionable strategic recommendations covering:
- Open-core vs. community-driven distribution models.
- Enterprise adoption vectors (candidate screening, team upskilling benchmarks).
- Content expansion roadmap (e.g., gRPC, GraphQL, Cypress vs Playwright, Chaos Mesh).

## Acceptance Criteria

### Depth & Rigor
- [ ] `market_analysis.md` is generated in the workspace root with exhaustive quantitative and qualitative analysis.
- [ ] At least 10 named competitors across the 4 specified segments are benchmarked in a structured comparative matrix.
- [ ] All 11 tracks of `cherenkov-lings` are mapped against real-world industry demand and certification standards.

### Strategic Actionability
- [ ] Includes detailed SWOT analysis (Strengths, Weaknesses, Opportunities, Threats) specifically tailored to local-first learning.
- [ ] Defines concrete TAM/SAM/SOM estimates backed by software testing market data.
- [ ] Details 3 granular learner personas with concrete learning objectives and friction points.
- [ ] Outlines a 3-phase strategic roadmap for community adoption and enterprise credibility.
```

## 2026-09-04T17:03:26Z

```markdown
# Teamwork Project: Market Analysis Integrity & Sourcing Remediation

Revise `market_analysis.md` for `cherenkov-lings` to fix factual errors against the actual repo, add real sourcing (or explicit unsourced-estimate labeling) to every market-sizing and demographic claim, disclose the competitor scoring rubric, and correct the document's own description of what its "audit" verified. This is a remediation pass on the existing deliverable, not a rewrite from scratch — preserve the structure, personas, curriculum mapping, and SWOT/roadmap content; fix what's broken.

Working directory: C:\Users\moaid\Documents\antigravity\wonderful-raman
Integrity mode: development

## Requirements

### R1. Reconcile Every Repo-Derived Claim Against Actual Source
Every number in the document that claims to describe `cherenkov-lings` itself (drill count, track count, port numbers, scoring weights, test counts, file paths) must be re-verified against the current repo state, not against the previous draft's own assertions. Known issue to fix: the document states "68 drills across 13 tracks" — the actual count under `exercises/` (mindepth 2, maxdepth 2 directories) is 63. Either correct the number or, if 68 is deliberately counting something else (e.g. including planned-but-unshipped drills), say so explicitly in the text rather than presenting it as a shipped count.

### R2. Source or Explicitly Label Every Market/Demographic Figure
None of the following currently have a named, dated source anywhere in the document: total software testing market size ($52.4B/$89.2B), TAM/SAM/SOM dollar figures, "4.2M QA professionals globally," "45% still manual," the 7.9% and 12.8% CAGR figures, the Katacoda shutdown date/cause, and every competitor's stated pricing/TCO. For each such figure:
- If a real source exists (a named industry report, analyst firm, public company filing, or survey, with a year), cite it inline.
- If no real source is available, keep the figure but relabel it inline as an explicit estimate — e.g. "Illustrative estimate (unsourced) — not independently verified" — directly next to the number in the table or prose, not only in a general caveats footnote.
Do not present modeled/invented numbers with the same confidence and formatting as sourced ones.

### R3. Disclose the Competitive Scoring Rubric
The 14-competitor Dual-Index scoring table currently presents precise decimal scores (e.g. "25.0/40," "22.0/40") with no explanation of how a given competitor earned that number versus a neighboring one. Add an explicit rubric: what each sub-criterion within Index A and Index B measures, its point range, and — for each competitor — one sentence of concrete evidence backing its score on that sub-criterion (a stated feature, a publicly known price, a publicly known latency figure). Where no concrete evidence is available for a sub-criterion, mark that cell as an estimate rather than presenting an unexplained decimal.

### R4. Correct the Document's Self-Description of Its Own Verification
The document (and its supporting `.agents/handoff.md` / Victory Auditor trail) currently claims "100% Passed & Independently Verified" and "VICTORY CONFIRMED" in a way that reads as validating the market research itself. In fact, per the handoff record, the audit validated: (a) internal arithmetic consistency (SOM = SAM × stated %, weights summing to 1.0), and (b) alignment between a handful of document claims and this repo's own source code/test suite. It did not independently verify any external market datum. Rewrite the verification language in `market_analysis.md`'s own text (not just the internal audit artifacts) to state precisely what was checked, and stop claiming independent verification of figures that were never checked against an external source.

## Acceptance Criteria

### Factual accuracy
- [ ] All drill/track/port/weight/file-path claims in `market_analysis.md` match the current repo state exactly, verified by re-running the relevant check (e.g. counting `exercises/*/*/` directories, grepping `src/feedback.rs` weight constants) — not by re-reading the prior draft.

### Sourcing integrity
- [ ] Every dollar figure and demographic/growth-rate statistic in the R3 (market sizing) section either carries a named, dated citation or an inline "unsourced estimate" label visible at the point of use.
- [ ] Zero numbers in the document present an invented figure with the same visual/textual confidence as a cited one.

### Scoring transparency
- [ ] The competitor benchmarking section states its Index A/B rubric (sub-criteria, point ranges) directly in `market_analysis.md`.
- [ ] Each competitor's score carries at least one line of stated evidence per sub-criterion, or is marked as an estimate.

### Verification honesty
- [ ] The document's own conclusion/attestation section accurately describes the audit as validating internal consistency and repo alignment, not external market accuracy.
- [ ] A new short section ("Data Provenance") near the top classifies the report's major claim categories as one of: Verified-from-repo, Externally-sourced-and-cited, or Estimate-unsourced — so a reader can tell at a glance which numbers to trust for external use (e.g. a pitch deck) and which are directional only.
```
