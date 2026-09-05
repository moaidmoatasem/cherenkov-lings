import { test, expect } from '@playwright/test';

test.describe('E2E User Journeys', () => {
  test('Journey 1: First-Time Learner completes checkout and code review', async ({ page }) => {
    // '/' is the Learn on-ramp (no data-testid="home-page" there); the sandbox
    // overview with that testid lives at '/sandbox'.
    await page.goto('/sandbox');
    await expect(page.locator('data-testid=home-page')).toBeVisible();

    // Navigate to checkout
    await page.click('button:has-text("Try Hydration Sandbox")');
    await expect(page).toHaveURL('/checkout');

    // Fill form and complete purchase
    await page.fill('#address', '742 Evergreen Terrace');
    await page.click('#checkout-btn');
    await expect(page.locator('data-testid=order-status')).toContainText('Order Confirmed');

    // Go to mission control
    await page.goto('/mission-control');
    await expect(page.locator('data-testid=mission-control-page')).toBeVisible();

    // Go to code review
    await page.goto('/code-review');
    await expect(page.locator('.code-review-page')).toBeVisible();

    // Select a template and run review
    await page.click('.template-pill:has-text("Hardcoded Sleep")');
    await page.click('button:has-text("Re-Run AST Review")');
    await expect(page.locator('text=Review complete')).toBeVisible();
  });

  test('Journey 2: Pipeline Builder builds and runs pipeline', async ({ page }) => {
    // No separate "Validate Workflow" step -- SDET validation shows up front,
    // and "Run Simulation" is the only action button.
    await page.goto('/pipeline-builder');
    await expect(page.locator('text=Enterprise SDET Architecture Validation')).toBeVisible();

    await page.click('button:has-text("Run Simulation")');
    await expect(page.locator('text=Parallel Matrix Simulation in Progress')).toBeVisible();
  });

  test('Journey 3: Security audit through chaos simulator', async ({ page }) => {
    await page.goto('/mission-control');

    // Mission Control has two <select>s (HTTP method, endpoint) -- target the
    // endpoint one specifically instead of the ambiguous bare 'select'.
    const endpointSelect = page.locator('select').nth(1);

    // Fire SQLi attack
    await endpointSelect.selectOption('/api/security/user-lookup?user_id=1%20OR%20SLEEP(1)');
    await page.click('button:has-text("Fire Request")');
    await expect(page.locator('text=Blind timing SQLi detected')).toBeVisible({ timeout: 10000 });

    // Fire SSRF attack
    await endpointSelect.selectOption('/api/security/fetch-url');
    await page.fill('input[placeholder="e.g. delay=500ms;jitter=100ms"]', '');
    await page.click('button:has-text("Fire Request")');
    await expect(page.locator('text=SSRF_ATTEMPT_PREVENTED')).toBeVisible();
  });

  test('Journey 4: Transfer with polling and ledger reset', async ({ page }) => {
    await page.goto('/transfer');
    await page.fill('#amount-input', '250.00');
    await page.click('#transfer-btn');
    await expect(page.locator('data-testid=transfer-status')).toHaveText('Transfer Settled', { timeout: 10000 });
    await page.click('[data-testid="reset-ledger-btn"]');
    await expect(page.locator('data-testid=account-balance')).toHaveText('1000.00');
  });
});
