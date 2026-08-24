# cherenkov-lings

> **An interactive, local-first experiential learning platform for QA Engineers and SDETs.**
> Master modern test automation across UI, API, Mobile, Performance, and GenAI — with zero cloud, zero GPU, zero vendor lock-in.

[![CI](https://github.com/your-org/cherenkov-lings/actions/workflows/ci.yml/badge.svg)](https://github.com/your-org/cherenkov-lings/actions)
[![License: MIT](https://img.shields.io/badge/License-MIT-yellow.svg)](LICENSE)

---

## What is this?

Cherenkov-lings is inspired by [Rustlings](https://github.com/rust-lang/rustlings). The core idea: **you learn test automation by breaking real things, not reading slides.**

Every drill runs against a purposely pathological embedded application, the **Micro-Crucible**, simulating exact production failures: React hydration races, Kafka lag, JWT mid-session expiry, database pool starvation, biometric auth failures, and LLM hallucinations.

When you save an exercise file, the Rust watcher detects the change within **50ms** and prints your 4D Feedback Matrix:

```
======= CHERENKOV-LINGS 4D FEEDBACK MATRIX =======
  Correctness      |   0  | [FAIL] waitForTimeout caused race
  Flakiness Guard  |  20  | Anti-pattern: waitForTimeout on line 8
  Locator Quality  | 100  | getByRole - semantic and resilient
  Speed            |  85  | 1.2s vs 1.0s baseline
  TOTAL SCORE: 42/100 -- check hints.md for guidance
==================================================
```

---

## Who is this for?

| Persona | Track |
|---|---|
| Manual QA transitioning to automation | Playwright TypeScript (Web UI) |
| Automation Engineer | REST Assured Java (API), Maestro YAML (Mobile) |
| Performance / Load Engineer | k6 JavaScript |
| AI-Era QA Engineer | Playwright TypeScript (GenAI Testing) |
| DevOps / Security QA | Python Pytest (DevSecOps) |

---

## Quick Start

### Prerequisites

- **Rust (MSVC)**: https://rustup.rs
- **Node.js 18+**: https://nodejs.org
- **Python 3.11+**: https://python.org
- **Java 17 + Maven**: https://openjdk.org

### 1. Install globally

```powershell
git clone https://github.com/your-org/cherenkov-lings
cd cherenkov-lings
.\install.ps1
```

### 2. Start the Micro-Crucible sandbox

```powershell
.\crucible\start.bat
```

Starts FastAPI backend on port 8081 and React frontend on port 8080.

### 3. Start the interactive watcher

```powershell
cherenkov-lings watch --track=playwright-ts
```

Available tracks: `playwright-ts`, `restassured-java`, `maestro-mobile`, `k6-js`, `genai-qa`, `devsecops-python`

### 4. Start learning!

Open any `exercise.ts` / `exercise.java` / `exercise.js` / `exercise.yaml` / `exercise.py` in your editor and hit Save. The feedback loop fires instantly. Check `hints.md` in the same drill directory when you are stuck.

---

## Track Index

### Track 1: Modern Web Automation (Playwright TypeScript)
exercises/01_web_playwright_ts/

| Drill | Anti-Pattern | Learning |
|---|---|---|
| 01_hydration_timing | waitForTimeout on hydrating button | waitFor attribute assertion |
| 02_shadow_dom_v2 | XPath into closed shadow root | pierce + semantic locators |
| 03_debounce_race_condition | Fixed assertion on out-of-order responses | Promise race handling |

### Track 2: API Resilience (REST Assured Java)
exercises/02_api_restassured_java/

| Drill | Anti-Pattern | Learning |
|---|---|---|
| drill01_idempotency | No retry on HTTP 409 | Idempotency key strategy |
| drill02_jwt_auth | No token refresh on 401 | JwtRefreshFilter pattern |
| drill03_kafka_lag | Thread.sleep() for async wait | Awaitility poll |

### Track 3: Mobile Automation (Maestro YAML)
exercises/03_mobile_maestro/

| Drill | Anti-Pattern | Learning |
|---|---|---|
| 01_biometric_fallback | No conditional flow on auth failure | runFlow: when: visible |
| 02_deep_link_cold_start | openLink assumes warm process | launchApp: arguments.deeplink |
| 03_activity_recreation | No post-rotation assertion | setOrientation + re-assert |

### Track 4: Load Testing (k6 JavaScript)
exercises/04_perf_k6_js/

| Drill | Anti-Pattern | Learning |
|---|---|---|
| 01_database_pool_starvation | 50 flat VUs cold start | Staged ramp-up with stages |
| 02_spike_profile_p99 | No tail latency metric | Trend metric + p99 threshold |
| 03_chaos_sla_assertion | No error rate tracking | Rate metric + SLA threshold |

### Track 5: GenAI QA (Playwright TypeScript)
exercises/05_genai_qa/

| Drill | Anti-Pattern | Learning |
|---|---|---|
| 01_rag_context_faithfulness | Exact-match on LLM answer string | Assert on grounded + source_facts |
| 02_llm_assertion_flakiness | Assert on raw_text that rotates | Assert on intent + entities |

### Track 6: Cloud-Native & DevSecOps (Python Pytest)
exercises/06_cloud_devsecops/

| Drill | Anti-Pattern | Learning |
|---|---|---|
| 01_insecure_docker_mount | docker.sock volume mount | Principle of least privilege |
| 02_jwt_weak_signing_key | alg: none in JWT config | Whitelist secure algorithms only |

---

## Chaos Proxy

The Chaos Proxy runs on port 8086 and forwards to port 8081:

```bash
# Inject 800ms delay
curl -H "X-Chaos: delay=800" http://localhost:8086/checkout

# Force HTTP 502 Bad Gateway
curl -H "X-Chaos: status=502" http://localhost:8086/checkout

# Simulate Kafka eventual consistency lag
curl -H "X-Chaos: kafka_lag=1500" http://localhost:8081/transfer
```

---

## AI-IDE Integration (MCP Server)

Open this folder in Cursor or VS Code. The `.cursor/mcp.json` and `.vscode/mcp.json` files auto-register cherenkov-lings as an MCP server.

Your AI assistant can then call:
- **get_diagnostic_report**: AST analysis of your exercise file, structured anti-patterns with recommendations.
- **get_hints**: Progressive 3-hint coaching for any drill.

You can test MCP manually:
```powershell
echo '{"jsonrpc":"2.0","id":1,"method":"tools/list"}' | cherenkov-lings mcp
```

---

## Architecture

```
cherenkov-lings/
+-- src/
|   +-- main.rs         # CLI: init | watch | diagnose | proxy | mcp
|   +-- watcher.rs      # 50ms debounced file watcher
|   +-- runner.rs       # Node IPC | JVM (Maven) | k6 | Maestro | Pytest
|   +-- feedback.rs     # 4D Feedback Matrix + AST anti-pattern scanner
|   +-- proxy.rs        # L4/L7 Programmable Chaos Proxy (8086->8081)
|   +-- config.rs       # lings.toml typed parser
|   +-- mcp.rs          # JSON-RPC stdio MCP server
+-- workers/
|   +-- node_worker.js  # Pre-warmed NDJSON IPC Node worker
|   +-- pytest_worker.py
+-- crucible/
|   +-- backend/app.py  # FastAPI: /checkout /transfer /search /api/rag /api/llm
|   +-- frontend/       # Vite + React (hydration, Shadow DOM, debounce, iframe)
|   +-- start.bat       # One-command Crucible launcher
+-- exercises/          # 6 tracks, 13 drills
+-- .cursor/mcp.json    # Cursor AI auto-registers MCP tools
+-- .vscode/mcp.json    # VS Code / Copilot auto-registers MCP tools
+-- lings.toml          # Platform manifest: tracks, thresholds, ports
+-- install.ps1         # Windows global install script
```

---

## Philosophy

> "Not to evaluate learners, but to help them learn from A to Z."

Every word is chosen intentionally:
- **Feedback**, not grades. **Insights**, not scores.
- **Puzzles** to solve, not tests to pass.
- **Detective clues** in hints, not answers handed to you.

---

## License

MIT
