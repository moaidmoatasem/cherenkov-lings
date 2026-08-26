# Architecture Analysis & Design Blueprint: "-lings" Experiential Learning Platforms

**Platform:** QA & SDET Lings Experiential Learning Platform (`cherenkov-lings`)  
**Document:** `architecture_notes.md` (Requirement R1)  
**Author:** Worker 1  
**Project Workspace:** `C:\Users\moaid\Documents\antigravity\wonderful-raman`  
**Status:** Canonical Reference Architecture & Pedagogy Guide  

---

## 1. Executive Summary & Philosophy of Experiential Pedagogy

Interactive "-lings" style learning platforms—originating with **Rustlings** and evolving through **Ziglings**, **Pylings**, and **Cherenkov-Lings**—represent a transformative shift in technical education: **local-first, drill-based, experiential active recall**. 

Traditional software training relies heavily on passive video tutorials, cloud-hosted sandboxes with high latency, or vendor-locked SaaS playgrounds. In contrast, the "-lings" methodology drops the student directly into their own local IDE with:
1. **Authentic Broken Starter Code**: Real code containing realistic anti-patterns or bugs.
2. **Zero-Friction Feedback Loop**: Reactive file-system watchers dispatch the compiler or test runner on save (`Ctrl+S`) after a 50ms debounce, so the only wait is the runner itself — no manual re-run, no CI queue.
3. **Compiler / Runner as Socratic Mentor**: Clear, sanitized error diagnostics and progressive hint scaffolds rather than outright spoilers.
4. **Zero-Cloud & Zero-GPU Resilience**: Complete offline operation on localhost without external API keys, database servers, or GPU clusters.
5. **Intentional Progression Gates**: Deliberate sentinel markers (`// I AM NOT DONE`) ensuring learners actively engage before moving forward.

`cherenkov-lings` extends this paradigm beyond single-language syntax tutorials into the multi-disciplinary domain of **Quality Engineering & Software Development in Test (SDET)**.

---

## 2. Comparative Analysis: Exemplar "-lings" Platforms

Below is an architectural comparison of major "-lings" implementations alongside `cherenkov-lings`:

| Dimension | **Rustlings** | **Ziglings** | **Py-lings / Pylings** | **Cherenkov-Lings (Our Platform)** |
| :--- | :--- | :--- | :--- | :--- |
| **Domain** | Rust language syntax, borrow checker, lifetimes, generics | Zig language syntax, manual memory management, C interop | Python syntax, data structures, OOP idioms | **Quality Engineering, SDET, UI/API/Perf/Security/GenAI Automation** |
| **Execution Engine** | `rustc` compiler / `cargo test` | `zig test` / `zig build` | `python -m pytest` | **Polyglot Runner Pool** (Node.js IPC, Pytest JSON, JVM/Maven, k6, JMeter, Maestro) |
| **Manifest / Discovery** | `info.toml` with strict ordered sequence | Numbered directory scan (`001_xxx.zig`) | `exercises.json` or directory scan | `lings.toml` (11 track definitions, evaluation thresholds, chaos proxy parameters) |
| **Exercise Sentinel** | `// I AM NOT DONE` | `// "I AM NOT DONE"` or inline markers | `# I AM NOT DONE` or `TODO` tags | `// I AM NOT DONE` / `# I AM NOT DONE` + static anti-pattern scanner |
| **Evaluation Dimensions** | 1D: Boolean compile/test pass | 1D: Boolean compilation pass | 1D: Boolean pytest assertion pass | **4D Matrix**: Correctness (35%), Chaos Flakiness (35%), Locator Quality (15%), Speed (15%) |
| **Target Application** | Self-contained standard library calls | Self-contained Zig standard library | In-memory functions / mocks | **Micro-Crucible Live Sandbox** (FastAPI backend + Vite/React frontend + Chaos Proxy) |
| **Gamification** | Simple completion bar | Sequential index counter | Percentage counter | **7 SDET Career Ranks, Level Formulas, 8 Badges, Daily Streaks, ANSI Dashboard** |
| **Hint System** | Single `rustlings hint <name>` | Inline comment hints / patch diffs | Markdown hint files | **3-Tier Progressive Scaffolding** (`hints.md`: Tier 1 Concept, Tier 2 API, Tier 3 Snippet) |
| **AI / Tooling Interop** | None | None | None | **Built-in Model Context Protocol (MCP)** server for IDE AI agents |

---

## 3. Core Architectural Subsystems

```
┌────────────────────────────────────────────────────────────────────────┐
│                       LEARNER WORKSPACE (IDE / Editor)                 │
│         exercises/00_foundations/01_what_is_a_test/exercise.py         │
└───────────────────────────────────┬────────────────────────────────────┘
                                    │ (File Save: Ctrl+S)
                                    ▼
┌────────────────────────────────────────────────────────────────────────┐
│                        CORE LEARNING ENGINE (Rust)                     │
│                                                                        │
│  ┌──────────────────────┐   50ms Debounce   ┌───────────────────────┐  │
│  │   notify Watcher     │ ────────────────> │  Ignore Filter        │  │
│  │ (OS File Events)     │                   │ (target/, .tmp, etc.) │  │
│  └──────────────────────┘                   └──────────┬────────────┘  │
│                                                        │               │
│                                                        ▼               │
│  ┌──────────────────────────────────────────────────────────────────┐  │
│  │                     Polyglot Runner Subsystem                    │  │
│  │   ┌──────────────┐  ┌──────────────┐  ┌───────────────────────┐  │  │
│  │   │ Node.js IPC  │  │ Pytest JSON  │  │ JVM / k6 / Maestro    │  │  │
│  │   └──────┬───────┘  └──────┬───────┘  └───────────┬───────────┘  │  │
│  └──────────┼─────────────────┼──────────────────────┼──────────────┘  │
│             │                 │                      │                 │
│             ▼                 ▼                      ▼                 │
│  ┌──────────────────────────────────────────────────────────────────┐  │
│  │                     Micro-Crucible Sandbox                       │  │
│  │  FastAPI (8081) ◄─── L4/L7 Chaos Proxy (8086) ◄─── React (8080)   │  │
│  └──────────────────────────────────┬───────────────────────────────┘  │
│                                     │ Multi-Iteration Execution        │
│                                     ▼                                  │
│  ┌──────────────────────────────────────────────────────────────────┐  │
│  │                4D Evaluation & Static Analysis Feedback         │  │
│  │  - Correctness (35%)      - Flakiness against Chaos (35%)        │  │
│  │  - Locator Quality (15%)  - Execution Speed (15%)                │  │
│  └──────────────────────────────────┬───────────────────────────────┘  │
│                                     │                                  │
│                                     ▼                                  │
│  ┌──────────────────────────────────────────────────────────────────┐  │
│  │              Progression State Machine & Gamification            │  │
│  │  - .cherenkov-progress.json (XP, Ranks, Streaks, 8 Badges)       │  │
│  │  - Unlock Next Drill if Score >= 85.0 & Sentinel Removed         │  │
│  └──────────────────────────────────┬───────────────────────────────┘  │
│                                     │                                  │
│                                     ▼                                  │
│  ┌──────────────────────────────────────────────────────────────────┐  │
│  │                    ANSI Terminal UI / Dashboard                  │  │
│  └──────────────────────────────────────────────────────────────────┘  │
└────────────────────────────────────────────────────────────────────────┘
```

### 3.1 Exercise Progression State Machine

The progression engine manages curriculum state across multiple tracks:
- **`Locked`**: Drill prerequisites unmet.
- **`Unlocked / Active`**: Current drill being edited. Evaluated on each file save.
- **`Mastered`**: Drill passed with composite score $\ge 85.0\%$, sentinel removed, and verified under chaos. XP and badges awarded.
- **`Refactored`**: Re-evaluating previously completed drills to optimize speed or improve locator resilience.

#### Sentinel Detection Engine
The sentinel marker prevents accidental auto-advancement:
```rust
pub fn is_marked_done(source: &str) -> bool {
    let re = regex::Regex::new(r"(?m)^\s*(?://|#|--|/\*)\s*I\s+AM\s+NOT\s+DONE").unwrap();
    !re.is_match(source)
}
```

#### State Persistence Layer
Progress is stored locally in `.cherenkov-progress.json`:
```json
{
  "total_xp": 1250,
  "level": 3,
  "streak_days": 4,
  "completed_drills": ["foundations-01", "foundations-02", "web-01"],
  "track_progress": {
    "foundations": { "completed": 2, "total": 5 },
    "playwright-ts": { "completed": 1, "total": 10 }
  },
  "unlocked_badges": ["first_blood", "resilience_master"]
}
```

---

### 3.2 High-Velocity File Watcher & Debouncing Engine

To maintain flow state, file saving must trigger evaluation in under 50 milliseconds without duplicate process spawns:
- **Kernel-Level File Events**: Powered by the Rust `notify` crate (`ReadDirectoryChangesW` on Windows, `kqueue` on macOS, `inotify` on Linux).
- **50ms Sliding-Window Debounce**: Modern editors perform atomic saves by writing temporary files and renaming them. The debouncer aggregates burst events and emits a single dispatch per save.
- **Ignore Filter**: Discards `target/`, `node_modules/`, `__pycache__/`, `.git/`, swap files, and build outputs.

---

### 3.3 Polyglot Runner & Worker Subsystem

In a polyglot QA curriculum, tests span multiple runtimes:
- **Node.js Long-Lived IPC Worker**: Avoids cold-start VM overhead by communicating over stdin/stdout JSON-RPC for Playwright and Axe drills.
- **Pytest JSON Subprocess Runner**: Executes Python tests with structured JSON test outcome reporting.
- **JVM Maven Runner**: Executes REST Assured Java tests with Surefire XML parsing.
- **k6 / JMeter / Maestro Runners**: CLI wrappers with summary output parsing and graceful fallback logic.

---

### 3.4 Live Embedded Sandbox & Chaos Engine (Micro-Crucible)

Unlike standard programming drills that test pure algorithms in isolation, SDET drills require a real target application with deliberate runtime pathologies:
- **FastAPI Backend (Port 8081)**: In-memory REST, JWT authentication, GraphQL endpoints, SSE streaming, idempotency token validation.
- **Vite / React Frontend (Port 8080)**: Simulates hydration delays, Shadow DOM boundaries, async data grids, and flaky UI race conditions.
- **L4/L7 Chaos Proxy (Port 8086)**: Programmable network proxy injecting latency ($200\text{ms} \pm 75\text{ms}$), TCP socket drops, and HTTP 502/504 errors to test automation resilience.

---

### 3.5 Sacred 4-File Exercise Anatomy

Every drill across all tracks strictly adheres to the **4-File Sacred Contract**:

```
exercises/00_foundations/01_what_is_a_test/
├── exercise.py       # Broken starter code containing anti-patterns and sentinel
├── solution.py       # Flakiness-tested reference solution
├── hints.md          # 3-tier progressive hints (Concept, Syntax, Snippet)
└── theory.md         # Deep architectural background & real-world outage story
```

#### Anatomical Requirements for `theory.md`:
1. **Real-World Incident Case Study**: (e.g., NASA Mars Climate Orbiter unit conversion failure, Knight Capital $440M trading loss, Amazon Prime Day hydration drops).
2. **Protocol & Runtime Explanation**: Deep architectural root cause analysis.
3. **ASCII Diagram**: Visual flow diagram illustrating failure mechanics.
4. **Crucible Simulation Anchor**: Direct instructions to test against the local sandbox.

---

### 3.6 4D Feedback Matrix & Static Source Analysis

Evaluation goes beyond binary pass/fail to evaluate code quality, resilience, and speed:

$$\text{Composite Score} = (0.35 \times \text{Correctness}) + (0.35 \times \text{Flakiness}) + (0.15 \times \text{LocatorQuality}) + (0.15 \times \text{Speed})$$

- **Correctness (35%)**: Complete assertion verification against the system under test.
- **Flakiness Guard (35%)**: 5 consecutive test iterations executed against network chaos and jitter.
- **Locator Quality (15%)**: Static source analysis (regex rules over comment-stripped source, not a parsed syntax tree) penalizing anti-patterns:
  - Hardcoded sleeps (`page.waitForTimeout`, `time.sleep`) $\to -40\text{ pts}$
  - Absolute XPath (`/html/body/div[2]/span`) $\to -50\text{ pts}$
  - Semantic Roles (`getByRole`, `getByTestId`) $\to +100\text{ pts}$
- **Speed Benchmarking (15%)**: Non-blocking async execution compared against baseline thresholds.

---

## 4. Best Practices for Designing Local, Experiential Learning Exercises

When designing drills for a local-first QA/SDET platform:

1. **Zero External Dependencies**: Never make outbound calls to cloud APIs, third-party databases, or vendor endpoints. All state must live in memory or local SQLite.
2. **Deterministic Failure Scaffolding**: Starter exercises must fail reliably for the right architectural reason, not due to missing environment variables or network timeouts.
3. **Sanitized Error Reporting**: Strip framework-internal stack traces (e.g., 50 lines of Python or Node.js internal frames) and highlight the exact failing line and assertion expectation.
4. **Progressive Hint Scaffolding**: Provide 3 distinct tiers:
   - **Tier 1 (Concept)**: Gentle nudge toward the architectural pattern.
   - **Tier 2 (API / Syntax)**: Specific method signature or library approach.
   - **Tier 3 (Code Snippet)**: Exact code replacement block.
5. **Automated Solver Verification**: Maintain automated solver scripts (`verify_all_exercises.py`) in CI to guarantee every exercise remains 100% solvable.

---

## 5. Conclusion

The "-lings" experiential architecture transforms QA/SDET education by replacing passive consumption with active, high-frequency practice. By pairing low-latency reactive watching with authentic production post-mortems and multi-dimensional chaos validation, `cherenkov-lings` provides a rigorous, offline-first proving ground for modern automation engineers.
