import { test, expect } from '@playwright/test';

/**
 * The Allure/Triage page used to score submissions with a keyword heuristic of
 * its own and keep the XP in component state, so every point a learner earned
 * vanished on reload and never reached Mission Control. These pin the wiring:
 * the dataset comes from the API, and the award round-trips to /api/progress.
 */
test.describe('Triage page is backed by the triage API', () => {
  test.beforeEach(async ({ page }) => {
    await page.goto('/allure-triage');
    // Both the dataset and the XP total arrive by fetch. Waiting for the label
    // to flip to "live cases" is the signal that the API answered — asserting
    // before that reads the seeded values and passes for the wrong reason.
    await expect(page.locator('.xp-label')).toContainText('live cases');
  });

  test('renders the backend chaos dataset, not the bundled sample', async ({ page }) => {
    // The seeded fallback ships 7 cases; the real dataset is far larger.
    await expect(page.locator('.xp-label')).toContainText('live cases');
    const rows = page.locator('.test-row');
    await expect(rows.first()).toBeVisible();
    expect(await rows.count()).toBeGreaterThan(7);
  });

  test('the XP counter shows the persisted total, not a seeded number', async ({ page }) => {
    const progress = await page.request.get('http://localhost:8081/api/progress');
    expect(progress.ok()).toBeTruthy();
    const persisted = (await progress.json()).total_xp as number;

    await expect(page.locator('.xp-val')).toHaveText(`${persisted} XP`);
  });

  test('a correct hypothesis is scored by the API and persisted', async ({ page }) => {
    const before = (await (await page.request.get('http://localhost:8081/api/progress')).json())
      .total_xp as number;

    await page.getByRole('button', { name: /Product Bug/i }).click();
    const areas = page.locator('textarea');
    await areas.nth(0).fill(
      'Missing RBAC middleware check lets a standard role elevate; the endpoint returns 200 instead of 403.'
    );
    await areas.nth(1).fill('Assert response.status_code == 403 and add the middleware role check.');
    await page.getByRole('button', { name: /Submit Hypothesis/i }).click();

    const card = page.locator('.evaluation-result-card');
    await expect(card).toBeVisible();
    await expect(card).toHaveClass(/passed/);
    // The award is XP, not a percentage — no "/100" anywhere.
    await expect(page.locator('.xp-earned-badge')).toContainText('XP Earned');

    const after = (await (await page.request.get('http://localhost:8081/api/progress')).json())
      .total_xp as number;
    expect(after).toBeGreaterThan(before);
    await expect(page.locator('.xp-val')).toContainText(String(after));
  });

  test('a wrong category earns nothing and says why', async ({ page }) => {
    const before = (await (await page.request.get('http://localhost:8081/api/progress')).json())
      .total_xp as number;

    await page.getByRole('button', { name: /Flaky/i }).first().click();
    const areas = page.locator('textarea');
    await areas.nth(0).fill('Suspected proxy flakiness.');
    await areas.nth(1).fill('Retry with backoff.');
    await page.getByRole('button', { name: /Submit Hypothesis/i }).click();

    const card = page.locator('.evaluation-result-card');
    await expect(card).toBeVisible();
    await expect(card).toHaveClass(/needs-improvement/);
    await expect(page.locator('.eval-title-wrap h4')).toContainText('Wrong Category');
    // The evaluator's contrastive reasoning, not a local string.
    await expect(page.locator('.contrast-text li').first()).toBeVisible();

    const after = (await (await page.request.get('http://localhost:8081/api/progress')).json())
      .total_xp as number;
    expect(after).toBe(before);
  });
});
