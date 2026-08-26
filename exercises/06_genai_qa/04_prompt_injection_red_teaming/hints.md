## Hint 1 (Architectural Nudge)
Prompt injection occurs when untrusted user input alters the developer-defined system role and instructions.

## Hint 2 (API Pattern)
Assert on HTTP 400 Bad Request and verify `data.error === "PROMPT_INJECTION_DETECTED"`.

## Hint 3 (Code Diff)
```diff
- expect(res.status()).toBeLessThan(500);
+ expect(res.status()).toBe(400);
+ expect(data.error).toBe('PROMPT_INJECTION_DETECTED');
```
