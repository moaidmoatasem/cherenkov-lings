import { test, expect } from '@playwright/test';

/**
 * The review page scanned code with five regex rules of its own while
 * /api/review and /api/review/fix — the engine the CLI uses, with nine rules, a
 * mentor critique and a real patcher — had no consumer. These pin the wiring
 * and, importantly, that wiring it did not cost detection: the templates are
 * named after the anti-patterns they demonstrate, so a template must never come
 * back clean.
 */
const runReview = async (page: import('@playwright/test').Page) => {
  await page.getByRole('button', { name: /Run .*(Review|Analysis)|Analyz/i }).first().click();
};

test.describe('Code review page is backed by the review engine', () => {
  test.beforeEach(async ({ page }) => {
    await page.goto('/code-review');
    await expect(page.locator('.ast-source')).toBeVisible();
  });

  test('names which scanner produced the violation list', async ({ page }) => {
    await expect(page.locator('.ast-source')).toContainText('local scan');
    await runReview(page);
    await expect(page.locator('.ast-source')).toHaveText('review engine');
  });

  test('the engine finds the anti-pattern each template is named after', async ({ page }) => {
    // The fragile-locator template: absolute XPath, a class stack, and an
    // xpath= prefixed selector. The engine used to see only the first.
    await page.getByText('Checkout Form: Fragile XPath & Deep CSS').click();
    await runReview(page);
    await expect(page.locator('.ast-source')).toHaveText('review engine');
    expect(await page.locator('.violation-card').count()).toBeGreaterThanOrEqual(3);
  });

  test('an unsafe unwrap does not score a clean bill of health', async ({ page }) => {
    await page.getByText('Kafka Consumer: Unsafe Unwrap').click();
    await runReview(page);
    await expect(page.locator('.violation-card')).toHaveCount(1);
    // The regression this pins: the engine had no TypeScript unwrap rule, so
    // the drill named for it came back 100/100.
    await expect(page.locator('.ast-source')).toHaveText('review engine');
  });

  test('the clean reference template still passes', async ({ page }) => {
    await page.getByText('Clean Reference Solution').click();
    await runReview(page);
    await expect(page.locator('.violation-card')).toHaveCount(0);
  });

  test('applying a fix patches through the engine and invalidates the verdict', async ({
    page,
  }) => {
    await page.getByText('Hydration Timing: Hardcoded Sleep').click();
    const editor = page.locator('textarea').first();
    await expect(editor).toContainText('waitForTimeout');

    await runReview(page);
    await expect(page.locator('.ast-source')).toHaveText('review engine');

    await page.locator('.apply-direct-btn').first().click();
    await expect(editor).not.toContainText('waitForTimeout');
    // A score describes the code it was asked about, so editing must drop it.
    await expect(page.locator('.ast-source')).toContainText('local scan');
  });
});
