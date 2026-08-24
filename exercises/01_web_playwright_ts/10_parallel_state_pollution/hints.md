# Hints: Drill 10 - Parallel State Isolation

## Hint 1 (Architectural Nudge)
Tests running in parallel must not share mutable global state (such as global tokens or single static accounts). State mutation in one test worker will pollute the environment of another.

## Hint 2 (API Pattern)
Use Playwright's `browser.newContext({ storageState: ... })` or separate test worker projects with distinct `storageState` JSON files to ensure isolation.

## Hint 3 (Code Diff)
```diff
- let globalUserToken = 'shared_token';
- await page.evaluate((token) => localStorage.setItem('auth_token', token), globalUserToken);
+ const context = await browser.newContext({
+   storageState: {
+     cookies: [{ name: 'session_id', value: `sess_worker_${workerIndex}`, domain: 'localhost', path: '/' }],
+     origins: [{ origin: 'http://localhost:8080', localStorage: [{ name: 'auth_token', value: `token_${workerIndex}` }] }],
+   },
+ });
```
