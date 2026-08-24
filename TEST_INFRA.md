# E2E Test Infra: cherenkov-lings Sprint 3

## Test Philosophy
- Polyglot, opaque-box, and requirement-driven testing across Web (Playwright TS), API (REST Assured Java), Load (k6 JS), Mobile (Maestro YAML), and GenAI QA (Playwright TS).
- High-fidelity feedback matrix verification ensuring exact anti-pattern detection and SLA score computations.

## Feature Inventory
| # | Feature | Source | Tier 1 | Tier 2 | Tier 3 | Tier 4 |
|---|---------|--------|:------:|:------:|:------:|:------:|
| 1 | Micro-Crucible /api/rag | ORIGINAL_REQUEST §R3 | ✓ | ✓ | ✓ | ✓ |
| 2 | Micro-Crucible /api/llm | ORIGINAL_REQUEST §R3 | ✓ | ✓ | ✓ | ✓ |
| 3 | k6 Drills & Options | ORIGINAL_REQUEST §R1 | ✓ | ✓ | ✓ | ✓ |
| 4 | k6 Runner & Summary JSON Parser | ORIGINAL_REQUEST §R1 | ✓ | ✓ | ✓ | ✓ |
| 5 | Maestro Mobile Drills | ORIGINAL_REQUEST §R2 | ✓ | ✓ | ✓ | ✓ |
| 6 | Maestro Runner & YAML Anti-Patterns | ORIGINAL_REQUEST §R2 | ✓ | ✓ | ✓ | ✓ |
| 7 | GenAI QA Drills | ORIGINAL_REQUEST §R3 | ✓ | ✓ | ✓ | ✓ |
| 8 | CLI Watch & Diagnose Multi-Track | ORIGINAL_REQUEST | ✓ | ✓ | ✓ | ✓ |

## Test Architecture
- **Rust Integration Tests (`tests/`)**:
  - `tests/adversarial_matrix_tests.rs`: Tests 4D Feedback Matrix, locator scoring, flakiness penalty, and anti-patterns.
  - `tests/jvm_runner_test.rs`: Tests Surefire XML parsing, class mapping, and JVM runner lifecycle.
  - `tests/k6_runner_test.rs` / `tests/maestro_runner_test.rs`: Tests k6 JSON summary parser, Maestro YAML flow validation, and runner dispatch.
  - `tests/e2e_tier1_to_tier4_suite.rs`: End-to-end integration across all 5 tracks and CLI commands.
- **Python Pytest Suite (`tests/test_micro_crucible_chaos.py`)**:
  - Validates all Micro-Crucible endpoints including chaos directives and GenAI endpoints.
- **Playwright Test Suite (`exercises/01_web_playwright_ts/`, `exercises/05_genai_qa/`)**:
  - Validates drill pass/fail contracts.

## Coverage Goals
- `cargo test`: >= 145 passing tests (target 160+).
- `cargo clippy -- -D warnings`: 0 warnings.
- `cargo build --release`: 0 errors.
- `python -m ruff check crucible/`: 0 errors.
