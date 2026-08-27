/**
 * PRODUCTION STORY:
 * GitLab Multi-Tenant Storage Leak (2018)
 * Parallel CI test workers sharing a global browser profile and session storage concurrently modified
 * user settings, causing cross-worker state pollution and intermittent failures across test suites.
 */

import { test, expect } from '@playwright/test';

// Anti-pattern: Shared global state mutated concurrently across parallel test runs
// TODO: Isolate browser context and session state per worker using storageState or fresh context fixtures

let globalUserToken = 'shared_admin_token_default';

test.describe.parallel('User Profile Mutations (SHARED STATE POLLUTION)', () => {
  test('worker A updates username to Alice', async ({ page }) => {
    await page.goto('/profile');
    // Uses shared mutable global state
    await page.evaluate((token) => localStorage.setItem('auth_token', token), globalUserToken);

    await page.fill('#username', 'Alice');
    await page.click('#save-profile');
    globalUserToken = 'token_alice_updated';

    await expect(page.locator('#display-name')).toHaveText('Alice');
  });

  test('worker B updates username to Bob', async ({ page }) => {
    await page.goto('/profile');
    // Collides with worker A's token update
    await page.evaluate((token) => localStorage.setItem('auth_token', token), globalUserToken);

    await page.fill('#username', 'Bob');
    await page.click('#save-profile');

    await expect(page.locator('#display-name')).toHaveText('Bob');
  });
});
