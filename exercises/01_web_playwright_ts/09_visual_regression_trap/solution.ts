import { test, expect } from '@playwright/test';

/**
 * SDET Resilient Pattern: Masked Visual Regression with Tolerance
 * Real-world UIs frequently contain dynamic timestamps, user badges, or animations.
 * Applying masks to volatile regions and setting reasonable pixel diff tolerances
 * prevents false-positive visual test breaks while preserving layout regression catching.
 */
test('verify dashboard visual layout with masks and tolerance (RESILIENT)', async ({ page }) => {
  await page.goto('http://localhost:8080/dashboard');

  // Identify volatile dynamic locators
  const dynamicClock = page.getByTestId('live-clock').or(page.locator('.timestamp'));
  const sessionBadge = page.getByTestId('session-id').or(page.locator('.badge-session'));
  const chaosIndicator = page.getByTestId('chaos-status').or(page.locator('.status-pill'));

  // Resilient visual comparison with masks and 5% pixel diff threshold
  await expect(page).toHaveScreenshot('dashboard-baseline.png', {
    maxDiffPixelRatio: 0.05,
    mask: [dynamicClock, sessionBadge, chaosIndicator],
    animations: 'disabled',
  });
});
