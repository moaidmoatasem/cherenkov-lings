import { test, expect } from '@playwright/test';

test.describe('Tailored E2E Functional: CI/CD Pipeline Simulator & Matrix Runners', () => {
  test.beforeEach(async ({ page }) => {
    await page.goto('/pipeline-builder');
    await expect(page.getByRole('heading', { name: 'CI/CD Pipeline Simulator & Workflow Builder' })).toBeVisible();
  });

  test('Mode Navigation Tabs switch between Canvas, YAML Editor, and Simulation views', async ({ page }) => {
    // Check initial canvas tab
    await expect(page.locator('text=Pipeline Execution Stages')).toBeVisible();

    // Switch to YAML Editor tab
    await page.click('button:has-text("Live GitHub Actions YAML Editor")');
    await expect(page.locator('.yaml-editor-card')).toBeVisible();
    await expect(page.locator('.yaml-textarea')).toBeVisible();

    // Switch to Simulation tab
    await page.click('button:has-text("Virtual Runner Simulation")');
    await expect(page.locator('.simulation-tab-view')).toBeVisible();
    await expect(page.locator('text=Ready to Simulate Matrix Execution')).toBeVisible();
  });

  test('SDET Policy Validation dynamically warns and Auto-Fix heals missing chaos stage', async ({ page }) => {
    // Select Chaos Proxy stage and disable it via the inspector
    const chaosCard = page.locator('.stage-card', { hasText: 'L4/L7 Chaos Proxy' });
    await expect(chaosCard).toBeVisible();
    await chaosCard.click();

    const toggleBtn = page.locator('button.toggle-btn:has-text("Disable Stage")');
    await expect(toggleBtn).toBeVisible();
    await toggleBtn.click();

    // Verify SDET Validation alert appears warning about missing chaos injection
    await expect(page.locator('text=SDET-R03: No Chaos Fault Injection Step')).toBeVisible();

    // Click Auto-Fix to re-enable chaos proxy
    const autoFixBtn = page.locator('.issue-card', { hasText: 'SDET-R03' }).locator('button:has-text("Auto-Fix")');
    await autoFixBtn.click();

    // Verify toast appears and validation warning disappears
    await expect(page.locator('.pipeline-toast')).toBeVisible();
    await expect(page.locator('text=SDET-R03: No Chaos Fault Injection Step')).not.toBeVisible();
  });

  test('Bidirectional YAML synchronization updates code when stages change', async ({ page }) => {
    // Switch to YAML tab
    await page.click('button:has-text("Live GitHub Actions YAML Editor")');
    const textarea = page.locator('.yaml-textarea');
    await expect(textarea).toBeVisible();

    // Verify default YAML content includes matrix and chaos proxy
    const content = await textarea.inputValue();
    expect(content).toContain('Enterprise SDET CI/CD Matrix Pipeline');
    expect(content).toContain('sdet-chaos-matrix-test');
    expect(content).toContain('matrix:');

    // Click Regenerate button
    await page.click('button:has-text("Regenerate")');
    await expect(textarea).toBeVisible();
  });

  test('Matrix Simulation executes parallel virtual runners and streams live terminal logs', async ({ page }) => {
    // Trigger simulation
    await page.click('button.run-sim-btn');

    // Automatically transitions to Simulation tab with running state
    await expect(page.locator('text=Parallel Matrix Simulation in Progress')).toBeVisible();

    // Verify runner cards are rendered for the matrix combinations
    const runnerCards = page.locator('.runner-card');
    await expect(runnerCards.first()).toBeVisible({ timeout: 5000 });
    const count = await runnerCards.count();
    expect(count).toBeGreaterThan(0);

    // Verify live terminal logs card is rendered for selected runner
    const terminalCard = page.locator('.terminal-card');
    await expect(terminalCard).toBeVisible();
    await expect(page.locator('.terminal-log-output pre')).toBeVisible();

    // Click the second runner card to switch log stream
    if (count > 1) {
      await runnerCards.nth(1).click();
      await expect(runnerCards.nth(1)).toHaveClass(/active/);
    }

    // Wait for simulation progress to advance
    await expect(page.locator('.sim-progress-fill')).toBeVisible();
  });
});
