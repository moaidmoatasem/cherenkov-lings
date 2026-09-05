import { test, expect } from '@playwright/test';

test.describe('Profile Drill — Storage Isolation', () => {
  test.beforeEach(async ({ page }) => {
    await page.goto('/profile');
  });

  test('profile page loads', async ({ page }) => {
    await expect(page.locator('data-testid=profile-page')).toBeVisible();
  });

  test('save profile updates display name', async ({ page }) => {
    await page.fill('#username', 'Alice');
    await page.click('#save-profile-btn');
    await expect(page.locator('data-testid=display-name')).toHaveText('Alice');
    await expect(page.locator('data-testid=save-confirmation')).toContainText('Profile saved');
  });

  test('empty username shows warning', async ({ page }) => {
    await page.click('#save-profile-btn');
    await expect(page.locator('data-testid=save-confirmation')).toContainText('Please enter a username');
  });

  test('localStorage persists username', async ({ page }) => {
    await page.fill('#username', 'Bob');
    await page.click('#save-profile-btn');
    await page.reload();
    await expect(page.locator('data-testid=display-name')).toHaveText('Bob');
  });
});
