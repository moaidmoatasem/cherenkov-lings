import { test, expect } from '@playwright/test';

test.describe('Search Drill — Debounced Autocomplete Race', () => {
  test.beforeEach(async ({ page }) => {
    await page.goto('/search');
  });

  test('search page loads with input', async ({ page }) => {
    await expect(page.locator('data-testid=search-page')).toBeVisible();
    await expect(page.locator('#search-box')).toBeVisible();
  });

  test('typing playwright shows results', async ({ page }) => {
    await page.fill('#search-box', 'playwright');
    await expect(page.locator('data-testid=search-results')).toHaveCount(5, { timeout: 5000 });
    await expect(page.locator('data-testid=active-query')).toHaveText('playwright');
  });

  test('empty search shows no results message', async ({ page }) => {
    await expect(page.locator('data-testid=no-results')).toContainText('Start typing');
  });

  test('request log tracks queries', async ({ page }) => {
    await page.fill('#search-box', 'playwright');
    await expect(page.locator('.log-list .log-item')).toHaveCount(1, { timeout: 5000 });
  });

  test('search input has autocomplete off', async ({ page }) => {
    await expect(page.locator('#search-box')).toHaveAttribute('autocomplete', 'off');
  });
});
