/**
 * PRODUCTION STORY:
 * Amazon Prime Day Checkout Outage (2018)
 * Server-Side Rendered (SSR) HTML was visible on the screen, but React/Vue hydration had not yet
 * attached event listeners. Clicks on the checkout button were dropped silently, costing millions in lost revenue.
 */

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
