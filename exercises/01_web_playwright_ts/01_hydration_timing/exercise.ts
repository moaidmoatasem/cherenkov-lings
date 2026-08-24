import { test, expect } from '@playwright/test';

test('checkout completes successfully despite hydration delay', async ({ page }) => {
  await page.goto('/checkout');

  // Anti-pattern: Hardcoded sleep that is too short for hydration under load
  // TODO: Fix the flaky sleep with a web-first assertion waiting for data-hydrated="true"
  await page.waitForTimeout(200);

  // Naive click on unhydrated button drops the event
  await page.locator('#checkout-btn').click();

  // Assertion fails because click was dropped before event listeners attached
  await expect(page.getByTestId('order-status')).toHaveText('Order Confirmed', { timeout: 3000 });
});
