import { test, expect } from '@playwright/test';
import AxeBuilder from '@axe-core/playwright';

test('asserts no keyboard-hazard markup and a correct Tab sequence', async ({ page }) => {
  await page.goto('/transfer');

  // axe-core's `tabindex` rule flags a positive tabindex value, the single most
  // common cause of a Tab order that no longer follows the visual/DOM layout.
  const axeResults = await new AxeBuilder({ page })
    .include('.transfer-form')
    .withRules(['tabindex'])
    .analyze();
  expect(axeResults.violations).toEqual([]);

  // axe-core cannot simulate key presses, so the actual sequence still needs a
  // behavioral check: confirm focus moves in the order a keyboard-only or
  // screen-reader user would experience.
  const recipientInput = page.getByLabel('Recipient Account ID:');
  await recipientInput.focus();
  await expect(recipientInput).toBeFocused();

  // Tab to Amount input
  await page.keyboard.press('Tab');
  const amountInput = page.getByLabel('Transfer Amount ($ USD):');
  await expect(amountInput).toBeFocused();
});
