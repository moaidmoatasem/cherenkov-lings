import { test, expect } from '@playwright/test';

/**
 * Sprint 5 Phase 3: the Embedded/Native view toggle on Mission Control's Code
 * Review page, and the StreamViewer it swaps in.
 */
test.describe('Code Review — embedded stream viewer', () => {
  test.beforeEach(async ({ page }) => {
    await page.goto('/code-review');
    await expect(page.getByTestId('view-mode-toggle')).toBeVisible();
  });

  test('defaults to Embedded and shows the stream viewer', async ({ page }) => {
    const toggle = page.getByTestId('view-mode-toggle');

    await expect(toggle).toContainText('View Mode: Embedded');
    await expect(toggle).toHaveAttribute('aria-pressed', 'true');
    await expect(page.getByTestId('stream-viewer')).toBeVisible();
    await expect(page.getByTestId('native-mode-message')).toHaveCount(0);
  });

  test('switching to Native replaces the stream with the external-window notice', async ({
    page,
  }) => {
    const toggle = page.getByTestId('view-mode-toggle');
    await toggle.click();

    await expect(toggle).toContainText('View Mode: Native');
    await expect(toggle).toHaveAttribute('aria-pressed', 'false');
    await expect(page.getByTestId('stream-viewer')).toHaveCount(0);
    await expect(page.getByTestId('native-mode-message')).toContainText(
      'Tests are running in external native windows.',
    );
  });

  test('the toggle round-trips back to Embedded', async ({ page }) => {
    const toggle = page.getByTestId('view-mode-toggle');

    await toggle.click();
    await expect(page.getByTestId('native-mode-message')).toBeVisible();
    await toggle.click();

    await expect(toggle).toContainText('View Mode: Embedded');
    await expect(page.getByTestId('stream-viewer')).toBeVisible();
  });

  test('stream viewer renders its header and placeholder body honestly', async ({ page }) => {
    const viewer = page.getByTestId('stream-viewer');

    await expect(viewer.locator('.stream-title')).toContainText('NoVNC/WebRTC');
    await expect(viewer.locator('.stream-content')).toContainText(
      'Mocking live device emulation stream',
    );
    // The badge used to claim "LIVE" over text admitting the stream is mocked.
    await expect(viewer.getByRole('status')).toContainText('SIMULATED');
  });

  test('the live indicator actually animates', async ({ page }) => {
    // Regression: the dot declared `animation: pulse 1.5s infinite` while no
    // @keyframes pulse existed, so the "live" indicator sat perfectly still.
    const animationName = await page
      .getByTestId('stream-viewer')
      .locator('.live-dot')
      .evaluate((el) => getComputedStyle(el).animationName);

    expect(animationName).toBe('pulse');
  });

  test('view mode does not disturb the AST review panel', async ({ page }) => {
    // The toggle swaps the left column only; the review workspace must survive.
    await expect(page.locator('.review-workspace-grid')).toBeVisible();
    await page.getByTestId('view-mode-toggle').click();
    await expect(page.locator('.review-workspace-grid')).toBeVisible();
    await expect(page.locator('.editor-card')).toBeVisible();
  });
});
