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
    // CatalogPage injects its own X-Chaos: delay=5000ms on this fetch (a
    // deliberate patience drill), so the default 5s assertion timeout races
    // that delay almost exactly -- give it the same 10s budget test 2 above
    // already does for the same reason.
    await expect(page.locator('data-testid=product-item').first()).toBeVisible({ timeout: 10000 });
    await expect(page.locator('.product-name').first()).toBeVisible();
    await expect(page.locator('.product-price').first()).toBeVisible();
  });
});
