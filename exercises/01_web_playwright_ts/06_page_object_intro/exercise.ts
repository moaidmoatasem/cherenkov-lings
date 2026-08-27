/**
 * PRODUCTION STORY:
 * Spotify Desktop Web Refactor (2022)
 * When the core web player UI was refactored, over 800 automated test cases failed simultaneously
 * across 40 squads because CSS selectors were copy-pasted across individual tests rather than encapsulated in Page Object Models.
 */

import { test, expect } from '@playwright/test';

// Anti-pattern: Hardcoded, duplicated selector chains scattered throughout the test
// TODO: Refactor using a Page Object Model (POM) class that encapsulates locators and actions

test('complete checkout flow without page object encapsulation (BRITTLE)', async ({ page }) => {
  await page.goto('/checkout');

  // Step 1: Fill item info using brittle selector
  await page.locator('div.checkout-container input#item-id').fill('item-1');
  await page.locator('div.checkout-container input#quantity').fill('2');

  // Step 2: Select shipping and fill address using brittle selectors
  await page.locator('div.checkout-container select#shipping-type').selectOption('express');
  await page.locator('div.checkout-container input#address').fill('742 Evergreen Terrace');

  // Step 3: Click pay and verify status using raw selectors
  await page.locator('div.checkout-container button#pay-btn').click();
  await expect(page.locator('div.checkout-container div#order-status')).toHaveText('Order Confirmed', { timeout: 3000 });
});
