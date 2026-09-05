import { test, expect } from '@playwright/test';

test.describe('Payment Drill — Iframe Boundary', () => {
  test.beforeEach(async ({ page }) => {
    await page.goto('/payment');
  });

  test('payment page loads with iframe', async ({ page }) => {
    await expect(page.locator('data-testid=payment-page')).toBeVisible();
    // /payment's iframe serves the checkout-frame gateway (card number +
    // expiry), not the PIN-based frame from /shadow-dom -- see /embed/checkout-frame.
    const frame = page.frameLocator('iframe[name="payment-gateway"]');
    await expect(frame.locator('#card-number')).toBeVisible();
    await expect(frame.locator('#card-expiry')).toBeVisible();
  });

  test('iframe authorize button works', async ({ page }) => {
    const frame = page.frameLocator('iframe[name="payment-gateway"]');
    await frame.locator('#card-number').fill('4242424242424242');
    await frame.locator('#card-expiry').fill('12/28');
    await frame.locator('#btn-submit-payment').click();
    await expect(frame.locator('#payment-status')).toHaveText('Payment Authorized - Success');
  });

  test('iframe rejects invalid card details', async ({ page }) => {
    const frame = page.frameLocator('iframe[name="payment-gateway"]');
    await frame.locator('#btn-submit-payment').click();
    await expect(frame.locator('#payment-status')).toHaveText('Error: invalid card details');
  });
});
