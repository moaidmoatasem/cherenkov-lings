import { test, expect } from '@playwright/test';

test.describe('Navigation & Routing', () => {
  test.beforeEach(async ({ page }) => {
    await page.goto('/sandbox');
  });

  test('sandbox home page loads with drill cards', async ({ page }) => {
    await expect(page).toHaveTitle('Cherenkov Micro-Crucible Target Sandbox');
    await expect(page.locator('data-testid=home-page')).toBeVisible();
    // Verify key drill cards are present
    await expect(page.locator('[data-testid="card-01_hydration_timing"]')).toBeVisible();
    await expect(page.locator('[data-testid="card-02_shadow_dom_v2"]')).toBeVisible();
    await expect(page.locator('[data-testid="card-03_debounce_race_condition"]')).toBeVisible();
    await expect(page.locator('[data-testid="card-transfer_kafka_lag"]')).toBeVisible();
    await expect(page.locator('[data-testid="card-pipeline_builder"]')).toBeVisible();
    await expect(page.locator('[data-testid="card-allure_triage"]')).toBeVisible();
  });

  test('sandbox overview shows correct title', async ({ page }) => {
    await expect(page.locator('text=Micro-Crucible Target Sandbox')).toBeVisible();
  });

  test('learn page loads with today screen', async ({ page }) => {
    await page.goto('/learn');
    await expect(page.locator('.learn-root')).toBeVisible();
  });

  test('root path loads learn app', async ({ page }) => {
    await page.goto('/');
    await expect(page.locator('.learn-root')).toBeVisible();
  });

  test('drill cards navigate to correct routes', async ({ page }) => {
    await page.goto('/sandbox');
    await page.click('[data-testid="btn-goto-01_hydration_timing"]');
    await expect(page).toHaveURL('/checkout');
    
    await page.goto('/sandbox');
    await page.click('[data-testid="btn-goto-02_shadow_dom_v2"]');
    await expect(page).toHaveURL('/shadow-dom');
    
    await page.goto('/sandbox');
    await page.click('[data-testid="btn-goto-transfer_kafka_lag"]');
    await expect(page).toHaveURL('/transfer');
  });

  test('mission control button on home works', async ({ page }) => {
    await page.goto('/sandbox');
    await page.click('button:has-text("Open Mission Control")');
    await expect(page).toHaveURL('/mission-control');
  });

  test('unknown route shows not-found page', async ({ page }) => {
    await page.goto('/unknown-route');
    await expect(page.locator('data-testid=not-found')).toBeVisible();
  });
});
