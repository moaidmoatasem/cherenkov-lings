# ⚡ cherenkov-lings

> **An interactive, local-first experiential learning platform for QA Engineers and SDETs.**
> Master modern test automation across UI, API, Mobile, Performance, Security, and GenAI — with zero cloud, zero GPU, zero vendor lock-in.

[![License: MIT](https://img.shields.io/badge/License-MIT-yellow.svg)](LICENSE)

---

## 💡 The Core Idea

Cherenkov-lings is inspired by [Rustlings](https://github.com/rust-lang/rustlings) and modern engineering pedagogy: **You master test automation by breaking real systems, not by reading slides.**

Every drill runs against a purposely pathological embedded application — the **Micro-Crucible** — which reproduces the exact production failures QA Engineers face in real-world environments:
* React hydration timing click drops
* Distributed Kafka eventual consistency lag
* JWT mid-session token invalidation
* Out-of-order debounced search clobbering
* Closed Shadow DOM encapsulation boundaries
* Concurrency connection pool starvation
* Non-deterministic LLM hallucinations

When you save an exercise file, the sub-50ms Rust watcher triggers the feedback loop and immediately evaluates your code against the **4D Feedback Matrix**:

```
╔══════════════════════════════════════════════════════════════════════════════════════╗
║  4D LEARNING & FEEDBACK MATRIX — cherenkov-lings v1.0.0                              ║
╠══════════════════════════════════════════════════════════════════════════════════════╣
║  Track   │  Modern Web Automation (Playwright TypeScript)                            ║
║  File    │  01_hydration_timing/exercise.ts                                          ║
╠═══════════════════╦══════╦═══════════════════════════════════════════════════════════╣
║  Dimension        ║ Score║ Diagnostic Insight                                        ║
╠═══════════════════╬══════╬═══════════════════════════════════════════════════════════╣
║  Correctness      ║  100 ║ [PASS] All assertions verified                            ║
║  Flakiness Guard  ║  100 ║ [PASS] 5/5 consecutive passes under injected network chaos║
║  Locator Quality  ║  100 ║ getByRole — semantic accessibility tree locator           ║
║  Speed            ║   92 ║ 820ms vs 1000ms baseline benchmark                        ║
╠═══════════════════╩══════╩═══════════════════════════════════════════════════════════╣
║  TOTAL SCORE: 98/100  │  [PASSED]  │  +150 XP Earned  │  Rank: Mid QA                ║
╚══════════════════════════════════════════════════════════════════════════════════════╝
```

---

## 🎮 Gamification & Career Progression

Track your journey from Manual QA to SDET Master:

```
  🌱 Trainee (0 XP) ──► 🔍 Junior QA (500 XP) ──► ⚡ Mid QA (1,500 XP)
                             │
                             ▼
  🔥 Senior QA (3,000 XP) ──► 🎯 Lead QA (6,000 XP) ──► 🏗️ QA Architect (10,000 XP)
                                                                 │
                                                                 ▼
                                                    ⚛️ SDET Master (20,000 XP)
```

* **8 Specialist Badges**: `First Blood`, `Flakiness Slayer`, `Chaos Survivor`, `Tool Polyglot`, `The Architect`, `Perfect Locator`, `Speed Demon`, `SDET Master`.
* **Real-Time Dashboards**: View via terminal (`cherenkov-lings dashboard`) or browser (`http://localhost:8080/mission-control`).

---

## 🚀 Quick Start

### 1. Install cherenkov-lings globally

**Windows (PowerShell):**
```powershell
git clone <repository-url>
cd cherenkov-lings
.\install.ps1
```

**macOS / Linux:**
```bash
git clone https://github.com/your-org/cherenkov-lings
cd cherenkov-lings
chmod +x install.sh && ./install.sh
```

---

### 2. Start the Micro-Crucible Sandbox

**Native:**
```bash
# Windows
.\crucible\start.bat

# macOS / Linux
chmod +x crucible/start.sh && ./crucible/start.sh
```

**Or with Docker (zero-install):**
```bash
docker compose up
```

To point the sandbox UI at a non-default backend URL, set `VITE_API_BASE` at frontend image build time (e.g. `docker compose build --build-arg VITE_API_BASE=http://192.168.1.50:8081 frontend`).

Services will be live at:
* 🌐 **Web Sandbox & Pathology Demos**: `http://localhost:8080`
* 🏆 **Mission Control & Badges**: `http://localhost:8080/mission-control`
* 🔬 **FastAPI Backend Swagger**: `http://localhost:8081/docs`

---

### 3. Start Learning!

```bash
# View your progress
cherenkov-lings dashboard

# Start watching any curriculum track
cherenkov-lings watch --track=foundations
cherenkov-lings watch --track=playwright-ts
cherenkov-lings watch --track=restassured-java
cherenkov-lings watch --track=maestro-mobile
cherenkov-lings watch --track=k6-js
cherenkov-lings watch --track=jmeter
cherenkov-lings watch --track=tool-decisions
```

Open any `exercise.*` file in your favorite editor, write your fix, and hit **Save**. Feedback is instantaneous!

---

## 📚 60-Drill Curriculum Matrix (11 Tracks)

| Track | Stack | Drills | Core Concepts & Incident Case Studies |
|---|---|:---:|---|
| **0. Foundations** | Python / Pytest | 5 | AAA pattern, assertions as documentation, avoiding mock traps, single-responsibility |
| **1. Web UI** | Playwright TS | 10 | React hydration click drops, closed Shadow DOM piercing, debounced race conditions, Page Object Model, cross-origin payment iframes, network request intercepts, visual regression thresholds, worker storageState isolation |
| **2. API Resilience** | REST Assured Java | 7 | HTTP 409 idempotency collisions, transparent JWT 401 interceptors, Kafka lag polling, multi-page pagination loops, JSON schema contracts, aliased GraphQL queries, `RequestSpecBuilder` reuse |
| **3. Mobile UI** | Maestro YAML | 5 | Biometric auth fallback flows, deep link cold starts, activity recreation & screen rotation UI state, dynamic list `scrollUntilVisible`, OS permission dialog handlers |
| **4. Performance (k6)** | k6 JS | 5 | Database pool starvation, 10x spike p99 latency profiling, chaos proxy SLA thresholds, Server-Sent Events continuous streams, InfluxDB / Grafana outputs |
| **5. Performance (JMeter)** | JMeter JMX | 8 | Non-GUI CI execution, response assertions, Gaussian random timers, listener memory leaks, CSRF extraction & correlation, throughput vs concurrency math, master-agent distributed load, automated JTL HTML dashboards |
| **6. GenAI QA & LLMs** | Playwright TS | 5 | RAG semantic faithfulness, structured intent assertions, hallucination citation grounding, prompt injection red-teaming defenses, TTFT streaming latency thresholds |
| **7. Cloud DevSecOps** | Python Pytest | 5 | Docker socket privilege escalation, JWT signature bypass, blind timing SQLi, cloud metadata SSRF blocking, CORS origin isolation |
| **8. Tool Decisions** | Python | 4 | UI vs API layer decision matrix, k6 vs JMeter enterprise tradeoffs, Appium vs Maestro mobile strategy, Pact contract vs E2E microservice testing |
| **9. Contract Testing** | Python / Pact | 3 | Consumer schema definition, automated provider verification gates, breaking additive vs destructive API changes |
| **10. Accessibility** | Playwright TS | 3 | WCAG semantic trees & color contrast, keyboard sequential focus traps, ARIA screen-reader live regions |

---

## 🧠 Theory Modules (`theory.md`)

Every single drill directory contains a standalone `theory.md` document featuring:
1. **Real-world case study** (e.g. Stripe, Shopify, GitHub, Spotify, Robinhood, Air Canada, Knight Capital).
2. **Protocol-, DOM-, or runtime-level root-cause breakdown**.
3. **Visual ASCII failure mode diagram**.
4. **"Simulate this in the Crucible"** bridge directly connecting theory to hands-on code.

---

## 🤖 AI-IDE Integration (MCP Server)

Cherenkov-lings includes a built-in Model Context Protocol (MCP) server for Cursor, VS Code, and Copilot Agent Mode:

```bash
# Starts JSON-RPC stdio MCP server
cherenkov-lings mcp
```

* `.cursor/mcp.json` and `.vscode/mcp.json` automatically register:
  * `get_diagnostic_report`: AST anti-pattern analysis & locator scoring.
  * `get_hints`: Progressive 3-tier hints for any drill.

---

## 📜 License

MIT © cherenkov-lings contributors
