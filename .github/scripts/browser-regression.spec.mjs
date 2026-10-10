import { test, expect } from '@playwright/test'

const suites = [
  { name: 'SDK dedicated/shared workers', path: 'browser-smoke', checks: 5 },
  { name: 'Vue canvas and asynchronous composables', path: 'browser-vue', checks: 6 },
]

for (const suite of suites) {
  test(suite.name, async ({ page }, testInfo) => {
    const errors = []
    const canvasReadbackNotices = []
    page.on('pageerror', error => errors.push(`pageerror: ${error.message}`))
    page.on('console', message => {
      if (['error', 'warning'].includes(message.type())) {
        // This pixel-test fixture deliberately reads the GPU canvas repeatedly.
        // Preserve Chromium's exact performance advisory without changing the
        // component's rendering backend just for test instrumentation.
        if (suite.path === 'browser-vue' && message.type() === 'warning'
          && message.text() === 'Canvas2D: Multiple readback operations using getImageData are faster with the willReadFrequently attribute set to true. See: https://html.spec.whatwg.org/multipage/canvas.html#concept-canvas-will-read-frequently') {
          canvasReadbackNotices.push(message.text())
          return
        }
        errors.push(`console ${message.type()}: ${message.text()}`)
      }
    })

    try {
      const response = await page.goto(`tests/${suite.path}.html`)
      expect(response?.status()).toBe(200)
      const run = page.locator('#run')
      const output = page.locator('#results')
      for (let iteration = 1; iteration <= 2; iteration++) {
        await test.step(`Run ${iteration}: complete all ${suite.checks} checks`, async () => {
          await run.click()
          // Each fixture disables the button synchronously, then enables it in finally.
          // Wait for completion before reading output, including on the second run.
          await expect(run).toBeEnabled()
          const result = await output.innerText()
          await testInfo.attach(`suite-run-${iteration}`, { body: result, contentType: 'text/plain' })
          const lines = result.trim().split('\n')
          expect(lines.at(-1), result).toBe('ALL PASSED')
          expect(lines.filter(line => line.startsWith('PASS ')), result).toHaveLength(suite.checks)
          expect(lines.some(line => line.startsWith('FAIL ')), result).toBe(false)
          expect(errors).toEqual([])
        })
      }
    } finally {
      await testInfo.attach('browser-errors', {
        body: JSON.stringify(errors, null, 2), contentType: 'application/json',
      })
      await testInfo.attach('canvas-readback-notices', {
        body: JSON.stringify(canvasReadbackNotices, null, 2), contentType: 'application/json',
      })
    }
  })
}
