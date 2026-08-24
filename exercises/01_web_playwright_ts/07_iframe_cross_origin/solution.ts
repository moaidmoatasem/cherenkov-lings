import { test, expect } from '@playwright/test';

/**
 * SDET Resilient Pattern: frameLocator for Sandboxed / Cross-Origin Iframes
 * Playwright's frameLocator automatically pierces iframe boundaries and handles
 * frame reloading, lifecycle events, and cross-origin security contexts.
 */
test('submit payment through sandboxed payment gateway (RESILIENT)', async ({ page }) => {
  await page.goto('http://localhost:8080/payment');

  // Scope locators to the payment iframe
  const paymentFrame = page.frameLocator('iframe[name="payment-gateway"], iframe#stripe-frame, iframe.payment-frame');

  const cardInput = paymentFrame.getByLabel(/card number/i).or(paymentFrame.locator('#card-number'));
  const expiryInput = paymentFrame.getByLabel(/expiry/i).or(paymentFrame.locator('#card-expiry'));
  const submitButton = paymentFrame.getByRole('button', { name: /submit|pay/i }).or(paymentFrame.locator('#btn-submit-payment'));

  await cardInput.fill('4242424242424242');
  await expiryInput.fill('12/28');
  await submitButton.click();

  // Assert status inside frame or top-level parent page
  const statusLocator = paymentFrame.locator('#payment-status').or(page.locator('#payment-status'));
  await expect(statusLocator).toContainText(/Authorized|Success|Confirmed/i, { timeout: 5000 });
});
