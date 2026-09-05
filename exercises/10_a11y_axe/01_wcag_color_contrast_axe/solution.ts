import { test, expect } from '@playwright/test';
import AxeBuilder from '@axe-core/playwright';

test('asserts button has accessible name, role, and passes an axe color-contrast scan', async ({ page }) => {
  await page.goto('/checkout');

  // Semantic accessibility locator verifying role and accessible name
  const checkoutBtn = page.getByRole('button', { name: 'Confirm Purchase' });
  await expect(checkoutBtn).toBeVisible({ timeout: 5000 });
  await expect(checkoutBtn).toBeEnabled();

  // axe-core's `color-contrast` rule computes the actual foreground/background
  // luminance ratio (WCAG 1.4.3) — something no role/name assertion can catch,
  // since a button can be perfectly semantic and still be unreadable.
  const results = await new AxeBuilder({ page })
    .include('#confirm-purchase')
    .withRules(['color-contrast'])
    .analyze();
  expect(results.violations).toEqual([]);
});
