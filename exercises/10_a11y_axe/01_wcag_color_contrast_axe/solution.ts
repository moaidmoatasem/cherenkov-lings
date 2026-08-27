import { test, expect } from '@playwright/test';

test('asserts button has accessible name, role, and passes semantic checks', async ({ page }) => {
  await page.goto('/checkout');

  // Semantic accessibility locator verifying role and accessible name
  const checkoutBtn = page.getByRole('button', { name: 'Confirm Purchase' });
  await expect(checkoutBtn).toBeVisible({ timeout: 5000 });
  await expect(checkoutBtn).toBeEnabled();
});
