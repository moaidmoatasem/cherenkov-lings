# ?? Instructions for AI Agents Contributing to cherenkov-lings

Hello, fellow AI Agent! If a human has asked you to contribute to this repository, please read these rules carefully. This codebase is highly interdependent, and breaking the "Curriculum Contract" will cause the platform's gamification engine to panic.

## ?? CRITICAL RULES

1. **The 4-File Drill Contract is Sacred**
   If you are asked to create or modify an exercise, it MUST contain exactly these four files:
   - `exercise.<ext>` (Must contain a `// TODO:` or `# TODO:` comment marking the anti-pattern).
   - `solution.<ext>` (Must perfectly pass under chaotic network conditions).
   - `hints.md` (Must contain exactly `## Hint 1`, `## Hint 2`, and `## Hint 3`).
   - `theory.md` (Must be = 150 words and include a fenced ASCII diagram).
   *Verification:* Always run `cargo run -- audit` after modifying drills. DO NOT commit if the audit fails.

2. **The Micro-Crucible is Intentionally Pathological**
   If you are editing `crucible/backend/app.py` or `crucible/frontend/src/`, **DO NOT "FIX" THE BUGS**.
   The Crucible is *supposed* to be broken. It has React hydration traps, SQL injections, SSRF vulnerabilities, and Kafka eventual consistency lag. If you "fix" the backend, the QA exercises will no longer work. 
   - *Exception:* You may add new endpoints or new failure mechanisms.

3. **Chaos Header Propagation**
   If you add a new endpoint to the FastAPI backend, you must ensure it yields to the `X-Chaos` header (parsed by `ChaosMiddleware`). Use `asyncio.sleep` to respect the `delay` and `jitter` fields if applicable.

4. **Never Break the 4D Feedback Matrix**
   The Rust AST parser (`src/feedback.rs`) statically analyzes TypeScript and Python files to score "Locator Quality" and penalize "Hardcoded Sleeps". 
   - Do not change how Playwright/Pytest reporters output JSON without also updating `src/runner.rs`.

5. **Powershell Environment**
   If you are executing terminal commands, you are likely in a PowerShell environment. 
   - Do NOT use `<` for file redirection.
   - Do NOT use `&&`. Use `;`.
   - When using `Set-Content` with multi-line strings, use the `@' ... '@` here-string syntax.

## ?? Verification Protocol
Before telling the human you have completed a task, you MUST run this sequence:
1. `cargo test --all`
2. `cargo run -- audit`
3. `python -m pytest tests/test_micro_crucible_chaos.py`

If any of these fail, fix your code before reporting completion.

Good luck, agent!
