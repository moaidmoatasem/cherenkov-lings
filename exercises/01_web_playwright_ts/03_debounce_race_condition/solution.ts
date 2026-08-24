import { test, expect } from '@playwright/test';

test('debounced search displays accurate results without out-of-order race', async ({ page }) => {
  await page.goto('/search');

  const searchInput = page.getByRole('textbox', { name: /search/i });

  // Type search query
  await searchInput.fill('playwright');

  // Deterministic synchronization: Wait until the active-query tag matches our search query
  await expect(page.getByTestId('active-query')).toHaveText('playwright');

  // Assert that search results contain Playwright courses
  const results = page.getByTestId('search-results');
  await expect(results.getByTestId('result-item').first()).toContainText('Playwright');
});
