# Hints: Drill 01 - Leaked Secret in Workflow

## Hint 1 (Concept)
The workflow file is committed source code. Anything you type as a literal value in it is readable by everyone who can read the repository, is not masked in build logs, and stays in `git log -p` after you delete the line. Credentials must therefore never be *values* in the file — only *references* resolved at run time.

Ask yourself: what in this file would still be a secret if the repository were made public right now?

## Hint 2 (Syntax)
CI secret stores are read through an expression, not a literal. On GitHub Actions the form is:

```
${{ secrets.NAME_OF_SECRET }}
```

The validator deliberately skips any value containing `${{` — that is the signal that the value is late-bound rather than baked in. Keep the environment variable name the test suite already expects; only the right-hand side changes.

## Hint 3 (Snippet)
The offending step sets its token inline:

```yaml
      - name: Run Playwright suite
        run: npx playwright test --reporter=json
        env:
          CRUCIBLE_TOKEN: "ghp_C4f81a2Kx9QwErTyUiOpAsDfGhJkLzXcVbNm"
```

Replace the literal with the store reference:

```yaml
      - name: Run Playwright suite
        run: npx playwright test --reporter=json
        env:
          CRUCIBLE_TOKEN: ${{ secrets.CRUCIBLE_TOKEN }}
```

In a real repository there is a mandatory second step the validator cannot check for you: **rotate the leaked token**. Once a credential has been pushed, it is compromised whether or not the commit was amended.
