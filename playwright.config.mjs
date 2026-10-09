import { defineConfig } from '@playwright/test'

export default defineConfig({
  testDir: '.github/scripts',
  testMatch: 'browser-regression.spec.mjs',
  fullyParallel: false,
  workers: 1,
  forbidOnly: true,
  retries: 0,
  timeout: 90_000,
  expect: { timeout: 30_000 },
  outputDir: 'target/browser-tests/results',
  reporter: [
    ['list'],
    ['html', { outputFolder: 'target/browser-tests/report', open: 'never' }],
    ['json', { outputFile: 'target/browser-tests/results.json' }],
  ],
  use: {
    browserName: 'chromium',
    baseURL: 'http://127.0.0.1:4187',
    trace: 'retain-on-failure',
    screenshot: 'only-on-failure',
  },
  webServer: {
    command: 'pnpm --filter @geo-rs/site exec vite preview --outDir dist/browser --host 127.0.0.1 --port 4187 --strictPort',
    url: 'http://127.0.0.1:4187/tests/browser-smoke.html',
    reuseExistingServer: false,
    timeout: 30_000,
  },
})
