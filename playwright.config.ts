import { defineConfig, devices } from '@playwright/test';

export default defineConfig({
  testDir: '.',
  testMatch: ['exercises/**/*.ts', 'crucible/frontend/e2e/**/*.ts'],
  testIgnore: ['**/.claude/**', '**/node_modules/**', '**/dist/**', '**/.git/**'],
  timeout: 10000,
  retries: 0,
  use: {
    baseURL: 'http://localhost:8080',
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
