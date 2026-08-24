/**
 * PRODUCTION STORY:
 * Twitter / X Search Autocomplete Bug (2020)
 * Fast typing triggered multiple concurrent debounced autocomplete requests. A slow initial request ('p')
 * resolved after a fast subsequent request ('playwright'), overwriting the UI with obsolete stale data.
 */

import { test, expect } from '@playwright/test';

test('debounced search displays accurate results without out-of-order race', async ({ page }) => {
  await page.goto('/search');

  const searchInput = page.getByRole('textbox', { name: /search/i });

  // Type search query
  await searchInput.fill('playwright');

  // Anti-pattern: Fixed short sleep that races with out-of-order responses
  // TODO: Replace sleep with deterministic state verification on active-query
  await page.waitForTimeout(600);

  const firstResult = page.locator('[data-testid="result-item"]').first();
  // Fails when delayed 'p' response arrives at ~500ms and overwrites the list with non-Playwright items
  await expect(firstResult).toHaveText('Playwright', { timeout: 1000 });
});
