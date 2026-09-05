/**
 * PRODUCTION STORY:
 * Target & Domino's ADA Accessibility Lawsuits (2019)
 * Major digital retail platforms faced federal ADA accessibility lawsuits when vision-impaired
 * users could not navigate low-contrast buttons and unlabeled form controls.
 */
import { test, expect } from '@playwright/test';

// Anti-pattern: Verifying only DOM presence — never runs an automated accessibility scan
// TODO: Use AxeBuilder from '@axe-core/playwright' to run axe-core's `color-contrast`
// rule against the checkout action buttons and assert the violations array is empty.
test('verifies checkout button accessibility', async ({ page }) => {
  await page.goto('/checkout');

  // Brittle check: Only asserts element is present in DOM — never runs axe-core,
  // so a genuinely low-contrast or mislabeled button would still pass this test.
  const btn = page.locator('#checkout-btn');
  await expect(btn).toBeAttached();
});
