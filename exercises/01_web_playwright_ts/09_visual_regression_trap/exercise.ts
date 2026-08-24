/**
 * PRODUCTION STORY:
 * Airbnb Daylight Saving Snapshot Glitch (2019)
 * Over 3,000 CI visual regression test suites failed simultaneously on a Monday morning because a daylight
 * saving time clock shift rendered dynamic timestamp text differently, failing rigid 0-tolerance pixel diffs.
 */

import { test, expect } from '@playwright/test';

// Anti-pattern: Visual regression testing on dynamic UI elements with zero tolerance and no masks
// TODO: Add maxDiffPixelRatio tolerance and mask dynamic elements (clocks, badges, session tokens)

test('verify dashboard visual layout without masks (FLAWED)', async ({ page }) => {
  await page.goto('http://localhost:8080/dashboard');

  // Anti-pattern: Strict 0% diff comparison fails when live clock or dynamic session ID updates
  await expect(page).toHaveScreenshot('dashboard-baseline.png');
});
