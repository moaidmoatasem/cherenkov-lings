# Cherenkov-lings: A Comprehensive Curriculum Analysis and Engineering Guide

## 1. The Experiential Pedagogy: Engineering the "Micro-Crucible"

Modern SDET training has suffered from an over-reliance on passive documentation and "happy-path" tutorials that fail to prepare engineers for the entropic reality of production. The strategic shift pioneered by Cherenkov-lings replaces these abstractions with the "experiential breaking" of systems. By utilizing a local-first architecture, the platform strips away cloud-provider noise and vendor lock-in, forcing the engineer into an immediate, high-fidelity interaction with the system's runtime and protocol layers. Mastery is not granted; it is forged through the systematic resolution of failure.

### The Micro-Crucible Sandbox

The core of this curriculum is the "Micro-Crucible," a purposely pathological sandbox designed to simulate the volatile failure modes of distributed architectures. Rather than testing static "Hello World" apps, learners must confront:

* **React Hydration Timing:** Reproducing and solving intermittent click drops during the client-side hydration window.
* **Distributed Systems Lag:** Managing Kafka eventual consistency delays and out-of-order debounced search clobbering.
* **Encapsulation Barriers:** Automating across closed Shadow DOM boundaries.
* **Concurrency Issues:** Inducing and identifying database connection pool starvation under load.

### The 4D Feedback Matrix

The transition from a standard automation scripter to a Principal-level engineer is enforced by the 4D Feedback Matrix. Powered by a sub-50ms Rust watcher and a built-in MCP (Model Context Protocol) server, the platform provides instantaneous, protocol-level diagnostics. Every submission is evaluated against:

1. **Correctness:** Strict assertion verification of the intended state.
2. **Flakiness Guard:** Validating code resilience through 5/5 consecutive passes under injected network chaos.
3. **Locator Quality:** Utilizing AST anti-pattern analysis to enforce semantic accessibility (e.g., getByRole) over brittle CSS/XPath selectors.
4. **Speed:** Benchmarking execution against enterprise-grade sub-second baselines.

This 4D loop transforms the student's habits, moving them away from "making it pass" toward "architecting it to survive."

## 2. Core Technical Foundations (Tracks 0 – 0b)

A robust foundation in Python and the Pytest ecosystem is the prerequisite for managing the complexities of modern distributed systems. This stage focuses on clean-code standards, ensuring that automation suites remain maintainable and documentation-centric.

### Foundation Track Analysis

| Track Name | Tech Stack | Drill Count | Core Competencies |
| :--- | :--- | :--- | :--- |
| **0. Foundations** | Python / Pytest | 5 | AAA Pattern, Assertions as Documentation, Single-Responsibility |
| **0a. Advanced Pytest** | Python / Pytest | 3 | Fixture Scoping, Plugin Architecture, Parameterization |
| **0b. API Validation** | Python / Requests | 5 | JSON Schema Contracts, Status Code Semantics, Error Handling |

### Critical Takeaways

* **The AAA Pattern:** Strict adherence to Arrange-Act-Assert ensures tests serve as executable specifications.
* **Avoiding Mock Traps:** Drills emphasize testing against the runtime rather than over-mocking, which frequently masks integration failures. This groundwork is vital for moving into the "pathological" environments of the later tracks.

## 3. Advanced Automation & Resilience (Tracks 1 – 3)

In modern, asynchronous environments, UI and API testing must account for platform-specific quirks and network volatility.

### Track 1: Web UI (Playwright TypeScript)

The 10 drills in this track focus on the "flaky" reality of modern frontends. Engineers must master:

* **Hydration Race Conditions:** Solving click drops during complex client-side renders.
* **Enterprise Web Patterns:** Testing cross-origin payment iframes, closed Shadow DOM piercing, and worker storageState isolation for authenticated sessions.
* **Visual Integrity:** Implementing visual regression thresholds and network request intercepts to stabilize non-deterministic UI states.

### Track 2: API Resilience (REST Assured Java)

These 7 drills move beyond basic GET/POST requests to focus on consistency and distributed state:

* **Resilience Patterns:** Handling HTTP 409 idempotency collisions and transparent JWT 401 interceptors.
* **Distributed State:** Managing Kafka lag polling and multi-page pagination loops.
* **GraphQL Mastery:** Validating aliased GraphQL queries and implementing RequestSpecBuilder for scalable reuse.

### Track 3: Mobile UI (Maestro YAML)

Focusing on the unique constraints of mobile platforms across 5 drills:

* **OS-Level Interactions:** Handling biometrics (FaceID/TouchID) fallback flows and OS permission dialogs.
* **Lifecycle Management:** Testing activity recreation and screen rotation to ensure UI state persistence.
* **Dynamic Navigation:** Utilizing scrollUntilVisible for indeterminate lists and testing cold-start deep linking.

## 4. Performance Engineering & Distributed Load (Tracks 4 – 5)

High-scale enterprise systems treat performance as a functional requirement. Identifying "Database pool starvation" before it reaches production is the hallmark of a Senior SDET.

### The k6 vs. JMeter Duality

The curriculum balances developer-centric performance-as-code with traditional enterprise load testing:

* **Track 4: k6 (JavaScript)**
  * *Focus:* p99 spikes, Server-Sent Events (SSE) continuous streams, and SLA threshold monitoring via Chaos Proxy.
  * *Strategic Goal:* Real-time latency profiling and developer-integrated performance gates.
* **Track 5: JMeter (JMX)**
  * *Focus:* Non-GUI CI execution, CSRF extraction & correlation, and listener memory leak detection.
  * *Strategic Goal:* Mastering distributed load through master-agent configurations and automated JTL HTML reporting.

## 5. Specialist Domains: GenAI, DevSecOps, and Accessibility (Tracks 6, 7, 10)

The modern SDET profile requires expertise in non-functional domains, ensuring reliability in AI, cloud security, and inclusive design.

### Track 6: GenAI QA & LLMs (5 Drills)

As LLMs integrate into products, engineers must validate non-deterministic outputs:

* **RAG Faithfulness:** Ensuring model responses are grounded in provided citations.
* **Latency Metrics:** Monitoring TTFT (Time to First Token) streaming latency thresholds.
* **Security:** Red-teaming via prompt injection defenses and structured intent assertions.

### Track 7: Cloud DevSecOps (5 Drills)

This track addresses the security of the pipeline and runtime:

* **Infrastructure Hardening:** Identifying Docker socket privilege escalation and cloud metadata SSRF blocking.
* **Application Security:** Validating blind timing SQLi vulnerabilities and JWT signature bypass attempts.

### Track 10: Accessibility (A11y) (3 Drills)

Using Playwright and Axe to enforce WCAG standards:

* **Inclusive Design:** Detecting keyboard sequential focus traps and verifying ARIA screen-reader live regions.

## 6. Strategy, CI/CD, and Contract Testing (Tracks 8 – 9)

An SDET Architect must decide which tools and gates provide the highest ROI for a given architecture.

### Decision Matrices (Track 8)

Through 4 strategy-focused drills, engineers evaluate:

* **Tooling Trade-offs:** k6 vs. JMeter for enterprise load; Appium vs. Maestro for mobile; Pact vs. E2E for microservice boundaries.
* **Layer Selection:** Utilizing the UI vs. API layer decision matrix to optimize the test pyramid.

### Pipeline & Contracts (Track 9)

Focused on Pact (Python) to prevent breaking changes in microservice ecosystems:

* **Automated Provider Validation Gates:** Ensuring consumer schema definitions are honored before deployment.
* **Schema Evolution:** Managing the impact of additive vs. destructive API changes.

## 7. Theory & Real-World Alignment: The Enterprise Connection

The pedagogical value of Cherenkov-lings is grounded in its theory.md framework. Every drill directory contains a post-mortem analysis mapping the code to historical industry failures.

By dissecting the root causes of major outages at Stripe, Shopify, Spotify, GitHub, and Robinhood, students move beyond "passing tests." Understanding why a "Knight Capital-style" deployment disaster or an "Air Canada-level" glitch occurs provides the architectural context needed to prevent similar outcomes. Every theory module provides a "Simulate this in the Crucible" bridge, connecting historical protocol failures to hands-on mitigation.

## 8. Cognitive Learning Paths & Gamification

To maintain engagement through the 60-drill curriculum, the platform uses a structured progression map and an XP-based rank system.

### Progression Map

```text
  🌱 Trainee (0 XP) ──► 🔍 Junior QA (500 XP) ──► ⚡ Mid QA (1,500 XP)
                             │
                             ▼
  🔥 Senior QA (3,000 XP) ──► 🎯 Lead QA (6,000 XP) ──► 🏗️ QA Architect (10,000 XP)
                                                                 │
                                                                 ▼
                                                    ⚛️ SDET Master (20,000 XP)
```

### Specialist Badges

Engineers earn 8 distinct badges, including Flakiness Slayer, Chaos Survivor, and The Architect. These represent specific technical proficiencies (e.g., mastering network intercepts or solving concurrency locks) that signal a high level of systems-engineering maturity.

## 9. Future Expansions: The Roadmap

The Cherenkov-lings platform is an evolving ecosystem designed to stay ahead of the architectural curve. Our roadmap for future modules includes:

1. **Database & State Validation:** Deep-dive into data integrity and stateful testing across SQL/NoSQL boundaries.
2. **gRPC & Protobuf:** Mastering high-performance, contract-first communication protocols.
3. **System Telemetry & OpenTelemetry:** Bridging the gap between active testing and production observability.
4. **The Lings SDK:** A framework enabling the community to contribute their own pathological drills.

This curriculum does not merely teach tools; it uses the Lings SDK philosophy to architect quality into the very fabric of modern systems, preparing engineers to handle the high-stakes challenges of tomorrow's software.

---

# Cherenkov-lings Strategic Roadmap: Technical Curriculum Expansion & Learning Path Revision

## 1. Evolution of the Experiential Learning Model

The strategic shift for Cherenkov-lings necessitates a transition from traditional, passive test automation training to "hardcore systems engineering." While our current 60-drill baseline provides a foundation, it is insufficient for the demands of modern distributed systems. To achieve engineering mastery, we must force learners to confront the non-deterministic nature of production environments—race conditions, resource starvation, and partial failures—where simple "green-light" scripting fails. By expanding beyond the Micro-Crucible's existing UI-centric failures, we are moving toward a model where the test is an instrument for systems analysis. The current Micro-Crucible approach has proven effective at simulating failures; however, the new 3-Tier Execution Framework is required to formalize the transition from syntactic fluency to architectural rigor.

### The 3-Tier Execution Framework

| Tier | Technical Focus | Student Feedback Loop |
| :--- | :--- | :--- |
| **Tier 1: Local IDE-Watcher Loop** | Real-time AST evaluation and deterministic execution using the Rust-based watcher. | Sub-50ms hot-reload loop; immediate terminal/MCP feedback upon file save to minimize context switching. |
| **Tier 2: Real-World Outage Reconstruction** | Matching architectural theory (from theory.md) to live code failures within the Crucible. | Learners must map code failures to case studies like Stripe (idempotency) or Knight Capital (deployment risk). |
| **Tier 3: Pathological Mutation & Anti-pattern Safeguards** | Verification of test resilience against injected network chaos and AST-level anti-pattern analysis. | 4D Matrix scoring: evaluating flakiness under Chaos Proxy injection and identifying brittle locator patterns. |

This framework ensures that every fix is rooted in a deep understanding of the underlying system internals, providing the necessary rigor for our advanced technical tracks.

## 2. Advanced Technical Tracks: Curriculum Enhancements & Gaps

To bridge the chasm between "Web UI Automation" and "Enterprise Infrastructure Engineering," we are expanding the curriculum into stateful, asynchronous, and observability-driven testing. Modern SDETs must validate more than just the DOM; they must ensure system integrity across persistence layers and message brokers. These tracks address the "silent killers" of enterprise systems—state leakage, eventual consistency lag, and trace fragmentation.

### Module Deep-Dives

* **Database State & Isolation:** Drills focusing on PostgreSQL and Redis transaction rollbacks. Learners must implement teardown logic that ensures environment idempotency, preventing state leakage between concurrent test executions.
* **gRPC & Kafka Testing:** Supporting Protobuf schema validation and mocking. Drills focus on Kafka eventual consistency lag, requiring learners to implement deterministic polling loops that account for partition offsets and consumer group rebalancing.
* **Observability & Telemetry:** Hardcore engineering assertions against OpenTelemetry traces and Span IDs. The challenge involves correlating logs across distributed service boundaries to verify backend side-effects that are invisible at the API response layer.
* **Infrastructure & Container Testing:** Programmatic verification of Docker configurations. Students will write assertions against health-check statuses, resource limits, and privilege escalation vulnerabilities (e.g., Docker socket mounting).

### Expanded Micro-Crucible Architecture

```text
       [ IDE / Cursor ] <────> [ Rust-Watcher (<50ms) ] <────> [ MCP Server ]
                                      │                            │
                                      ▼                            ▼
                    ╔══════════════════════════════════════════════════════╗
                    ║            THE EXPANDED MICRO-CRUCIBLE               ║
                    ║       (Pathological Distributed Environment)         ║
                    ╠══════════════════════════════════════════════════════╣
                    ║ [UI Layer] <───> [API Gateway] <───> [Telemetry/OTel]║
                    ║      │                │                │             ║
                    ║      ▼                ▼                ▼             ║
                    ║ [Chaos Proxy] <─> [Kafka/MQ] <───> [DB/Redis]        ║
                    ║      │                │                │             ║
                    ║      └───────┬────────┴────────────────┘             ║
                    ║              ▼                                       ║
                    ║     (Injected Latency / Race Conditions / Drifts)    ║
                    ╚══════════════════════════════════════════════════════╝
```

These technical enhancements ensure the platform evolves from a UI testing tool into a comprehensive systems validation suite.

## 3. Strategic Structural Shifts: Polyglot Parity & Reverse QA

Modern engineering excellence requires "Tool Polyglotism." Engineering teams are rarely mono-stack; therefore, providing parity across TypeScript, Python, and Java is a non-negotiable differentiator. This shift decouples the architectural principle from the language syntax, forcing the learner to focus on the logic of the test rather than the flavor of the framework.

### Language-Polyglot Parity

We are implementing a dynamic configuration engine that allows the Core 11 tracks to be solved interchangeably using Playwright (TS), Pytest (Python), or REST Assured (Java). The MCP Server will leverage its get_diagnostic_report capability to perform AST anti-pattern analysis across all three languages, ensuring that performance traps specific to the Java Virtual Machine or Python’s GIL are identified and scored.

### The 'Testing the Tests' (Reverse QA) Track

Learners will be tasked with refactoring "pathological" test suites—codebases saturated with technical debt, slow execution times, and non-deterministic logic.

| Module | Technical Debt Challenge | Success strengthening (Reverse QA) |
| :--- | :--- | :--- |
| **Locator Remediation** | Brittle XPaths and nested CSS selectors. | Conversion to semantic getByRole accessibility tree locators. |
| **Race Condition Fixes** | Hard-coded sleep() calls used for hydration. | Implementation of deterministic web-first assertions and actionability checks. |
| **State Decoupling** | Inter-test dependencies causing cascading failures. | Implementation of storageState isolation and idempotent DB seeding. |

This track develops the critical ability to manage technical debt—the most common hurdle in enterprise SDET roles.

## 4. Redefined Learning Paths: Targeted Career Acceleration

Granular learning paths transform the curriculum into high-stakes career roadmaps, aligning XP milestones with the specific technical rigor required for senior engineering roles.

### Path A: Manual-to-SDET Acceleration

* **Objective:** Transition from manual validation to robust systems automation.
* **Key Tracks:** Foundations, Web UI (Playwright TS), API Resilience.
* **XP Milestones:** 0 XP (Trainee) to 3,000 XP (Senior QA).
* **Capstone 'Boss Fight':** A multi-page React application where hydration timing causes non-deterministic click drops.
* **Roadmap:** `[Foundations] ===(DOM Mastery)===> [Web UI] ===(API Logic)===> [!! BOSS: HYDRATION !!]`

### Path B: Enterprise Test Architect

* **Objective:** Design scalable, deterministic, and performant validation infrastructures.
* **Key Tracks:** Performance (k6), Contract Testing (Pact), Tool Decision Matrices.
* **XP Milestones:** 3,000 XP (Senior QA) to 10,000 XP (QA Architect).
* **Capstone 'Boss Fight':** A cross-service Pact verification challenge involving breaking additive vs. destructive API changes.
* **Roadmap:** `[Perf Profiling] --(Contract Gates)--> [Decision Matrix] --(Scaling)--> [!! BOSS: PACT GATES !!]`

### Path C: DevSecOps & Security Automation Champion

* **Objective:** Integrate security and chaos resilience into the hardened CI/CD pipeline.
* **Key Tracks:** Cloud DevSecOps, Chaos Proxy, API Resilience.
* **XP Milestones:** 6,000 XP (Lead QA) to 20,000 XP (SDET Master).
* **Capstone 'Boss Fight':** Detecting and blocking a JWT signature bypass exploit in a hardened containerized environment.
* **Roadmap:** `[Docker Hardening] ~~ (Chaos Proxy) ~~> [Auth Bypass] ~~ (CI/CD) ~~> [!! BOSS: JWT EXPLOIT !!]`

## 5. Roadmap Implementation & Quality Benchmarks

To maintain the integrity of our feedback loop, the 4D Feedback Matrix must be redefined for non-UI contexts. The rigor of the platform depends on our ability to measure backend performance and observability with the same precision as UI interaction.

### The 4D Matrix Evolution for Backend/OTel

* **Correctness:** Verified through idempotent side-effect checks (e.g., DB state matches trace context).
* **Flakiness Guard:** Determinism scoring under Chaos Proxy-injected Kafka lag and network jitter.
* **Locator Quality (Redefined):** Evaluated as Trace Context Propagation and Schema Contract Adherence. Brittle trace-linking or schema drifts result in score deductions.
* **Speed:** Assertion of p99 latency against established baseline benchmarks for gRPC and API endpoints.

### Platform Developer Milestone Checklist (MCP Server & Core)

* [x] **Polyglot AST Analysis:** Update `get_diagnostic_report` to support Java (REST Assured) and Python (Pytest) performance trap detection. (`src/feedback.rs`, dispatched via `src/mcp.rs`)
* [x] **OTel Hinting:** Expand `get_hints` to provide progressive 3-tier guidance for Span ID correlation and distributed trace assertions. (`telemetry_hints()` in `src/feedback.rs`, wired via `src/mcp.rs`)
* [x] **Micro-Crucible Expansion:** Integrate Kafka brokers and OpenTelemetry collectors into the standard `docker-compose.yml` orchestration. Containers and `otel-collector-config.yaml` are real and valid; note the backend does not yet emit real OTLP or use a Kafka client — `tracing.py` is a documented "Dummy OpenTelemetry" middleware that fabricates trace/span IDs and logs to stdout, and `kafka_lag` in `app.py` is a simulated chaos delay, not an actual broker integration. Follow-up: wire a real `opentelemetry-sdk` exporter and/or a Kafka client if live integration (not just simulated lag) is desired.
* [x] **Badging System:** Implement "Chaos Survivor" and "The Architect" badges in the Mission Control UI based on Path B and C completions. (`crucible/frontend/src/components/badges/types.ts`, unlock logic in `BadgesShowcase.tsx` driven by live `/api/progress` data)

This roadmap reaffirms our commitment to a zero-cloud, local-first environment where mastery is not granted through slides, but earned by systematically breaking and fixing real systems.
