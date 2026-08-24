# Contributing to cherenkov-lings

> Thanks for helping us expand the curriculum. Every new drill teaches a real QA engineer a real skill.

## How to Add a New Drill

### 1. Pick a track and drill number

Each track lives in `exercises/<NN>_<track_name>/`. Drills are numbered sequentially:

`
exercises/01_web_playwright_ts/
  01_hydration_timing/    <- existing
  02_shadow_dom_v2/       <- existing
  04_my_new_drill/        <- your new drill
`

### 2. Create the three required files

Every drill must have exactly these three files:

| File | Purpose | Contract |
|---|---|---|
| `exercise.ts` / `.java` / `.js` / `.yaml` / `.py` | The broken/incomplete starting code | Must **FAIL** against the running Crucible |
| `solution.ts` / `.java` / etc. | The correct, flakiness-resistant version | Must **PASS** all 5 chaos iterations |
| `hints.md` | Exactly 3 progressive hints | Nudge, Pattern, Diff |

### 3. The Pedagogical Contract

The `exercise` file must:
- Contain a `// TODO:` comment (or `# TODO:` for Python/YAML) identifying the exact anti-pattern to fix
- Fail for a **clear pedagogical reason** (not a typo, not a missing import)
- The failure message should be a clue, not noise

The `hints.md` must follow this structure:
`markdown
# Hints: Drill NN — Title

## Hint 1 (Architectural Nudge)
Why does this class of anti-pattern cause flakiness in production?

## Hint 2 (API Pattern)
Show the correct API or pattern, without giving away the full solution.

## Hint 3 (Code Diff)
A minimal diff showing exactly what to change.
`

### 4. Test your drill end-to-end

`powershell
# Start the Crucible
.\crucible\start.bat

# Verify exercise FAILS
npx playwright test exercises/01_web_playwright_ts/04_my_new_drill/exercise.ts

# Verify solution PASSES
npx playwright test exercises/01_web_playwright_ts/04_my_new_drill/solution.ts

# Run through the watcher loop
cherenkov-lings watch --track=playwright-ts
# then save exercise.ts and confirm the scorecard appears
`

### 5. Verify Rust tests still pass

`powershell
cargo test
# Must show 0 failed across all suites
`

### 6. Open a Pull Request

The CI pipeline (`ci.yml`) will automatically:
- Lint with `cargo clippy` (no warnings allowed)
- Check formatting with `cargo fmt`
- Run all 254+ Rust tests
- Lint the Python backend with `ruff`

---

## How to Add a New Track

1. Add a new `[[tracks]]` entry in `lings.toml`
2. Create the exercises directory `exercises/NN_track_name/`
3. If it needs a new runner type, implement it in `src/runner.rs` following the `Runner` trait
4. Add anti-pattern detection patterns in `src/feedback.rs`
5. Wire the runner into `src/main.rs`'s `watch` command

---

## Philosophy Reminder

> "Not to evaluate learners, but to help them learn from A to Z."

- Language: **feedback**, not grades. **insights**, not scores. **puzzles**, not tests.
- Every drill must teach something a QA Engineer will encounter in production.
- No synthetic toy examples. Every pathology in the Micro-Crucible is based on a real incident class.
