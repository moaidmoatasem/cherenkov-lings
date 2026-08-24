/**
 * PRODUCTION STORY:
 * Robinhood Real-Time Market Data Lag (2020)
 * Automated integration test suites locked up for 6 hours because downstream third-party market quote APIs
 * suffered a 100x latency spike (from 50ms to 5,000ms), causing CI workers to exceed timeout thresholds.
 */

import { test, expect } from '@playwright/test';

// Anti-pattern: Relying on un-stubbed, fluctuating external network endpoints in UI integration tests
// TODO: Intercept the network route with page.route() and stub the slow API response with deterministic mock data

test('display catalog items from slow backend (FLAWED)', async ({ page }) => {
  // Test attempts to load products from a slow endpoint that takes 4000ms+ or fails under chaos
  await page.goto('http://localhost:8080/products');

  // Short timeout fails because network is delayed
  const productList = page.getByTestId('product-item');
  await expect(productList.first()).toBeVisible({ timeout: 1500 });
  await expect(productList).toHaveCount(3);
});
