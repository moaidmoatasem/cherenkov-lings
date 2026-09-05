import { defineConfig, devices } from '@playwright/test';

export default defineConfig({
  testDir: './e2e',
  fullyParallel: false,
  forbidOnly: !!process.env.CI,
  retries: process.env.CI ? 2 : 0,
  workers: 1,
  reporter: [['list'], ['html', { outputFolder: 'playwright-report' }]],
  use: {
    baseURL: 'http://localhost:8080',
    trace: 'on-first-retry',
    screenshot: 'only-on-failure',
    video: 'retain-on-failure',
  },
  projects: [
    {
      name: 'chromium',
      use: { ...devices['Desktop Chrome'] },
    },
    {
      name: 'Mobile Chrome',
      use: { ...devices['Pixel 5'] },
    },
  ],
  webServer: {
    // `node node_modules/.bin/vite` fails here: that file is a POSIX shell
    // shim, not JS, so Node throws a syntax error trying to parse it as a
    // script. `npx` resolves and executes the binary correctly cross-platform.
    // CI (.github/workflows/ci.yml) already starts its own server on 8080
    // before running these tests, so `reuseExistingServer: true` means this
    // command only actually runs for local development.
    command: 'npx vite preview --port 8080 --host 127.0.0.1',
    url: 'http://localhost:8080',
    reuseExistingServer: true,
    timeout: 30000,
  },
});
