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

### Register the drill in the curriculum manifest

`lings.toml` is the **single source of truth** for the curriculum. It is read by
the Rust engine (at runtime, and as a compile-time fallback via `include_str!`)
and by the FastAPI backend (`crucible/backend/curriculum.py`, which serves
`GET /api/curriculum`). Add your drill to its track:

```toml
  [[tracks.drills]]
  id = "04_new_concept"          # must match the directory name
  name = "Human-Readable Drill Title"
```

Do not hardcode curriculum data anywhere else. `tests/curriculum_manifest_tests.rs`
fails the build if the manifest and the repository diverge in either direction:
a drill on disk that is missing from the manifest, or a manifest entry with no
directory behind it.

Adding a whole track means one `[[tracks]]` block. Two optional keys cover
non-standard layouts:

* `drill_root` - where the drill directories actually live, when that is not
  `exercise_dir` (the Maven-structured Java track uses this).
* `exercise_file` / `solution_file` - when the starter and solution filenames
  are not `exercise{extension}` / `solution{extension}`.

Verify your drill with:
```bash
cherenkov-lings audit
```

If it does not pass the audit, the CI pipeline will reject the PR.

## ?? Expanding the Micro-Crucible
If your drill requires a new failure mode (e.g., a specific GraphQL vulnerability), add it to `crucible/backend/app.py` or the React frontend. Ensure you respect the `X-Chaos` header patterns established in `ChaosMiddleware`.

Thank you for helping us elevate the standard of QA Engineering globally!
