import { defineConfig, mergeConfig } from 'vite'
import { resolve } from 'node:path'
import siteConfig from './vite.config'

// Build browser suites against packaged SDK/Vue release output for CI and manual checks.
export default mergeConfig(siteConfig, defineConfig({
  build: {
    outDir: 'dist/browser',
    rollupOptions: { input: {
      smoke: resolve('tests/browser-smoke.html'),
      vue: resolve('tests/browser-vue.html'),
    } },
  },
}))
