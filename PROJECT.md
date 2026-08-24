# Project: Cherenkov-Lings QA Learning Engine Expansion

## Architecture
Cherenkov-Lings is a high-performance interactive CLI daemon written in Rust (2024 edition) that guides QA engineers to SDET mastery across 9 tracks. It integrates with a sandboxed FastAPI backend (Micro-Crucible on port 8081), an L4/L7 Chaos Proxy (port 8086), AST static analysis, multi-language runners, and a gamified progress tracking engine.

## Feature Inventory
Every requirement from ORIGINAL_REQUEST.md is inventoried below with its assigned milestone and completion status.

| # | Feature | Description | Milestone | Status | Source |
|---|---------|-------------|-----------|--------|--------|
| 1 | R4.1 `POST /upload` Endpoint | Multipart upload with `X-Chaos: drop_partial=true` simulation | M1 | DONE | ORIGINAL_REQUEST §R4 |
| 2 | R4.2 `GET /products` Pagination | Paginated catalog returning `total`, `page`, `per_page`, `total_pages`, `products` | M1 | DONE | ORIGINAL_REQUEST §R4 |
| 3 | R4.3 `GET /events/stream` SSE | Server-Sent Events stream at 1 evt/s with `X-Chaos: drop_after={n}` connection cutoff | M1 | DONE | ORIGINAL_REQUEST §R4 |
| 4 | R4.4 `POST /graphql` Endpoint | Minimal zero-dependency GraphQL query & field alias handler | M1 | DONE | ORIGINAL_REQUEST §R4 |
| 5 | R4.5 Crucible Pytest & Ruff | 10 new pytest tests in `tests/test_micro_crucible_chaos.py` (27 total), 0 ruff errors | M1 | DONE | ORIGINAL_REQUEST §R4 |
| 6 | R1.1 Playwright TS Drills 06–10 | POM, iframe cross-origin, network intercept, visual regression, parallel state isolation | M2 | DONE | ORIGINAL_REQUEST §R1 |
| 7 | R1.2 REST Assured Java Drills 04–07 | Pagination boundary loop, JSON schema validator, GraphQL assertions, RequestSpec reuse | M2 | DONE | ORIGINAL_REQUEST §R1 |
| 8 | R1.3 Maestro Mobile Drills 04–05 | Dynamic scroll-to-element, conditional push notification permission flow | M2 | DONE | ORIGINAL_REQUEST §R1 |
| 9 | R1.4 k6 JS Drills 04–05 | Streaming SSE test against `/events/stream`, InfluxDB/Grafana metrics & thresholds | M2 | DONE | ORIGINAL_REQUEST §R1 |
| 10 | R1.5 JMeter Drills 01–08 | Complete exercise.jmx + solution.jmx/solution.sh + hints for all 8 JMeter drills | M2 | DONE | ORIGINAL_REQUEST §R1 |
| 11 | R1.6 Tool Decisions Drills 03–04 | Appium vs Maestro and Pact contract vs E2E test matrices | M2 | DONE | ORIGINAL_REQUEST §R1 |
| 12 | R1.7 Production Stories | Comment block with named real-world incident at top of all 60 exercise files | M2 | DONE | ORIGINAL_REQUEST §R1 |
| 13 | R3.1 Theoretical Context Modules | `theory.md` for all 60 drills (>=150 words, real incident, mechanism, ASCII diagram, crucible anchor) | M3 | DONE | ORIGINAL_REQUEST §R3 |
| 14 | R6.1 JMeterRunner Implementation | Runner trait implementation in `src/runner.rs`, executes `jmeter -n -t ... -l results.jtl` | M4 | DONE | ORIGINAL_REQUEST §R6 |
| 15 | R6.2 JTL CSV Parser | Parses `elapsed`, `success`, `label` from JTL CSV for p99, avg latency, error rate | M4 | DONE | ORIGINAL_REQUEST §R6 |
| 16 | R6.3 JMeter PATH Graceful Handling | Detects missing `jmeter` binary without panicking, provides helpful install message | M4 | DONE | ORIGINAL_REQUEST §R6 |
| 17 | R6.4 AnyRunner::Jmeter Variant | Adds `AnyRunner::Jmeter` variant, wires `lings.toml` runner mapping | M4 | DONE | ORIGINAL_REQUEST §R6 |
| 18 | R6.5 Clippy Baseline Cleanup | Resolves `PytestRunner` missing `Default` and redundant closure warnings (0 warnings) | M4 | DONE | ORIGINAL_REQUEST §R6 |
| 19 | R2.1 Gamification State & Storage | `.cherenkov-progress.json` schema & serialization in `src/gamification.rs` | M5 | DONE | ORIGINAL_REQUEST §R2 |
| 20 | R2.2 XP Calculation & Tiers | `base_xp * (total_score / 100) * tier_multiplier` (Tier 1: 1.0x, Tier 2: 1.5x, Tier 3: 2.0x) | M5 | DONE | ORIGINAL_REQUEST §R2 |
| 21 | R2.3 Level Progression (7 Ranks) | Trainee (0), Junior (500), Mid (1500), Senior (3000), Lead (6000), Architect (10000), SDET Master (20000) | M5 | DONE | ORIGINAL_REQUEST §R2 |
| 22 | R2.4 8 Achievements / Badges | first_blood, flakiness_slayer, chaos_survivor, tool_polyglot, the_architect, perfect_locator, speed_demon, sdet_master | M5 | DONE | ORIGINAL_REQUEST §R2 |
| 23 | R2.5 Terminal Scorecard & Reveals | Scorecard displays XP earned, ASCII progress bar, and multi-line ASCII badge reveal | M5 | DONE | ORIGINAL_REQUEST §R2 |
| 24 | R5.1 Dashboard CLI Subcommand | `cherenkov-lings dashboard` in `src/main.rs` rendering ANSI stats from progress file | M6 | DONE | ORIGINAL_REQUEST §R5 |
| 25 | R5.2 Dashboard ANSI Visuals | Header, level progress bar, curriculum progress table, top 3 badges, streak, next recommendation | M6 | DONE | ORIGINAL_REQUEST §R5 |
| 26 | R.Final All Verification Gates Pass | cargo test, clippy, pytest, ruff, playwright, dashboard | M7 | DONE | ORIGINAL_REQUEST §Verification |

## Milestones
| # | Name | Scope | Dependencies | Status |
|---|------|-------|-------------|--------|
| M1 | Crucible Backend Expansion | Implement `/upload`, `/products`, `/events/stream`, `/graphql` in `crucible/backend/app.py`, update `chaos.py` and `models.py`, write 10 pytest tests, verify ruff | None | DONE |
| M2 | Drill Curriculum Expansion | Author new drills for Playwright (06-10), REST Assured (04-07), Maestro (04-05), k6 (04-05), JMeter (01-08), Tool Decisions (03-04), update `lings.toml` | M1 | DONE |
| M3 | Theoretical Context Modules | Author `theory.md` for all 60 drill directories with named incidents, mechanisms, ASCII diagrams, and crucible anchors | None | DONE |
| M4 | JMeter Runner & Clippy Cleanup | Implement `JMeterRunner` in `src/runner.rs`, CSV JTL parser, `AnyRunner::Jmeter`, graceful PATH fallback, fix clippy warnings | None | DONE |
| M5 | Gamification Engine | Implement `src/gamification.rs`, `.cherenkov-progress.json` persistence, XP formula, 7 levels, 8 badges, streak logic, terminal badge reveal | None | DONE |
| M6 | Interactive Dashboard Subcommand | Implement `cherenkov-lings dashboard` in `src/main.rs`, ANSI scorecard rendering, track progress table, recommendation | M5 | DONE |
| M7 | E2E Integration & Verification Gates | Run and verify all 6 gate commands: `cargo test`, `cargo clippy -- -D warnings`, `python -m pytest tests/test_micro_crucible_chaos.py`, `ruff check crucible/`, `npx playwright test exercises/05_genai_qa/`, `cargo run -- dashboard` | M1, M2, M3, M4, M5, M6 | DONE |

## Code Layout
- `src/main.rs`: CLI commands (`watch`, `diagnose`, `init`, `proxy`, `mcp`, `dashboard`)
- `src/lib.rs`: Library exports (`config`, `feedback`, `gamification`, `proxy`, `runner`, `watcher`)
- `src/runner.rs`: Test runners (`NodeRunner`, `JvmRunner`, `K6Runner`, `MaestroRunner`, `PytestRunner`, `JMeterRunner`, `AnyRunner`)
- `src/feedback.rs`: 4D Feedback Matrix & AST analyzer
- `src/gamification.rs`: Gamification engine, XP, badges, progress storage, scorecard rendering, dashboard renderer
- `crucible/backend/app.py`: FastAPI backend routes & GraphQL resolver
- `crucible/backend/chaos.py`: Chaos headers & middleware
- `crucible/backend/models.py`: Pydantic models
- `tests/test_micro_crucible_chaos.py`: Pytest chaos integration suite (33 tests)
- `exercises/`: 60 curriculum directories across 11 tracks
- `lings.toml`: Curriculum and runner configuration
