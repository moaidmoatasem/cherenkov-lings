/**
 * PRODUCTION STORY:
 * Shopify / Stripe 3D Secure Frame Drop (2020)
 * During a major merchant migration, payment integration smoke tests failed intermittently because
 * automated scripts attempted to locate credit card fields in the top-level document instead of scoping
 * into sandboxed, cross-origin 3D Secure iframes.
 */

import { test, expect } from '@playwright/test';

// Anti-pattern: Attempting to query elements inside an iframe using top-level page locators
// TODO: Use page.frameLocator() to target elements inside the sandboxed payment iframe

test('submit payment through sandboxed payment gateway (BRITTLE)', async ({ page }) => {
  await page.goto('/payment');

  // Fails because the input is inside an iframe (e.g. <iframe id="stripe-frame" ...>)
  const cardInput = page.locator('#card-number');
  await cardInput.fill('4242424242424242');

  const expiryInput = page.locator('#card-expiry');
  await expiryInput.fill('12/28');

  const submitButton = page.locator('#btn-submit-payment');
  await submitButton.click();

  await expect(page.locator('#payment-status')).toHaveText('Payment Authorized');
});
