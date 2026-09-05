import { test, expect } from '@playwright/test';

test.describe('Mission Control Badging System — Chaos Survivor & The Architect', () => {
  const BASE_PROGRESS = {
    total_xp: 0,
    level_name: 'SDET Learner',
    streak_days: 0,
    flakiness_100_streak: 0,
    perfect_locator_count: 0,
    achievements: [],
    completed_drills: {},
  };

  test.beforeEach(async ({ page }) => {
    // Intercept curriculum endpoint so test is hermetic and doesn't depend on backend server
    await page.route('**/api/curriculum', async (route) => {
      await route.fulfill({
        status: 200,
        contentType: 'application/json',
        body: JSON.stringify({ tracks: [] }),
      });
    });
  });

  test('both badges render in LOCKED state when no path completions exist', async ({ page }) => {
    await page.route('**/api/progress', async (route) => {
      await route.fulfill({
        status: 200,
        contentType: 'application/json',
        body: JSON.stringify(BASE_PROGRESS),
      });
    });

    await page.goto('/mission-control');
    await expect(page.locator('data-testid=badges-showcase')).toBeVisible();

    // Verify Chaos Survivor is locked
    const chaosBadge = page.locator('data-testid=badge-chaos_survivor');
    await expect(chaosBadge).toBeVisible();
    await expect(chaosBadge).toHaveAttribute('data-unlocked', 'false');
    await expect(chaosBadge.locator('data-testid=badge-status-pill')).toContainText('LOCKED');
    await expect(chaosBadge.locator('data-testid=badge-name')).toContainText('Chaos Survivor');
    await expect(chaosBadge.locator('data-testid=badge-path-desc')).toBeVisible();
    await expect(chaosBadge.locator('data-testid=badge-progress-bar')).toBeVisible();

    // Verify The Architect is locked
    const archBadge = page.locator('data-testid=badge-the_architect');
    await expect(archBadge).toBeVisible();
    await expect(archBadge).toHaveAttribute('data-unlocked', 'false');
    await expect(archBadge.locator('data-testid=badge-status-pill')).toContainText('LOCKED');
    await expect(archBadge.locator('data-testid=badge-name')).toContainText('The Architect');
    await expect(archBadge.locator('data-testid=badge-path-desc')).toBeVisible();
    await expect(archBadge.locator('data-testid=badge-progress-bar')).toBeVisible();

    // Counter shows 0 unlocked
    await expect(page.locator('data-testid=unlocked-counter')).toContainText('0 of');
  });

  test('Chaos Survivor renders UNLOCKED when chaos path completion state is passed in', async ({ page }) => {
    const chaosProgress = {
      ...BASE_PROGRESS,
      achievements: [
        {
          id: 'chaos_survivor',
          name: 'Chaos Survivor',
          description: 'Pass all 5 flakiness iterations against chaos on a k6 drill',
          unlocked_at: '2026-09-05T09:15:00Z',
        },
      ],
    };

    await page.route('**/api/progress', async (route) => {
      await route.fulfill({
        status: 200,
        contentType: 'application/json',
        body: JSON.stringify(chaosProgress),
      });
    });

    await page.goto('/mission-control');

    const chaosBadge = page.locator('data-testid=badge-chaos_survivor');
    await expect(chaosBadge).toBeVisible();
    await expect(chaosBadge).toHaveAttribute('data-unlocked', 'true');
    await expect(chaosBadge.locator('data-testid=badge-status-pill')).toContainText('UNLOCKED');
    await expect(chaosBadge.locator('data-testid=badge-unlocked-at')).toBeVisible();

    // The Architect remains locked
    const archBadge = page.locator('data-testid=badge-the_architect');
    await expect(archBadge).toBeVisible();
    await expect(archBadge).toHaveAttribute('data-unlocked', 'false');
    await expect(archBadge.locator('data-testid=badge-status-pill')).toContainText('LOCKED');

    await expect(page.locator('data-testid=unlocked-counter')).toContainText('1 of');
  });

  test('The Architect renders UNLOCKED when architecture path completion state is passed in', async ({ page }) => {
    const architectProgress = {
      ...BASE_PROGRESS,
      achievements: [
        {
          id: 'the_architect',
          name: 'The Architect',
          description: 'Complete all Tool Decisions drills',
          unlocked_at: '2026-09-05T09:45:00Z',
        },
      ],
    };

    await page.route('**/api/progress', async (route) => {
      await route.fulfill({
        status: 200,
        contentType: 'application/json',
        body: JSON.stringify(architectProgress),
      });
    });

    await page.goto('/mission-control');

    const archBadge = page.locator('data-testid=badge-the_architect');
    await expect(archBadge).toBeVisible();
    await expect(archBadge).toHaveAttribute('data-unlocked', 'true');
    await expect(archBadge.locator('data-testid=badge-status-pill')).toContainText('UNLOCKED');
    await expect(archBadge.locator('data-testid=badge-unlocked-at')).toBeVisible();

    // Chaos Survivor remains locked
    const chaosBadge = page.locator('data-testid=badge-chaos_survivor');
    await expect(chaosBadge).toBeVisible();
    await expect(chaosBadge).toHaveAttribute('data-unlocked', 'false');
    await expect(chaosBadge.locator('data-testid=badge-status-pill')).toContainText('LOCKED');

    await expect(page.locator('data-testid=unlocked-counter')).toContainText('1 of');
  });

  test('both badges render UNLOCKED simultaneously when both path completions are present', async ({ page }) => {
    const dualProgress = {
      ...BASE_PROGRESS,
      achievements: [
        {
          id: 'chaos_survivor',
          name: 'Chaos Survivor',
          description: 'Pass all 5 flakiness iterations against chaos on a k6 drill',
          unlocked_at: '2026-09-05T09:15:00Z',
        },
        {
          id: 'the_architect',
          name: 'The Architect',
          description: 'Complete all Tool Decisions drills',
          unlocked_at: '2026-09-05T09:45:00Z',
        },
      ],
    };

    await page.route('**/api/progress', async (route) => {
      await route.fulfill({
        status: 200,
        contentType: 'application/json',
        body: JSON.stringify(dualProgress),
      });
    });

    await page.goto('/mission-control');

    const chaosBadge = page.locator('data-testid=badge-chaos_survivor');
    await expect(chaosBadge).toBeVisible();
    await expect(chaosBadge).toHaveAttribute('data-unlocked', 'true');
    await expect(chaosBadge.locator('data-testid=badge-status-pill')).toContainText('UNLOCKED');
    await expect(chaosBadge.locator('data-testid=badge-unlocked-at')).toBeVisible();

    const archBadge = page.locator('data-testid=badge-the_architect');
    await expect(archBadge).toBeVisible();
    await expect(archBadge).toHaveAttribute('data-unlocked', 'true');
    await expect(archBadge.locator('data-testid=badge-status-pill')).toContainText('UNLOCKED');
    await expect(archBadge.locator('data-testid=badge-unlocked-at')).toBeVisible();

    await expect(page.locator('data-testid=unlocked-counter')).toContainText('2 of');
  });

  test('category filtering isolates Chaos and Architecture badges correctly', async ({ page }) => {
    await page.route('**/api/progress', async (route) => {
      await route.fulfill({
        status: 200,
        contentType: 'application/json',
        body: JSON.stringify(BASE_PROGRESS),
      });
    });

    await page.goto('/mission-control');

    // Click Chaos filter tab
    await page.locator('data-testid=filter-chaos').click({ force: true });
    await expect(page.locator('data-testid=badge-chaos_survivor')).toBeVisible();
    await expect(page.locator('data-testid=badge-the_architect')).toHaveCount(0);

    // Click Architecture filter tab
    await page.locator('data-testid=filter-architecture').click({ force: true });
    await expect(page.locator('data-testid=badge-chaos_survivor')).toHaveCount(0);
    await expect(page.locator('data-testid=badge-the_architect')).toBeVisible();

    // Return to All
    await page.locator('data-testid=filter-all').click({ force: true });
    await expect(page.locator('data-testid=badge-chaos_survivor')).toBeVisible();
    await expect(page.locator('data-testid=badge-the_architect')).toBeVisible();
  });

  test('completion state overrides via query parameters', async ({ page }) => {
    await page.route('**/api/progress', async (route) => {
      await route.fulfill({
        status: 200,
        contentType: 'application/json',
        body: JSON.stringify(BASE_PROGRESS),
      });
    });

    // Test chaos_survivor override
    await page.goto('/mission-control?chaos_survivor=true');
    const chaosBadge = page.locator('data-testid=badge-chaos_survivor');
    await expect(chaosBadge).toHaveAttribute('data-unlocked', 'true');
    await expect(chaosBadge.locator('data-testid=badge-status-pill')).toContainText('UNLOCKED');

    // Test the_architect override
    await page.goto('/mission-control?the_architect=true');
    const archBadge = page.locator('data-testid=badge-the_architect');
    await expect(archBadge).toHaveAttribute('data-unlocked', 'true');
    await expect(archBadge.locator('data-testid=badge-status-pill')).toContainText('UNLOCKED');
  });

  test('path completion evaluation dynamically awards badges from completed_drills', async ({ page }) => {
    const drillProgress = {
      ...BASE_PROGRESS,
      achievements: [],
      completed_drills: {
        'k6-js/04_perf_k6': {
          track_id: 'k6-js',
          drill_id: '04_perf_k6',
          best_score: 100,
          completion_count: 5,
          first_completed_at: '2026-09-05T08:00:00Z',
          last_completed_at: '2026-09-05T08:30:00Z',
        },
        'tool-decisions/drill_01': {
          track_id: 'tool-decisions',
          drill_id: 'drill_01',
          best_score: 95,
          completion_count: 1,
          first_completed_at: '2026-09-05T08:00:00Z',
          last_completed_at: '2026-09-05T08:30:00Z',
        },
        'tool-decisions/drill_02': {
          track_id: 'tool-decisions',
          drill_id: 'drill_02',
          best_score: 98,
          completion_count: 1,
          first_completed_at: '2026-09-05T08:00:00Z',
          last_completed_at: '2026-09-05T08:30:00Z',
        },
      },
    };

    await page.route('**/api/progress', async (route) => {
      await route.fulfill({
        status: 200,
        contentType: 'application/json',
        body: JSON.stringify(drillProgress),
      });
    });

    await page.goto('/mission-control');

    // Chaos Survivor dynamically awarded
    const chaosBadge = page.locator('data-testid=badge-chaos_survivor');
    await expect(chaosBadge).toHaveAttribute('data-unlocked', 'true');
    await expect(chaosBadge.locator('data-testid=badge-status-pill')).toContainText('UNLOCKED');

    // The Architect dynamically awarded
    const archBadge = page.locator('data-testid=badge-the_architect');
    await expect(archBadge).toHaveAttribute('data-unlocked', 'true');
    await expect(archBadge.locator('data-testid=badge-status-pill')).toContainText('UNLOCKED');
  });

  test('pedagogical integrity: failing drills with 0 score do not unlock Chaos Survivor or The Architect', async ({ page }) => {
    const failedDrillProgress = {
      ...BASE_PROGRESS,
      completed_drills: {
        'k6-js/04_perf_k6': {
          track_id: 'k6-js',
          drill_id: '04_perf_k6',
          best_score: 0,
          completion_count: 0,
          first_completed_at: '',
          last_completed_at: '',
        },
        'tool-decisions/drill_01': {
          track_id: 'tool-decisions',
          drill_id: 'drill_01',
          best_score: 0,
          completion_count: 0,
          first_completed_at: '',
          last_completed_at: '',
        },
        'tool-decisions/drill_02': {
          track_id: 'tool-decisions',
          drill_id: 'drill_02',
          best_score: 0,
          completion_count: 0,
          first_completed_at: '',
          last_completed_at: '',
        },
      },
    };

    await page.route('**/api/progress', async (route) => {
      await route.fulfill({
        status: 200,
        contentType: 'application/json',
        body: JSON.stringify(failedDrillProgress),
      });
    });

    await page.goto('/mission-control');

    // Both badges MUST remain LOCKED for score 0 / count 0
    const chaosBadge = page.locator('data-testid=badge-chaos_survivor');
    await expect(chaosBadge).toHaveAttribute('data-unlocked', 'false');
    await expect(chaosBadge.locator('data-testid=badge-status-pill')).toContainText('LOCKED');

    const archBadge = page.locator('data-testid=badge-the_architect');
    await expect(archBadge).toHaveAttribute('data-unlocked', 'false');
    await expect(archBadge.locator('data-testid=badge-status-pill')).toContainText('LOCKED');

    await expect(page.locator('data-testid=unlocked-counter')).toContainText('0 of');
  });

  test('defensive resilience: corrupted non-array achievements object does not crash showcase', async ({ page }) => {
    const pageErrors: string[] = [];
    page.on('pageerror', (err) => pageErrors.push(err.message));

    await page.route('**/api/progress', async (route) => {
      await route.fulfill({
        status: 200,
        contentType: 'application/json',
        body: JSON.stringify({
          total_xp: 100,
          achievements: { corrupted: true }, // Object instead of Array
          completed_drills: {},
        }),
      });
    });

    await page.goto('/mission-control');
    await expect(page.locator('data-testid=badges-showcase')).toBeVisible();
    await expect(page.locator('data-testid=badge-chaos_survivor')).toBeVisible();
    expect(pageErrors.filter((e) => e.includes('find is not a function')).length).toBe(0);
  });

  test('category filtering supports core and performance tabs', async ({ page }) => {
    await page.route('**/api/progress', async (route) => {
      await route.fulfill({
        status: 200,
        contentType: 'application/json',
        body: JSON.stringify(BASE_PROGRESS),
      });
    });

    await page.goto('/mission-control');

    // Core filter
    await page.locator('data-testid=filter-core').click({ force: true });
    await expect(page.locator('data-testid=badge-first_blood')).toBeVisible();
    await expect(page.locator('data-testid=badge-sdet_master')).toBeVisible();
    await expect(page.locator('data-testid=badge-chaos_survivor')).toHaveCount(0);

    // Performance filter
    await page.locator('data-testid=filter-performance').click({ force: true });
    await expect(page.locator('data-testid=badge-speed_demon')).toBeVisible();
    await expect(page.locator('data-testid=badge-first_blood')).toHaveCount(0);
    await expect(page.locator('data-testid=badge-chaos_survivor')).toHaveCount(0);
  });
});
