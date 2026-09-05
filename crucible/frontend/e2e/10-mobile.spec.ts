import { test, expect } from '@playwright/test';

test.describe('Mobile Test Drill — Biometric Fallback', () => {
  test.beforeEach(async ({ page }) => {
    await page.goto('/mobile-test');
  });

  test('mobile page loads with home screen', async ({ page }) => {
    await expect(page.locator('data-testid=login-biometric')).toBeVisible();
    await expect(page.locator('data-testid=view-balance')).toBeVisible();
  });

  test('biometric flow falls back to PIN', async ({ page }) => {
    await page.click('#login-biometric');
    await expect(page.locator('text=Checking biometric availability')).toBeVisible();
    await expect(page.locator('text=Biometric unavailable')).toBeVisible({ timeout: 5000 });
    await expect(page.locator('data-testid=pin-input')).toBeVisible();
  });

  test('PIN submit authenticates user', async ({ page }) => {
    await page.click('#login-biometric');
    await expect(page.locator('data-testid=pin-input')).toBeVisible({ timeout: 5000 });
    await page.fill('#pin-input', '1234');
    await page.click('#pin-submit');
    await expect(page.locator('data-testid=welcome-message')).toHaveText('Welcome, SDET Engineer');
  });

  test('view balance shows correct amount', async ({ page }) => {
    await page.click('#view-balance');
    await expect(page.locator('data-testid=account-summary')).toBeVisible();
    await expect(page.locator('data-testid=account-balance')).toContainText('1000');
  });

  test('view products shows scrollable list', async ({ page }) => {
    await page.click('#view-products');
    await expect(page.locator('data-testid=product-list')).toBeVisible();
  });

  test('deep link navigates to balance', async ({ page }) => {
    await page.goto('/mobile-test#account=ACC-999');
    await expect(page.locator('data-testid=account-summary')).toBeVisible();
  });
});
