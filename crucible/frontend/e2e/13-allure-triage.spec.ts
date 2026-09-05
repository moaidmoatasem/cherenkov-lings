import { test, expect } from '@playwright/test';

test.describe('Allure & Triage Drill — Root-Cause Triage', () => {
  test.beforeEach(async ({ page }) => {
    await page.goto('/allure-triage');
  });

  test('allure triage page loads', async ({ page }) => {
    await expect(page.locator('text=Enterprise Allure Reports')).toBeVisible();
  });

  test('test results table loads', async ({ page }) => {
    await expect(page.locator('text=test_checkout_with_ssr_hydration_delay')).toBeVisible();
  });

  test('category filter works', async ({ page }) => {
    await page.click('button:has-text("ProductBug")');
    await expect(page.locator('text=test_inventory_balance_underflow_race')).toBeVisible();
  });

  test('triage submission grades hypothesis', async ({ page }) => {
    await page.fill('input[placeholder="Test ID"]', 'tc-01');
    await page.click('button:has-text("Submit Triage")');
    await expect(page.locator('text=Score Awarded')).toBeVisible();
  });

  test('ground truth explanation displays', async ({ page }) => {
    await expect(page.locator('text=groundTruthExplanation')).toBeVisible();
  });
});
