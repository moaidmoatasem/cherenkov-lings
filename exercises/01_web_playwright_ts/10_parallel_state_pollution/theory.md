# Theoretical Context: Parallel State Pollution & Browser Context Isolation

## Production Incident: GitLab Multi-Tenant Storage State Leak (2018)

In 2018, GitLab expanded its CI/CD testing infrastructure to execute hundreds of end-to-end integration tests concurrently across distributed worker threads to reduce test execution times. Shortly after enabling high concurrency, tests began failing with intermittent authentication errors, permission denied exceptions, and data corruption in test tenant accounts. Post-incident triage revealed that parallel test workers shared a single global browser storage state (persisting cookies, `localStorage`, and `sessionStorage` across workers). When Worker A logged in as an administrator to delete a project, Worker B—executing a read-only viewer test simultaneously—inherited Worker A's mutated session token, leading to race-condition state overwrites and cross-tenant data leaks during CI test runs.

## The Underlying Mechanism

Modern test runners like Playwright execute tests across multiple parallel worker processes to maximize CPU utilization and minimize CI runtime:

1. **Shared Browser State Vulnerability**: If multiple parallel tests share a single `BrowserContext` or write to a global shared storage state file, mutations performed by one test (e.g., logging in, clearing cart, modifying preferences) immediately alter the environment for concurrent tests.
2. **Playwright `BrowserContext` Isolation**: A `BrowserContext` is an isolated incognito-equivalent profile. Each context operates with completely segregated cookies, local storage, indexedDB, and cache.
3. **Per-Worker `storageState` Authentication**: To avoid slow, redundant UI logins before every test while maintaining strict isolation, Playwright supports saving authenticated session states to disk (e.g., `userStorageState.json`) and initializing independent browser contexts per worker using `test.use({ storageState: ... })`.

```
[Shared Context Collision vs. Isolated Per-Worker Contexts]
Shared Context (COLLISION):
Worker 1 (Admin Login) ────┐
                           ├──> [ Global Cookies & LocalStorage ] ──> MUTATION COLLISION!
Worker 2 (Viewer Test) ────┘

Isolated Contexts:
Worker 1 ──> [ BrowserContext 1 (Admin Session)  ] ──> Test Passes Cleanly
Worker 2 ──> [ BrowserContext 2 (Viewer Session) ] ──> Test Passes Cleanly
```

Enforcing strict browser context isolation eliminates parallel state pollution, enabling massive concurrency without test flakiness or cross-test contamination.

You will now simulate this in the Crucible: isolate parallel test worker sessions using dedicated browser contexts and isolated storage states.
