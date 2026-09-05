/**
 * PRODUCTION STORY:
 * Silent Dynamic UI Updates (2020)
 * An airline seat selection tool updated pricing dynamically without ARIA live regions,
 * leaving screen reader users unaware of price changes upon selecting seats.
 */
import { test, expect } from '@playwright/test';

// Anti-pattern: Asserting text change without verifying ARIA live region notification
// or running an axe-core scan on the mutated DOM.
// TODO: Assert that status updates are announced via role="status", then run
// AxeBuilder against the status region to catch broken/invalid ARIA wiring.
test('verifies transfer message text', async ({ page }) => {
  await page.goto('/transfer');
  await page.click('#transfer-btn');

  // Brittle check: Checks visual text only — never runs axe-core, so a refactor
  // that drops role="status" (silencing screen readers) would still pass this.
  const status = page.locator('#transfer-status');
  await expect(status).toBeAttached();
});
