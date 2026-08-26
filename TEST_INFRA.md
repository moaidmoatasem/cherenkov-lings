# E2E Test Infra: Cherenkov-Lings QA Learning Engine

## Test Philosophy
- Opaque-box, requirement-driven, and multi-tier verification.
- Enforces strict zero-tolerance gate pass across Rust compiler/clippy, Python backend chaos test suite, Ruff linter, Playwright browser automation, and interactive CLI dashboard rendering.

## Verification Gate Commands
```powershell
# Gate 1: Rust Unit & Integration Tests (359 existing tests + new tests)
cargo test

# Gate 2: Zero-Warning Compiler & Clippy Strict Linter
cargo clippy -- -D warnings

# Gate 3: Micro-Crucible Chaos Pytest Suite (17 existing + 8+ new endpoint tests)
python -m pytest tests/test_micro_crucible_chaos.py

# Gate 4: Python Ruff Linter for Crucible Backend
python -m ruff check crucible/

# Gate 5: Playwright TS E2E Tests in GenAI Track
npx playwright test exercises/06_genai_qa/

# Gate 6: Interactive Dashboard Subcommand
cargo run -- dashboard
```

## Curriculum & Module Completeness Checks
1. **Curriculum Completeness**:
   - Track 1 Playwright: 10 drills
   - Track 2 REST Assured: 7 drills
   - Track 3 Maestro: 5 drills
   - Track 4 k6: 5 drills
   - JMeter: 8 drills (with complete exercise + solution + hints)
   - Tool Decisions: 4 drills
   - Track 0 (5), Track 5 (2), Track 6 (2)
   - Total 60 drills across 11 tracks.
2. **Bundle Integrity**: Every drill directory contains `exercise.*`, `solution.*`, `hints.md`, and `theory.md`.
3. **Production Story**: Every exercise file contains a named real-world incident comment block.
4. **Theory Modules**: Every `theory.md` >= 150 words, contains incident story, mechanism, ASCII diagram, and closes with "You will now simulate this in the Crucible".
5. **Gamification & Progress**: `.cherenkov-progress.json` format, XP calculations, 7 levels, 8 badges.
6. **JMeter Runner**: Gracefully handles missing `jmeter` without panic.
