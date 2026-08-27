/**
 * PRODUCTION STORY:
 * Keyboard Trap Lockout on Modal Dialog (2021)
 * A healthcare portal launched an unescapeable modal dialog where keyboard Tab focus
 * could not cycle out, blocking keyboard-only navigation users.
 */
import { test, expect } from '@playwright/test';

// Anti-pattern: Clicking directly with mouse without testing Tab key focus sequence
// TODO: Assert sequential keyboard Tab navigation across interactive form fields
test('tests form without keyboard navigation', async ({ page }) => {
  await page.goto('/transfer');
  await page.fill('#recipient', 'ACC-002');
  await page.click('#transfer-btn');
});
