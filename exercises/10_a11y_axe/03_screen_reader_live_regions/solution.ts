import { test, expect } from '@playwright/test';
import AxeBuilder from '@axe-core/playwright';

test('asserts status notification is exposed via an accessible, axe-clean live region', async ({ page }) => {
  await page.goto('/transfer');

  const transferBtn = page.getByRole('button', { name: 'Submit Transfer' });
  await transferBtn.click();

  // Verify status is accessible via getByRole('status') — the implicit ARIA
  // live region role that gets dynamic text announced to screen readers.
  const statusBox = page.getByRole('status');
  await expect(statusBox).toBeVisible({ timeout: 5000 });

  // axe-core can't detect a *missing* live region (that's an omission, not a DOM
  // violation) but it does validate that the ARIA markup actually present is
  // well-formed — catching regressions where a refactor swaps role="status" for
  // a plain <div> or otherwise breaks its ARIA wiring.
  const results = await new AxeBuilder({ page }).include('.transfer-status-box').analyze();
  expect(results.violations).toEqual([]);
});
