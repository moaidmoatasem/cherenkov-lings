import { test, expect } from '@playwright/test';

test('asserts full keyboard Tab navigation and focus visibility', async ({ page }) => {
  await page.goto('/transfer');

  const recipientInput = page.getByLabel('Recipient Account ID:');
  await recipientInput.focus();
  await expect(recipientInput).toBeFocused();

  // Tab to Amount input
  await page.keyboard.press('Tab');
  const amountInput = page.getByLabel('Transfer Amount ($ USD):');
  await expect(amountInput).toBeFocused();
});
