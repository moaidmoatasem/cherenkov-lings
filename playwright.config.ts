import { defineConfig, devices } from '@playwright/test';

export default defineConfig({
  testDir: '.',
  testMatch: ['exercises/**/*.ts', 'crucible/frontend/e2e/**/*.ts'],
  // No '.claude' entry here. testIgnore globs are matched against the ABSOLUTE
  // path, so '**/.claude/**' matched every file in a checkout that itself lives
  // under a .claude/ directory (any git worktree Claude Code creates) and the
  // whole suite silently discovered zero tests. testMatch above already limits
  // discovery to exercises/ and crucible/frontend/e2e/, so the entry was
  // redundant as well as harmful.
  testIgnore: ['**/node_modules/**', '**/dist/**', '**/.git/**'],
  timeout: 10000,
  retries: 0,
  use: {
    // Overridable so a run can target a preview on a free port (CI, or a box
    // where 8080 is already taken) without editing this file.
    baseURL: process.env.PLAYWRIGHT_BASE_URL ?? 'http://localhost:8080',
    trace: 'off',
    headless: true,
  },
  projects: [
    {
      name: 'chromium',
      use: { ...devices['Desktop Chrome'] },
    },
  ],
});
