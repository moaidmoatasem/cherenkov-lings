import { test, expect } from '@playwright/test';

test.describe('Dashboard Drill — Visual Regression', () => {
  test.beforeEach(async ({ page }) => {
    await page.goto('/dashboard');
  });

  test('dashboard loads with all widgets', async ({ page }) => {
    await expect(page.locator('data-testid=dashboard-page')).toBeVisible();
    await expect(page.locator('data-testid=live-clock')).toBeVisible();
    await expect(page.locator('data-testid=session-id')).toBeVisible();
    await expect(page.locator('data-testid=chaos-status')).toBeVisible();
  });

  test('service table renders 4 services', async ({ page }) => {
    await expect(page.locator('data-testid=service-table')).toBeVisible();
    await expect(page.locator('[data-testid="service-table"] tbody tr')).toHaveCount(4);
  });

  test('stats cards show correct values', async ({ page }) => {
    await expect(page.locator('.stat-card').first()).toBeVisible();
    await expect(page.locator('text=Requests (24h)')).toBeVisible();
  });

  test('clock updates every second', async ({ page }) => {
    const clock1 = await page.locator('data-testid=live-clock').textContent();
    await page.waitForTimeout(1500);
    const clock2 = await page.locator('data-testid=live-clock').textContent();
    expect(clock1).not.toBe(clock2);
  });
});
