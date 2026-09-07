import path from 'path';
import { defineConfig, devices } from '@playwright/test';

export default defineConfig({
  testDir: '.',
  testMatch: ['exercises/**/*.ts', 'crucible/frontend/e2e/**/*.ts'],
  // Safely ignore .claude only when it is a subdirectory of the current checkout root (__dirname),
  // avoiding false positives if the checkout itself is hosted inside a .claude/worktrees path.
  testIgnore: [
    '**/node_modules/**',
    '**/dist/**',
    '**/.git/**',
    path.resolve(__dirname, '.claude').replace(/\\/g, '/') + '/**',
  ],
  fullyParallel: false,
  workers: 1,
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
