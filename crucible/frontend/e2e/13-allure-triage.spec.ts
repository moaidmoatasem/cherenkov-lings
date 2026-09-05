import { test, expect } from '@playwright/test';

test.describe('Allure & Triage Drill — Root-Cause Triage', () => {
  test.beforeEach(async ({ page }) => {
    await page.goto('/allure-triage');
  });

  test('allure triage page loads', async ({ page }) => {
    await expect(page.getByRole('heading', { name: 'Enterprise Allure Reports & Triage Station' })).toBeVisible();
  });

  test('test results table loads', async ({ page }) => {
    await expect(page.locator('.name-text', { hasText: 'test_auth_role_privilege_escalation' })).toBeVisible();
  });

  test('category filter works', async ({ page }) => {
    // Root-cause filter is a <select>, not a button.
    await page.selectOption('select >> nth=1', { label: 'Product Bugs' });
    await expect(page.locator('.name-text', { hasText: 'test_auth_role_privilege_escalation' })).toBeVisible();
  });

  test('triage submission grades hypothesis', async ({ page }) => {
    await page.locator('button.triage-action-btn').first().click();
    await page.locator('.category-option-card', { hasText: 'Genuine Product Bug' }).click();
    const textareas = page.locator('#triage-challenge-section textarea');
    await textareas.nth(0).fill('The proxy returned 200 without an RBAC check on the role parameter.');
    await textareas.nth(1).fill('Add an RBAC middleware check on the role parameter.');
    await page.click('button.submit-hypothesis-btn');
    await expect(page.locator('text=XP Earned')).toBeVisible();
  });

  test('ground truth explanation displays', async ({ page }) => {
    await page.locator('button.triage-action-btn').first().click();
    await page.locator('.category-option-card', { hasText: 'Genuine Product Bug' }).click();
    const textareas = page.locator('#triage-challenge-section textarea');
    await textareas.nth(0).fill('The proxy returned 200 without an RBAC check on the role parameter.');
    await textareas.nth(1).fill('Add an RBAC middleware check on the role parameter.');
    await page.click('button.submit-hypothesis-btn');
    await expect(page.locator('text=Ground Truth Root-Cause')).toBeVisible();
  });
});
