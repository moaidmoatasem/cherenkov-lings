# Project: cherenkov-lings Sprint 4 — "The Enterprise SDET Simulator"

## Architecture
Sprint 4 elevates `cherenkov-lings` to an Enterprise SDET Simulator featuring:
1. **Virtual Senior QA & AI Mentor**: AST-based static rule linting (hardcoded sleeps, unwrap, missing assertions, fragile locators) combined with a local LLM client (Ollama/custom endpoint with deterministic offline mock fallback) and an interactive Socratic "Fix-It-Together" wizard.
2. **CI/CD Pipeline Simulator**: GitHub Actions workflow YAML parsing, strict enterprise SDET validation (mandatory matrix parallel strategies and test artifact uploads), and an interactive parallel mock runner.
3. **Enterprise Allure Chaos Reporting & Triage System**: Generation of Allure-compatible test execution telemetry and interactive HTML reports containing chaotic test runs (real defects, flaky infrastructure chaos, test automation anti-patterns) with an interactive root-cause triage hypothesis submission engine.
4. **Mission Control React Frontend**: Integrated tabs for Code Review, Drag-and-Drop CI Pipeline Builder, and Allure Test Reports & Triage Dashboard.
5. **Full API & CLI Integration**: Dedicated CLI subcommands (`review`, `pipeline run`, `triage`, `report`) and REST API endpoints.

## Feature Inventory
| # | Feature | Description | Milestone | Source |
|---|---------|-------------|-----------|--------|
| 1 | F1: AST Static Code Review Engine | Rule-based static AST analysis detecting anti-patterns (sleeps, fragile locators, missing assertions, raw unwraps) across TS, JS, Python, Java, Rust | M1 | Survey |
| 2 | F2: AI Senior QA Mentor | Configurable LLM client (Ollama / HTTP / deterministic offline mock) providing Socratic architectural critiques and refactoring guidance | M1 | Survey |
| 3 | F3: Interactive Fix-It-Together Flow | Interactive terminal flow offering progressive hints, unified diff previews, and one-click automated code patching | M1 | Survey |
| 4 | F4: GitHub Actions Workflow YAML Validator | Parser and strict SDET validator enforcing matrix parallelism (`strategy.matrix`) and artifact uploads (`actions/upload-artifact`) | M2 | Survey |
| 5 | F5: Mock CI Pipeline Runner Engine | Simulated parallel matrix execution engine with animated step progress, logs, and timing | M2 | Survey |
| 6 | F6: Enterprise Allure & Chaos Reporting Engine | Generator for Allure JSON test results and interactive HTML reports with chaotic test run telemetry | M3 | Survey |
| 7 | F7: Interactive Triage Hypothesis Engine | Root-cause triage submission challenge evaluating student hypotheses against failure taxonomy and awarding XP | M3 | Survey |
| 8 | F8: Server REST API Endpoints | Endpoints for review, pipeline validation/execution, reports, and triage (`/api/review`, `/api/pipeline/*`, `/api/triage/*`) | M4 | Survey |
| 9 | F9: React Code Review Tab | UI for AST rule gauge, violation cards, Senior QA mentor chat, and Fix-It-Together side-by-side diff applier | M5 | Survey |
| 10 | F10: React Drag-and-Drop CI Builder | Visual canvas for workflow stages/steps, 2-way YAML sync, SDET validation warnings, and simulated runner logs | M5 | Survey |
| 11 | F11: React Allure & Triage Dashboard | Allure donut charts, flaky test trends, chaos log correlation, and interactive triage hypothesis form | M5 | Survey |
| 12 | F12: E2E Verification & Forensic Hardening | Comprehensive test suite covering Tiers 1-5, regression protection for Sprints 1-3, Challenger verification, and Forensic Audit | M6 | Survey |

## Milestones
| # | Name | Scope | Dependencies | Status |
|---|------|-------|-------------|--------|
| M1 | R1: Review Engine & AI Mentor | `src/review/`, AST rules, LLM client/mock, Fix-It-Together flow, CLI command `cherenkov-lings review` | None | DONE |
| M2 | R2: CI/CD Pipeline Simulator | `src/pipeline/`, YAML parser, SDET validator, mock runner, CLI command `cherenkov-lings pipeline run` | None | DONE |
| M3 | R3: Allure Chaos Reporting & Triage | `src/reports/`, `src/triage/`, Allure JSON/HTML generation, chaotic test data, hypothesis submission engine | None | DONE |
| M4 | Backend API & Server Integration | Server endpoints in `crucible/backend/`, CLI integration in `src/main.rs` | M1, M2, M3 | DONE |
| M5 | Frontend React Mission Control Tabs | `crucible/frontend/src/pages/` (CodeReview, PipelineBuilder, AllureTriage), Navbar, App routing | M1, M2, M3 | DONE |
| M6 | Dual Track E2E Verification & Forensic Audit | Full test suite, Challenger stress tests, Forensic Auditor integrity verification | M1, M2, M3, M4, M5 | DONE |

## Interface Contracts

### 1. Review Engine (`src/review/`) ↔ CLI / API
- **Struct `AstViolation`**: `rule_id: String`, `severity: Severity` (Error, Warning, Info), `file_path: String`, `line_number: usize`, `message: String`, `code_snippet: String`, `suggested_fix: Option<String>`
- **Struct `ReviewReport`**: `exercise_name: String`, `score: u32`, `passed: bool`, `violations: Vec<AstViolation>`, `mentor_critique: String`, `socratic_questions: Vec<String>`, `suggested_diff: Option<String>`
- **Fn `run_review(file_path: &Path, config: &ReviewConfig) -> Result<ReviewReport>`**
- **Fn `apply_fix(file_path: &Path, fix_id: &str) -> Result<String>`**

### 2. Pipeline Simulator (`src/pipeline/`) ↔ CLI / API
- **Struct `PipelineValidation`**: `valid: bool`, `errors: Vec<PipelineError>`, `warnings: Vec<PipelineWarning>`, `matrix_detected: bool`, `artifact_upload_detected: bool`
- **Struct `PipelineRunResult`**: `workflow_name: String`, `jobs: Vec<JobRunResult>`, `duration_ms: u64`, `success: bool`, `logs: Vec<LogEntry>`
- **Fn `validate_workflow(yaml_content: &str) -> PipelineValidation`**
- **Fn `run_pipeline(yaml_path: &Path, opts: &PipelineRunOptions) -> Result<PipelineRunResult>`**

### 3. Allure & Triage (`src/reports/`, `src/triage/`) ↔ CLI / API
- **Struct `ChaosTestResult`**: `test_id: String`, `name: String`, `status: TestStatus` (Passed, Failed, Broken, Flaky), `duration_ms: u64`, `error_message: Option<String>`, `stack_trace: Option<String>`, `chaos_event: Option<ChaosEventTelemetry>`, `category: FailureCategory` (RealBug, FlakyInfra, AntiPattern)
- **Struct `TriageSubmission`**: `test_id: String`, `learner_category: FailureCategory`, `root_cause_explanation: String`, `suggested_fix: String`
- **Struct `TriageResult`**: `correct: bool`, `actual_category: FailureCategory`, `score_awarded: u32`, `feedback: String`, `badge_unlocked: Option<String>`
- **Fn `generate_chaos_allure_report(output_dir: &Path) -> Result<AllureReportSummary>`**
- **Fn `evaluate_triage(submission: &TriageSubmission) -> TriageResult`**

## Code Layout
```
src/
├── main.rs                    # CLI entrypoint: clap arg parsing and all subcommands
│                               #   (review, pipeline, triage, report, watch, audit, ...)
│                               #   live directly in this file -- there is no src/cli/
├── review/                    # R1: AST rules engine, LLM client/mock, Fix-It-Together flow
│   ├── mod.rs
│   ├── rules.rs               # AST lint rules (sleep, locator, unwrap, assertion)
│   ├── llm.rs                 # Ollama / OpenAI / Mock AI Mentor provider
│   └── interactive.rs         # Fix-It-Together terminal wizard
├── pipeline/                  # R2: CI/CD simulator
│   ├── mod.rs
│   ├── parser.rs              # GitHub Actions YAML parser
│   ├── validator.rs           # Strict SDET rules (matrix, artifact uploads)
│   └── runner.rs              # Mock parallel matrix execution engine
├── reports/                   # R3: Allure & chaos reports
│   ├── mod.rs
│   ├── allure.rs              # Allure JSON / HTML report generator
│   └── chaos_dataset.rs       # 70+ chaotic test telemetry generator
└── triage/                    # R3: Interactive triage challenge
    ├── mod.rs
    ├── evaluator.rs           # Hypothesis scoring & XP/badge rewards
    └── interactive.rs         # Terminal triage challenge flow

crucible/
├── backend/                   # FastAPI backend server
│   ├── main.py                # REST endpoints for review, pipeline, reports, triage
│   ├── review.py              # AST review logic & diff generator
│   ├── pipeline.py            # Workflow validator & matrix simulator
│   ├── reports.py             # Allure HTML generator & chaos dataset
│   ├── triage.py              # Root-cause evaluator & XP engine
│   ├── models.py              # Pydantic schemas for Sprint 4
│   └── tests/
│       └── test_sprint4_api.py # 17/17 passing tests
└── frontend/                  # React Vite Mission Control
    └── src/
        ├── pages/
        │   ├── CodeReviewPage.tsx       # Code Review & Fix-It-Together tab
        │   ├── PipelineBuilderPage.tsx  # Drag-and-Drop CI workflow builder
        │   └── AllureTriagePage.tsx     # Allure chaos reports & triage dashboard
        ├── components/
        │   └── Navbar.tsx               # Updated navigation items
        └── App.tsx                      # Updated route switchboard
```
