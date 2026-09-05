/**
 * PRODUCTION STORY:
 * Keyboard Trap Lockout on Modal Dialog (2021)
 * A healthcare portal launched an unescapeable modal dialog where keyboard Tab focus
 * could not cycle out, blocking keyboard-only navigation users.
 */
import { test, expect } from '@playwright/test';

// Anti-pattern: Clicking directly with mouse, never checking Tab order or running
// an accessibility scan for keyboard-hazard markup (e.g. a positive tabindex).
// TODO: Add an AxeBuilder scan for the `tabindex` rule, then verify the real Tab
// key sequence with page.keyboard.press('Tab') / toBeFocused() — axe can't
// simulate key presses, so the sequence itself still needs a behavioral check.
test('tests form without keyboard navigation', async ({ page }) => {
  await page.goto('/transfer');
  await page.fill('#recipient-input', 'ACC-002');
  await page.click('#transfer-btn');
});
