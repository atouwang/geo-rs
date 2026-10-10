import { defineConfig } from '@playwright/test'
import browserConfig from './playwright.config.mjs'

export default defineConfig({
  ...browserConfig,
  globalTimeout: 180_000,
  outputDir: 'target/package-tests/browser/results',
  reporter: [
    ['list'],
    ['html', { outputFolder: 'target/package-tests/browser/report', open: 'never' }],
    ['json', { outputFile: 'target/package-tests/browser/results.json' }],
  ],
  use: { ...browserConfig.use, baseURL: 'http://127.0.0.1:4189/package-consumer/' },
  webServer: {
    command: 'node .github/scripts/package-preview.mjs',
    url: 'http://127.0.0.1:4189/package-consumer/tests/browser-smoke.html',
    reuseExistingServer: false, timeout: 30_000,
  },
})
