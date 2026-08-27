import { test, expect } from '@playwright/test';

test('asserts status notification is exposed via accessible status role', async ({ page }) => {
  await page.goto('/transfer');

  const transferBtn = page.getByRole('button', { name: 'Submit Transfer' });
  await transferBtn.click();

  // Verify status is accessible via getByRole('status') or live region
  const statusBox = page.getByRole('status');
  await expect(statusBox).toBeVisible({ timeout: 5000 });
});
