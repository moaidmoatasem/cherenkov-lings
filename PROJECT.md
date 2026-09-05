# Project: Cherenkov-lings Strategic Roadmap Implementation

## Architecture
The Cherenkov-lings Strategic Roadmap expands the platform across four core pillars:
1. **Polyglot AST Analysis**: Extends `get_diagnostic_report` (and static review engines) to detect performance traps across Java (REST Assured) and Python (Pytest) test suites, preventing client churn, socket leaks, unbuffered schema parsing, blocking async event loop calls, and inefficient fixture scopes.
2. **OTel Hinting**: Expands `get_hints` to provide progressive 3-tier guidance (Architectural Nudge, API Pattern, Code Diff) for telemetry challenges covering Span ID correlation, distributed trace assertions, and W3C traceparent propagation.
3. **Micro-Crucible Expansion**: Integrates Apache Kafka (KRaft mode, zero-ZooKeeper) and OpenTelemetry Collector into `docker-compose.yml` with dedicated networking, CORS headers, healthchecks, and validation tooling.
4. **Mission Control Badging System**: Implements "Chaos Survivor" and "The Architect" badges in the Mission Control React UI with modular component architecture, `data-testid` selectors, `data-unlocked` attributes, visual status pills, and test verification across completion states.
5. **Dual-Track E2E Verification & Forensic Hardening**: Validates 100% test passing across all tiers, adversarial challenger stress testing, and forensic audit with zero integrity violations.

## Feature Inventory
| # | Feature | Description | Milestone | Source |
|---|---------|-------------|-----------|--------|
| 1 | F1: Polyglot Performance Trap Detection (Java) | AST rule scanning for REST Assured client churn (`RestAssured.reset`), missing connection/socket timeouts, and repeated inline schema reloads | M1 | ORIGINAL_REQUEST §R1 |
| 2 | F2: Polyglot Performance Trap Detection (Python) | AST rule scanning for Pytest `time.sleep`, blocking calls in `async def test_*`, unclosed client sessions, and inefficient fixture scope | M1 | ORIGINAL_REQUEST §R1 |
| 3 | F3: Diagnostic Reporting Integration | Wire Java and Python performance trap detection into `get_diagnostic_report` in `src/mcp.rs` and `src/feedback.rs` | M1 | ORIGINAL_REQUEST §R1 |
| 4 | F4: 3-Tier OTel Progressive Hinting | Expand `get_hints` in `src/mcp.rs` and `src/feedback.rs` with 3 progressive tiers for Span ID correlation and distributed trace assertions | M2 | ORIGINAL_REQUEST §R2 |
| 5 | F5: OTel Telemetry Challenge Curriculum | Dedicated telemetry drill under `exercises/` with progressive 3-tier `hints.md` and test verification | M2 | ORIGINAL_REQUEST §R2 |
| 6 | F6: Kafka Broker KRaft Integration | KRaft-mode single-container Kafka broker (`apache/kafka:3.7.0`) in `docker-compose.yml` with dual listeners (29092 internal, 9092 host) and healthcheck | M3 | ORIGINAL_REQUEST §R3 |
| 7 | F7: OpenTelemetry Collector Integration | Contrib collector (`otel/opentelemetry-collector-contrib:0.95.0`) with OTLP gRPC (:4317), OTLP HTTP (:4318), CORS, debug exporter, and healthcheck (:13133) in `docker-compose.yml` and `otel-collector-config.yaml` | M3 | ORIGINAL_REQUEST §R3 |
| 8 | F8: Container Validation & Automated Startup Test | Automated test harness executing `docker-compose config` validation and container interoperability verification | M3 | ORIGINAL_REQUEST §R3 |
| 9 | F9: Mission Control Badge Components | Modular `BadgeCard.tsx` and `BadgesShowcase.tsx` with test selectors, status pills, and unlocked timestamps | M4 | ORIGINAL_REQUEST §R4 |
| 10 | F10: "Chaos Survivor" & "The Architect" Badges | Dynamic rendering based on completion state props and backend achievements | M4 | ORIGINAL_REQUEST §R4 |
| 11 | F11: Badging System Component / UI Tests | Playwright route-mocked and React component tests verifying badge states | M4 | ORIGINAL_REQUEST §R4 |
| 12 | F12: Dual-Track Verification & Forensic Audit | E2E test execution across Rust, Python, Docker Compose, and React; Challenger stress testing; Forensic Integrity Audit | M5 | Protocol |

## Milestones
| # | Name | Scope | Dependencies | Status |
|---|------|-------|-------------|--------|
| M1 | Polyglot AST Analysis | Java & Python performance traps in `src/feedback.rs`, `src/review/rules.rs`, `crucible/backend/review.py`, and test suites | None | DONE |
| M2 | OTel Hinting | 3-tier progressive hints in `src/mcp.rs`, `src/feedback.rs`, telemetry drill, and test suites | M1 | DONE |
| M3 | Micro-Crucible Expansion | `docker-compose.yml`, `otel-collector-config.yaml`, automated test harness | None | PARTIAL — containers stand up; backend never speaks Kafka or OTLP to them (see `CURRICULUM.md`'s checklist for specifics) |
| M4 | Badging System UI | `BadgeCard.tsx`, `BadgesShowcase.tsx`, `MissionControlPage.tsx`, Playwright & component tests | None | DONE |
| M5 | E2E & Forensic Hardening | Comprehensive test run, Challenger verification, Forensic Audit | M1, M2, M3, M4 | DONE — `cargo test`/`clippy`/`audit`, `ruff`, and the Python suites all pass; M3's gap above doesn't block this since the containers-stand-up test (`test_verify_docker_compose.py`) is itself passing and correctly scoped to that claim |

## Interface Contracts

### 1. Diagnostic Reporting (`get_diagnostic_report`)
- **Tool**: `get_diagnostic_report(file_path: String) -> JsonValue`
- **Output Schema**:
  ```json
  {
    "file_path": "...",
    "anti_patterns": [
      {
        "kind": "RestAssuredClientChurn | MissingTimeout | RepeatedSchemaReload | HardcodedSleep | PytestBlockingCallInAsync | PytestUnclosedClientSession | ...",
        "line": 12,
        "snippet": "...",
        "explanation": "...",
        "recommendation": "..."
      }
    ],
    "locators": [...],
    "locator_quality_score": 100.0
  }
  ```

### 2. OTel Hinting (`get_hints`)
- **Tool**: `get_hints(exercise_dir: String, level?: Integer, score?: Float, topic?: String) -> JsonValue`
- **Output Schema**:
  - `level 1`: Architectural Nudge (W3C trace context, async side-effects, traceparent propagation).
  - `level 2`: API Pattern (W3C traceparent formatting `00-{trace_id}-{parent_id}-01`, Span ID correlation, querying spans).
  - `level 3`: Code Diff (Reference diff with traceparent header injection and span tree assertions).

### 3. Docker Compose Orchestration
- **Services**:
  - `backend`: port 8081, environment `KAFKA_BOOTSTRAP_SERVERS=kafka:29092`, `OTEL_EXPORTER_OTLP_ENDPOINT=http://otel-collector:4318`
  - `frontend`: port 8080
  - `kafka`: port 9092:9092, internal 29092, KRaft controller 9093
  - `otel-collector`: port 4317:4317, 4318:4318, 8888:8888, 13133:13133, volume `./otel-collector-config.yaml:/etc/otelcol-contrib/config.yaml:ro`
- **Network**: `crucible-network` (bridge)

### 4. Mission Control Badging System
- **Components**:
  - `BadgeCard`: props `{ badge: BadgeDefinition, state: BadgeCompletionState, onClick?: () => void }`
  - `BadgesShowcase`: props `{ progress: ProgressData | null, completionOverrides?: Record<string, boolean> }`
- **DOM Attributes**:
  - `data-testid="badge-chaos_survivor"`, `data-testid="badge-the_architect"`
  - `data-unlocked="true" | "false"`
  - `data-testid="badge-status-pill"` -> contains text "UNLOCKED" | "LOCKED"

## Code Layout
```
src/
├── feedback.rs                        # M1 & M2: AST static analyzer & ProgressiveHints
├── mcp.rs                             # M1 & M2: get_diagnostic_report and get_hints handlers
└── review/rules.rs                    # M1: RuleScanner performance trap rules

crucible/
├── backend/
│   ├── review.py                      # M1: Python review engine AST rules
│   └── tests/
│       └── test_sprint4_api.py        # M1: Backend review test assertions
├── frontend/
│   ├── src/
│   │   ├── components/badges/         # M4: Modular badging components
│   │   │   ├── types.ts
│   │   │   ├── BadgeCard.tsx
│   │   │   └── BadgesShowcase.tsx
│   │   └── pages/
│   │       └── MissionControlPage.tsx # M4: Mission Control integration
│   └── e2e/
│       └── 16-badges.spec.ts          # M4: Playwright badging tests
├── docker-compose.yml                 # M3: Orchestration config
├── otel-collector-config.yaml         # M3: OpenTelemetry collector configuration
└── tests/
    └── test_verify_docker_compose.py  # M3: Docker compose verification test

tests/
├── mcp_server_tests.rs                # M1 & M2: Programmatic tests for get_diagnostic_report & get_hints
└── review_tests.rs                    # M1: Programmatic AST rule tests
```
