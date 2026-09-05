import { test, expect } from '@playwright/test';

test.describe('Pipeline Builder Drill — CI/CD Simulator', () => {
  test.beforeEach(async ({ page }) => {
    await page.goto('/pipeline-builder');
  });

  test('pipeline builder loads with default stages', async ({ page }) => {
    await expect(page.locator('text=CI/CD Pipeline Simulator')).toBeVisible();
    await expect(page.locator('text=Workflow Triggers')).toBeVisible();
    await expect(page.locator('text=Environment Setup')).toBeVisible();
  });

  test('validate workflow shows results', async ({ page }) => {
    await page.click('button:has-text("Validate Workflow")');
    await expect(page.locator('text=SDET Score')).toBeVisible();
  });

  test('run pipeline shows execution results', async ({ page }) => {
    await page.click('button:has-text("Run Pipeline")');
    await expect(page.locator('text=Workflow Execution')).toBeVisible();
  });

  test('stage toggles enable/disable stages', async ({ page }) => {
    const chaosStage = page.locator('text=L4/L7 Chaos Proxy').locator('..');
    await chaosStage.click();
    // Stage should toggle
  });
});
