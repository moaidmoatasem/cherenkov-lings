import { test, expect } from '@playwright/test';

test.describe('Shadow DOM & Iframe Drill', () => {
  test.beforeEach(async ({ page }) => {
    await page.goto('/shadow-dom');
  });

  test('page loads with shadow dom content', async ({ page }) => {
    await expect(page.locator('data-testid=shadow-dom-page')).toBeVisible();
    await expect(page.locator('data-testid=vault-wrapper')).toBeVisible();
  });

  test('chaos-vault custom element exists', async ({ page }) => {
    const vault = page.locator('chaos-vault');
    await expect(vault).toBeVisible();
  });

  test('payment frame iframe exists', async ({ page }) => {
    const frame = page.frameLocator('iframe[name="payment-gateway"]');
    await expect(frame.locator('#secure-card-pin')).toBeVisible();
    await expect(frame.locator('#btn-authorize')).toBeVisible();
  });

  test('payment iframe authorize button works', async ({ page }) => {
    const frame = page.frameLocator('iframe[name="payment-gateway"]');
    await frame.fill('#secure-card-pin', '1234');
    await frame.click('#btn-authorize');
    await expect(frame.locator('#frame-auth-status')).toHaveText('Payment Authorized');
  });

  test('inspector shows expected secret token', async ({ page }) => {
    await expect(page.locator('text=CHERENKOV_SECRET_9876')).toBeVisible();
  });
});
