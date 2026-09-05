import { test, expect } from '@playwright/test';

// These three assert the exact state of an 800ms setTimeout-driven hydration
// flag. Under real wall-clock time that's a genuine race against test-runner
// scheduling delay (navigation + assertion overhead can eat into the 800ms
// window, especially with multiple parallel workers contending for CPU),
// which made them flaky under `--workers=2`+ even though the underlying
// behavior was correct. Installing Playwright's clock lets us control time
// deterministically instead of racing it.
test.describe('Checkout Drill — Hydration Timing Gap (clock-controlled)', () => {
  test.beforeEach(async ({ page }) => {
    await page.clock.install();
    await page.goto('/checkout');
  });

  test('page loads with hydration trap warning', async ({ page }) => {
    await expect(page.locator('data-testid=checkout-page')).toBeVisible();
    await expect(page.locator('text=Hydration Timing Gap')).toBeVisible();
    await expect(page.locator('#checkout-btn[data-hydrated="false"]')).toBeVisible();
  });

  test('hydration completes after 800ms', async ({ page }) => {
    const btn = page.locator('#checkout-btn');
    await expect(btn).toHaveAttribute('data-hydrated', 'false');
    await page.clock.fastForward(801);
    await expect(btn).toHaveAttribute('data-hydrated', 'true');
  });

  test('early click before hydration increments drop count', async ({ page }) => {
    const btn = page.locator('#checkout-btn');
    await btn.click();
    await expect(page.locator('data-testid=click-dropped-warning')).toBeVisible();
    await expect(page.locator('data-testid=click-dropped-warning')).toContainText('1 click');
  });
});

test.describe('Checkout Drill — Hydration Timing Gap', () => {
  test.beforeEach(async ({ page }) => {
    await page.goto('/checkout');
  });

  test('confirm purchase bypasses hydration trap', async ({ page }) => {
    await page.click('[data-testid="confirm-purchase-btn"]');
    await expect(page.locator('data-testid=order-status')).toBeVisible();
    await expect(page.locator('data-testid=order-status')).toContainText('Order Confirmed');
  });

  test('filling form enables hydration bypass', async ({ page }) => {
    await page.fill('#address', '742 Evergreen Terrace');
    await page.click('#checkout-btn');
    await expect(page.locator('data-testid=order-status')).toBeVisible();
  });

  test('cart totals calculate correctly', async ({ page }) => {
    await page.fill('#quantity', '3');
    await expect(page.locator('#item-id')).toHaveValue('item-1');
    const subtotal = page.locator('.summary-breakdown .summary-row').first();
    await expect(subtotal).toContainText('$447.00');
  });

  test('form validation for empty address', async ({ page }) => {
    // Wait past the hydration trap -- an untouched, still-empty address field
    // never fires onChange, so it never marks the form "touched", and clicking
    // before hydration would get silently dropped rather than testing what
    // this test is actually about (there's no client-side required-field
    // validation on address).
    await expect(page.locator('#checkout-btn')).toHaveAttribute('data-hydrated', 'true', { timeout: 5000 });
    await page.click('#checkout-btn');
    await expect(page.locator('data-testid=order-status')).toBeVisible();
  });
});
