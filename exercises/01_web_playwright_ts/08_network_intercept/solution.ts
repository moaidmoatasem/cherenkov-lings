import { test, expect } from '@playwright/test';

/**
 * SDET Resilient Pattern: Network Interception & API Stubbing
 * Using page.route() enables isolating UI validation from backend latency spikes,
 * downstream service outages, and third-party rate limits.
 */
test('display catalog items with network interception stub (RESILIENT)', async ({ page }) => {
  // Stub the products API endpoint before navigation
  await page.route('**/products*', async (route) => {
    // GOTCHA: route globs also match the SPA document navigation itself.
    // Without this guard the HTML document is replaced by raw stubbed JSON.
    if (route.request().resourceType() === 'document') {
      await route.continue();
      return;
    }
    await route.fulfill({
      status: 200,
      contentType: 'application/json',
      body: JSON.stringify({
        total: 3,
        page: 1,
        per_page: 10,
        total_pages: 1,
        products: [
          { id: 'prod-1', name: 'High-Speed CI Runner', price: 99.0 },
          { id: 'prod-2', name: 'Resilience Gateway', price: 149.0 },
          { id: 'prod-3', name: 'Chaos Proxy Pro', price: 199.0 },
        ],
      }),
    });
  });

  await page.goto('/products');

  const productList = page.getByTestId('product-item').or(page.locator('.product-card'));
  // Fast and deterministic response without external network dependence
  await expect(productList.first()).toBeVisible({ timeout: 5000 });
});
