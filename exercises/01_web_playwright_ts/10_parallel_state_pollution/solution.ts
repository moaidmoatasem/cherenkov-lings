import { test, expect } from '@playwright/test';

// storageState seeds localStorage per ORIGIN, and unlike page.goto() Playwright
// does not resolve that origin against baseURL. It has to be spelled out, or the
// token is silently never applied and the test fails for a reason that looks
// like anything but this.
const APP_ORIGIN = process.env.PLAYWRIGHT_BASE_URL ?? 'http://localhost:8080';

/**
 * SDET Resilient Pattern: Per-Worker Context and State Isolation
 * Isolates each parallel test worker with dedicated user credentials,
 * independent browser contexts, and unique storage state tokens.
 */
test.describe.parallel('User Profile Mutations (ISOLATED WORKER SESSIONS)', () => {
  test('worker A updates username to Alice with isolated context', async ({ browser }) => {
    const context = await browser.newContext({
      storageState: {
        cookies: [{ name: 'session_id', value: 'sess_worker_alice', domain: 'localhost', path: '/', expires: -1, httpOnly: false, secure: false, sameSite: 'Lax' }],
        origins: [{ origin: APP_ORIGIN, localStorage: [{ name: 'auth_token', value: 'token_alice_123' }] }],
      },
    });
    const page = await context.newPage();

    await page.goto('/profile');
    await page.locator('#username').fill('Alice');
    await page.locator('#save-profile').click();

    await expect(page.locator('#display-name')).toHaveText('Alice', { timeout: 5000 });
    await context.close();
  });

  test('worker B updates username to Bob with isolated context', async ({ browser }) => {
    const context = await browser.newContext({
      storageState: {
        cookies: [{ name: 'session_id', value: 'sess_worker_bob', domain: 'localhost', path: '/', expires: -1, httpOnly: false, secure: false, sameSite: 'Lax' }],
        origins: [{ origin: APP_ORIGIN, localStorage: [{ name: 'auth_token', value: 'token_bob_456' }] }],
      },
    });
    const page = await context.newPage();

    await page.goto('/profile');
    await page.locator('#username').fill('Bob');
    await page.locator('#save-profile').click();

    await expect(page.locator('#display-name')).toHaveText('Bob', { timeout: 5000 });
    await context.close();
  });
});
