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
