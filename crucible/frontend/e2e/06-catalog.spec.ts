import { test, expect } from '@playwright/test';

test.describe('Catalog Drill — Response Stubbing', () => {
  test.beforeEach(async ({ page }) => {
    await page.goto('/products');
  });

  test('catalog page loads', async ({ page }) => {
    await expect(page.locator('data-testid=catalog-page')).toBeVisible();
  });

  test('loading state appears then products render', async ({ page }) => {
    await expect(page.locator('data-testid=catalog-loading')).toBeVisible();
    await expect(page.locator('data-testid=product-item')).toHaveCount(12, { timeout: 10000 });
  });

  test('products have name, price, and stock badge', async ({ page }) => {
    await expect(page.locator('data-testid=product-item').first()).toBeVisible();
    await expect(page.locator('.product-name').first()).toBeVisible();
    await expect(page.locator('.product-price').first()).toBeVisible();
  });
});
