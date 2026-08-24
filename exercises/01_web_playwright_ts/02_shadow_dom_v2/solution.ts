import { test, expect } from '@playwright/test';

test('pierce shadow dom to unlock vault and verify secret token', async ({ page }) => {
  await page.goto('/shadow-dom');

  // Scope locator to custom element host
  const vault = page.locator('chaos-vault');

  // Playwright selector engine natively traverses shadow roots with CSS & testids
  const secret = vault.locator('[data-testid="vault-secret"]');
  await expect(secret).toBeVisible();
  await expect(secret).toHaveText('CHERENKOV_SECRET_9876');

  // Click unlock button inside shadow root
  await vault.getByRole('button', { name: 'Unlock' }).click();

  // Verify updated status badge
  const status = vault.locator('[data-testid="vault-status"]');
  await expect(status).toHaveText('Unlocked');
});
