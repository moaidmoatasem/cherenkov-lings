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
    const frame = page.frameLocator('iframe[data-testid="payment-frame"]');
    await expect(frame.locator('#secure-card-pin')).toBeVisible();
    await expect(frame.locator('#btn-authorize')).toBeVisible();
  });

  test('payment iframe authorize button works', async ({ page }) => {
    const frame = page.frameLocator('iframe[data-testid="payment-frame"]');
    await frame.locator('#secure-card-pin').fill('1234');
    await frame.locator('#btn-authorize').click();
    await expect(frame.locator('#frame-auth-status')).toHaveText('Payment Authorized');
  });

  test('inspector shows expected secret token', async ({ page }) => {
    await expect(page.getByTestId('vault-secret')).toHaveText('CHERENKOV_SECRET_9876');
  });
});
