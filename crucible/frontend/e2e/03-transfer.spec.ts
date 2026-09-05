import { test, expect } from '@playwright/test';

test.describe('Transfer Drill — Kafka Lag Simulation', () => {
  test.beforeEach(async ({ page }) => {
    await page.goto('/transfer');
    // Reset ledger before each test
    await page.click('#reset-ledger-btn');
  });

  test('balance loads on mount', async ({ page }) => {
    await expect(page.locator('data-testid=account-balance')).toHaveText('1000.00', { timeout: 5000 });
  });

  test('transfer submits and settles after kafka lag', async ({ page }) => {
    await page.fill('#amount-input', '250.00');
    await page.click('#transfer-btn');
    
    await expect(page.locator('data-testid=transfer-status')).toContainText('Transfer Queued');
    await expect(page.locator('data-testid=transfer-status')).toHaveText('Transfer Settled', { timeout: 10000 });
  });

  test('balance decreases after transfer settles', async ({ page }) => {
    await page.fill('#amount-input', '250.00');
    await page.click('#transfer-btn');
    await expect(page.locator('data-testid=transfer-status')).toHaveText('Transfer Settled', { timeout: 10000 });
    await expect(page.locator('data-testid=account-balance')).toHaveText('750.00', { timeout: 10000 });
  });

  test('transfer id is displayed', async ({ page }) => {
    await page.fill('#amount-input', '250.00');
    await page.click('#transfer-btn');
    await expect(page.locator('data-testid=transfer-status')).toContainText('TX-');
  });

  test('invalid amount shows error', async ({ page }) => {
    await page.fill('#amount-input', '-50');
    await page.click('#transfer-btn');
    await expect(page.locator('.alert-error')).toBeVisible();
  });

  test('reset ledger restores default balance', async ({ page }) => {
    await page.fill('#amount-input', '250.00');
    await page.click('#transfer-btn');
    await expect(page.locator('data-testid=transfer-status')).toHaveText('Transfer Settled', { timeout: 10000 });
    await page.click('#reset-ledger-btn');
    await expect(page.locator('data-testid=account-balance')).toHaveText('1000.00');
  });

  test('recipient field is editable', async ({ page }) => {
    await page.fill('#recipient-input', 'ACC-003');
    await expect(page.locator('#recipient-input')).toHaveValue('ACC-003');
  });
});
