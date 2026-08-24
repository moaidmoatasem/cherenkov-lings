import { defineConfig, devices } from '@playwright/test';

export default defineConfig({
  testDir: './exercises',
  testMatch: /.*\.ts/,
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
