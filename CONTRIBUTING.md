# Contributing to cherenkov-lings

First off, thank you for considering contributing to cherenkov-lings! 

This project aims to be the most comprehensive, scientifically rigorous experiential learning platform for Quality Engineering.

## ?? Core Philosophy
1. **Realism**: Drills must represent actual production failures, not synthetic toy problems.
2. **Resilience**: The platform deliberately injects network and timing chaos. Solutions must survive chaos, not just "happy path" local runs.
3. **Empathy**: Feedback should act like a Senior SDET mentoring a junior. Never use punitive language ("Failed 0/100"); use coaching language ("Needs Flakiness Hardening").

## ??? Local Development Setup

1. **Rust Toolchain**: Must use `stable-x86_64-pc-windows-msvc` (or target OS equivalent).
2. **Node.js**: v18+ required for Playwright / React.
3. **Python**: v3.11+ required for FastAPI / Pytest.

### Running the Test Gates
Before submitting a PR, you **must** pass all verification gates:
```bash
# 1. Rust Core Engine Tests & Lints
cargo clippy --all-targets -- -D warnings
cargo fmt --all -- --check
cargo test --all

# 2. Python Backend & Security Suites
python -m pytest tests/test_micro_crucible_chaos.py

# 3. TypeScript Compilation
npx tsc --noEmit (in crucible/frontend)
```

## ??? Adding a New Drill (The Strict Contract)

We do not accept "half-drills". Every drill in the platform must conform to a strict structural contract.

Use the built-in CLI to scaffold your drill:
```bash
cherenkov-lings new-drill --track=playwright-ts --name=04_new_concept
```

This ensures your directory has the mandatory 4 files:
1. `exercise.ts / .py`: The broken code containing the anti-pattern, marked with `// TODO`.
2. `solution.ts / .py`: The flakiness-resistant, chaos-proof solution.
3. `hints.md`: Exactly 3 progressive hints:
   - Hint 1: Architectural Nudge
   - Hint 2: API Pattern
   - Hint 3: Code Diff
4. `theory.md`: The real-world production incident story (must be = 150 words) and an ASCII failure diagram.

Verify your drill with:
```bash
cherenkov-lings audit
```

If it does not pass the audit, the CI pipeline will reject the PR.

## ?? Expanding the Micro-Crucible
If your drill requires a new failure mode (e.g., a specific GraphQL vulnerability), add it to `crucible/backend/app.py` or the React frontend. Ensure you respect the `X-Chaos` header patterns established in `ChaosMiddleware`.

Thank you for helping us elevate the standard of QA Engineering globally!
