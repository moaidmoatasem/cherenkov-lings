# Project: cherenkov-lings Sprint 3

## Architecture
- **Layer 4/7 Chaos Proxy (`src/proxy.rs`)**: High-performance reverse proxy listening on `127.0.0.1:8086`, routing to Micro-Crucible backend on `127.0.0.1:8081`. Supports raw TCP drops, latency jitter, 502/504 gateway errors, and chaos header passthrough.
- **Polyglot Runner Engine (`src/runner.rs`)**:
  - `NodeRunner`: Subprocess NDJSON IPC over stdio to `workers/node_worker.js` for Playwright Web TS and GenAI QA TS.
  - `JvmRunner`: Subprocess runner executing `mvn test -B -Dtest={class}` on `exercises/02_api_restassured_java/`.
  - `K6Runner`: High-concurrency load testing runner invoking `k6 run --summary-export=summary.json` and parsing JSON metrics & thresholds.
  - `MaestroRunner`: Mobile UI automation definition validator checking YAML flow syntax and structure.
  - `AnyRunner`: Unified enum dispatching across all 4 runner engines.
- **4D Feedback Matrix Engine (`src/feedback.rs`)**: Evaluates Correctness (35%), Flakiness (35%), Locator Quality (15%), Speed (15%). Analyzes TypeScript, Java, and YAML ASTs for anti-patterns (`waitForTimeout`, `Thread.sleep`, `MissingWhenCondition`, `MissingColdStartDeepLink`, `MissingActivityRecreation`).
- **Micro-Crucible Backend (`crucible/backend/`)**: FastAPI server on port 8081 providing mock e-commerce, banking, chaos directives, and GenAI mock endpoints (`/api/rag`, `/api/llm`).

## Code Layout
- `src/proxy.rs`: Chaos proxy implementation, configuration, and background supervisor.
- `src/main.rs`: CLI commands (`proxy`, `watch`, `diagnose`, `init`) and multi-track dispatch.
- `src/runner.rs`: `NodeRunner`, `JvmRunner`, `K6Runner`, `MaestroRunner`, and `AnyRunner`.
- `src/feedback.rs`: 4D Feedback Matrix, AST anti-pattern analysis for TS, Java, and YAML.
- `src/config.rs`: `lings.toml` configuration parsing.
- `crucible/backend/app.py`, `models.py`, `chaos.py`: Micro-Crucible backend endpoints.
- `exercises/01_web_playwright_ts/`: 3 Web UI drills.
- `exercises/02_api_restassured_java/`: 3 REST Assured Java drills.
- `exercises/03_mobile_maestro/`: 3 Mobile YAML drills + `maestro_runner.sh`.
- `exercises/04_perf_k6_js/`: 3 k6 Load Testing drills + `k6_runner.js`.
- `exercises/05_genai_qa/`: 2 GenAI QA Playwright TS drills.

## Feature Inventory
| # | Feature | Description | Milestone | Source | Status |
|---|---------|-------------|-----------|--------|:------:|
| 1 | L4/L7 Chaos Proxy Core | Reverse proxy on 8086, routing to 8081 | Baseline | ORIGINAL_REQUEST | DONE |
| 2 | Layer 4/7 Fault Injection | TCP drops, latency jitter, 502/504 errors | Baseline | ORIGINAL_REQUEST | DONE |
| 3 | Java Track & JVM Runner | Maven REST Assured track and Surefire XML parser | Baseline | ORIGINAL_REQUEST | DONE |
| 4 | Micro-Crucible GenAI Endpoints | `/api/rag` and `/api/llm` in FastAPI with Pydantic schemas | M1 | ORIGINAL_REQUEST §R3 | DONE |
| 5 | k6 Load Testing Drills | 3 drills in `exercises/04_perf_k6_js/` (pool starvation, spike p99, chaos SLA) | M2 | ORIGINAL_REQUEST §R1 | DONE |
| 6 | k6 Runner Engine & JSON Parser | `K6Runner` in `src/runner.rs` parsing `summary.json` for 4D matrix | M2 | ORIGINAL_REQUEST §R1 | DONE |
| 7 | Maestro Mobile Track Drills | 3 drills in `exercises/03_mobile_maestro/` (biometric fallback, deep link, orientation) | M3 | ORIGINAL_REQUEST §R2 | DONE |
| 8 | Maestro Runner & YAML Anti-Pattern AST | `MaestroRunner` in `src/runner.rs`, YAML anti-pattern detection in `src/feedback.rs` | M3 | ORIGINAL_REQUEST §R2 | DONE |
| 9 | GenAI QA Track Drills & Runner | 2 drills in `exercises/05_genai_qa/` (RAG faithfulness, LLM flakiness) | M4 | ORIGINAL_REQUEST §R3 | DONE |
| 10 | CLI Multi-Track Watcher & Diagnose | Dispatching all 5 tracks in `src/main.rs` | M2, M3, M4 | ORIGINAL_REQUEST | DONE |
| 11 | Comprehensive E2E Verification & Audit | >= 145 Rust tests, Clippy 0 warnings, Release build, Ruff, drills pass/fail | M5 | Acceptance Criteria | DONE |

## Milestones
| # | Name | Scope | Dependencies | Status | Key Outputs |
|---|------|-------|-------------|:------:|-------------|
| M1 | Crucible Backend GenAI Endpoints | `crucible/backend/app.py`, `models.py`, `tests/` | none | DONE | `/api/rag`, `/api/llm`, Pydantic schemas, 17 pytest tests |
| M2 | k6 Load Testing Track & Rust Runner | `exercises/04_perf_k6_js/`, `src/runner.rs`, `src/main.rs` | none | DONE | `k6_runner.js`, 3 drills, `K6Runner`, `parse_k6_summary_json` |
| M3 | Maestro Mobile Track & YAML Anti-Patterns | `exercises/03_mobile_maestro/`, `src/feedback.rs`, `src/runner.rs`, `src/main.rs` | none | DONE | `maestro_runner.sh`, 3 drills, `MaestroRunner`, YAML AST rules |
| M4 | GenAI QA Track & Playwright Integration | `exercises/05_genai_qa/`, `crucible/backend/` | M1 | DONE | 2 GenAI drills, Playwright config, pass/fail validation |
| M5 | Full Verification & E2E Acceptance | Rust test suite, Clippy, release build, Ruff, Forensic Audit | M1, M2, M3, M4 | DONE | 250 passing tests (>>145), 0 warnings, CLEAN audit |

## Interface Contracts
### k6 Runner ↔ Summary JSON (`src/runner.rs`)
- Command: `k6 run --summary-export=summary.json <file.js>`
- Metric threshold parsing: inspects `metrics.*.thresholds.*.ok`. If any `ok == false`, mark drill iteration failed.
- Speed/Duration: extracts `http_req_duration.values.avg` or `p(95)` for speed score.

### Maestro YAML AST ↔ Feedback Matrix (`src/feedback.rs`)
- Input: YAML source file (`.yaml` / `.yml`)
- Anti-patterns detected:
  - `MissingWhenCondition`: `Biometric` action without conditional `when:` / `runFlow` fallback.
  - `MissingColdStartDeepLink`: `openLink` without cold start `launchApp` arguments.
  - `MissingActivityRecreation`: Missing orientation change and re-assertion.
- Penalty: Caps flakiness score at 40.0 pts.

### Micro-Crucible GenAI Endpoints (`crucible/backend/`)
- `GET /api/rag?query={q}` -> `RagResponse { query, answer, source_facts, grounded, document_title }`
- `GET /api/llm?prompt={p}` -> `LlmResponse { prompt, intent, entities: { action, status, domain }, confidence, raw_text, model }`
