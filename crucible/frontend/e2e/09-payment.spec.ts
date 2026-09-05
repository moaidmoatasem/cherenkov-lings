import { test, expect } from '@playwright/test';

test.describe('Payment Drill — Iframe Boundary', () => {
  test.beforeEach(async ({ page }) => {
    await page.goto('/payment');
  });

  test('payment page loads with iframe', async ({ page }) => {
    await expect(page.locator('data-testid=payment-page')).toBeVisible();
    const frame = page.frameLocator('iframe[name="payment-gateway"]');
    await expect(frame.locator('#secure-card-pin')).toBeVisible();
  });

  test('iframe authorize button works', async ({ page }) => {
    const frame = page.frameLocator('iframe[name="payment-gateway"]');
    await frame.fill('#secure-card-pin', '1234');
    await frame.click('#btn-authorize');
    await expect(frame.locator('#frame-auth-status')).toHaveText('Payment Authorized');
  });

  test('iframe rejects empty pin', async ({ page }) => {
    const frame = page.frameLocator('iframe[name="payment-gateway"]');
    await frame.click('#btn-authorize');
    await expect(frame.locator('#frame-auth-status')).not.toBeVisible();
  });
});
