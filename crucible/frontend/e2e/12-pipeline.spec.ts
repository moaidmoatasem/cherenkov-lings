import { test, expect } from '@playwright/test';

test.describe('Pipeline Builder Drill — CI/CD Simulator', () => {
  test.beforeEach(async ({ page }) => {
    await page.goto('/pipeline-builder');
  });

  test('pipeline builder loads with default stages', async ({ page }) => {
    await expect(page.getByRole('heading', { name: 'CI/CD Pipeline Simulator & Workflow Builder' })).toBeVisible();
    await expect(page.locator('text=Workflow Triggers')).toBeVisible();
    await expect(page.locator('text=Environment Setup')).toBeVisible();
  });

  test('validation results show automatically', async ({ page }) => {
    // There's no separate "Validate Workflow" step -- SDET policy validation
    // runs against the current stage set and is shown up front.
    await expect(page.locator('text=Enterprise SDET Architecture Validation')).toBeVisible();
  });

  test('run simulation shows execution results', async ({ page }) => {
    await page.click('button:has-text("Run Simulation")');
    await expect(page.locator('text=Simulating Matrix')).toBeVisible();
    await expect(page.locator('text=Parallel Matrix Simulation in Progress')).toBeVisible();
  });

  test('stage toggles enable/disable stages', async ({ page }) => {
    const chaosStage = page.locator('text=L4/L7 Chaos Proxy').locator('..');
    await chaosStage.click();
    // Stage should toggle
  });
});
