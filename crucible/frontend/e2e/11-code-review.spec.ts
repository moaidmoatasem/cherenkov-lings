import { test, expect } from '@playwright/test';

test.describe('Code Review Drill — AST Review Engine', () => {
  test.beforeEach(async ({ page }) => {
    await page.goto('/code-review');
  });

  test('code review page loads', async ({ page }) => {
    await expect(page.locator('.code-review-page')).toBeVisible();
    await expect(page.locator('text=Code Review & Senior QA Mentor')).toBeVisible();
  });

  test('template selector shows 6 templates', async ({ page }) => {
    await expect(page.locator('.template-pill')).toHaveCount(6);
  });

  test('selecting template loads code', async ({ page }) => {
    await page.click('.template-pill:has-text("Fragile XPath")');
    const code = page.locator('.code-textarea');
    await expect(code).toContainText('xpath=');
  });

  test('local AST scanner detects violations', async ({ page }) => {
    await expect(page.locator('.violation-card').first()).toBeVisible();
    // Heading is singular/plural depending on violation count ("1 AST Violation
    // Identified" vs "N AST Violations Identified").
    await expect(page.locator('text=/AST Violations? Identified/')).toBeVisible();
  });

  test('score gauge renders', async ({ page }) => {
    await expect(page.locator('.gauge-score-value')).toBeVisible();
  });

  test('wizard walks through anti-pattern selection and fix preview', async ({ page }) => {
    // Select a template with violations so there's something to fix.
    await page.click('.template-pill:has-text("Fragile XPath")');
    await page.click('button:has-text("Fix-It Wizard")');
    await expect(page.locator('text=STEP 1')).toBeVisible();
    await page.locator('.vchip').first().click();
    await expect(page.locator('text=STEP 2')).toBeVisible();
    await expect(page.locator('text=Unified Code Diff Preview')).toBeVisible();
  });

  test('reset button restores original code', async ({ page }) => {
    await page.fill('.code-textarea', '// custom code');
    await page.click('.reset-code-btn');
    await expect(page.locator('.code-textarea')).toContainText('I AM NOT DONE');
  });

  test('view mode toggle works', async ({ page }) => {
    await page.click('[data-testid="view-mode-toggle"]');
    await expect(page.locator('[data-testid="native-mode-message"]')).toBeVisible();
  });
});
