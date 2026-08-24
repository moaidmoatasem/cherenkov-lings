/**
 * PRODUCTION STORY:
 * Salesforce Lightning Component Lockup (2019)
 * Automated regression test suites failed across thousands of customer orgs because tests relied
 * on absolute XPath selectors that could not traverse Web Component Shadow DOM encapsulation boundaries.
 */

import { test, expect } from '@playwright/test';

test('pierce shadow dom to unlock vault and verify secret token', async ({ page }) => {
  await page.goto('/shadow-dom');

  // Anti-pattern: Absolute XPath breaks across DOM restructuring and fails to resolve across shadow roots
  // TODO: Replace fragile absolute XPath with Playwright locators that pierce shadow boundaries
  const secretElement = page.locator('xpath=/html/body/div/div/div/chaos-vault/div/span[2]');
  await expect(secretElement).toHaveText('CHERENKOV_SECRET_9876', { timeout: 2000 });

  const unlockBtn = page.locator('xpath=/html/body/div/div/div/chaos-vault/div/button');
  await unlockBtn.click({ timeout: 2000 });

  const statusElement = page.locator('xpath=/html/body/div/div/div/chaos-vault/div/span[3]');
  await expect(statusElement).toHaveText('Unlocked', { timeout: 2000 });
});
