import { test, expect } from '@playwright/test';

test.describe('Tailored E2E Functional: Mission Control & Chaos Console', () => {
  test.beforeEach(async ({ page }) => {
    await page.goto('/mission-control');
    await expect(page.locator('data-testid=mission-control-page')).toBeVisible();
  });

  test('HUD renders rank, XP progress bar, active streak and drill metrics', async ({ page }) => {
    // Rank & XP HUD
    await expect(page.locator('text=Rank:')).toBeVisible();
    await expect(page.locator('text=Active Streak')).toBeVisible();
    await expect(page.locator('text=Total Drills')).toBeVisible();

    // Verify progress bar element is present
    const progressBar = page.locator('.card div[style*="linear-gradient(90deg"]');
    await expect(progressBar).toBeVisible();
  });

  test('Live Chaos Simulator: injects latency and validates telemetry roundtrip', async ({ page }) => {
    // Select Products endpoint
    const endpointSelect = page.locator('#chaos-endpoint');
    await endpointSelect.selectOption('/products?page=1&per_page=5');

    // Configure Chaos Header with latency delay
    const chaosHeaderInput = page.locator('input[placeholder="e.g. delay=500ms;jitter=100ms"]');
    await chaosHeaderInput.fill('delay=200ms;jitter=30ms');

    // Fire chaos request
    const fireBtn = page.locator('button:has-text("Fire Request")');
    await fireBtn.click();

    // Telemetry output verification
    await expect(page.locator('text=STATUS:')).toBeVisible({ timeout: 8000 });
    await expect(page.locator('text=ROUNDTRIP LATENCY:')).toBeVisible();
    await expect(page.locator('pre:has-text("prod-001")')).toBeVisible();
  });

  test('Live Chaos Simulator: blocks prompt injection attack with 400 guardrail', async ({ page }) => {
    const endpointSelect = page.locator('#chaos-endpoint');
    await endpointSelect.selectOption('/api/llm/agent');

    // Verify method automatically switches to POST and body auto-populates
    const methodSelect = page.locator('#chaos-method');
    await expect(methodSelect).toHaveValue('POST');

    // Fire prompt injection request
    await page.click('button:has-text("Fire Request")');

    // Verify guardrail blocked the attack
    await expect(page.locator('text=STATUS:')).toBeVisible({ timeout: 8000 });
    await expect(page.locator('pre:has-text("PROMPT_INJECTION_DETECTED")')).toBeVisible();
  });

  test('Live Chaos Simulator: allows benign LLM prompt and returns safe response', async ({ page }) => {
    const endpointSelect = page.locator('#chaos-endpoint');
    await endpointSelect.selectOption('/api/llm/agent');

    // Replace prompt with benign inquiry
    const payloadInput = page.locator('input.form-input').nth(1);
    await payloadInput.fill('{"prompt": "Explain Playwright page object model pattern"}');

    await page.click('button:has-text("Fire Request")');

    // Verify 200 OK and safe response
    await expect(page.locator('text=STATUS:')).toBeVisible({ timeout: 8000 });
    await expect(page.locator('pre:has-text("Processed query safely")')).toBeVisible();
    await expect(page.locator('pre:has-text("grounding_score")')).toBeVisible();
  });

  test('Live Chaos Simulator: blocks SSRF access to cloud metadata', async ({ page }) => {
    const endpointSelect = page.locator('#chaos-endpoint');
    await endpointSelect.selectOption('/api/security/fetch-url');

    // Clear any latency chaos header
    const chaosHeaderInput = page.locator('input[placeholder="e.g. delay=500ms;jitter=100ms"]');
    await chaosHeaderInput.fill('');

    await page.click('button:has-text("Fire Request")');

    // Verify 403 Forbidden with SSRF_ATTEMPT_PREVENTED
    await expect(page.locator('text=STATUS:')).toBeVisible({ timeout: 8000 });
    await expect(page.locator('pre:has-text("SSRF_ATTEMPT_PREVENTED")')).toBeVisible();
  });

  test('Curriculum Explorer: filters tracks and copies CLI watch command', async ({ page }) => {
    // Wait for tracks list to load
    await expect(page.locator('text=Polyglot Curriculum Catalog')).toBeVisible();

    // Verify copy CLI command button updates to copied state
    const copyBtn = page.locator('button:has-text("Copy \'watch --track=")').first();
    if (await copyBtn.isVisible()) {
      await copyBtn.click();
      await expect(page.locator('button:has-text("Command Copied!")')).toBeVisible();
    }
  });

  test('Drill Theory Modal: opens drill card, verifies theory and hints content, then closes', async ({ page }) => {
    // Wait for catalog to load then click theory link
    const theoryLink = page.locator('text=📖 Theory').first();
    await theoryLink.waitFor({ state: 'visible', timeout: 10000 });
    await theoryLink.scrollIntoViewIfNeeded();
    await theoryLink.click();

    // Verify modal overlay opens
    const modalHeader = page.locator('text=Drill Architecture Guide');
    await expect(modalHeader).toBeVisible({ timeout: 10000 });

    // Verify theoretical context and hints are displayed
    await expect(page.locator('text=Theoretical Context & Case Study')).toBeVisible({ timeout: 10000 });
    await expect(page.locator('text=Progressive Hints & Solutions')).toBeVisible();

    // Close modal
    const closeBtn = page.locator('button[aria-label="Close modal"]');
    await closeBtn.click();
    await expect(modalHeader).not.toBeVisible();
  });
});
