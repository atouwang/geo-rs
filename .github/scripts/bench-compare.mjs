// node .github/scripts/bench-compare.mjs baseline.json current.json
import { readFileSync } from 'node:fs'
import { compareBenchmarks, regressionThresholdPercent } from './bench-utils.mjs'

const [baselinePath, currentPath] = process.argv.slice(2)
if (!baselinePath || !currentPath) throw new Error('Usage: bench-compare.mjs baseline.json current.json')
const baseline = JSON.parse(readFileSync(baselinePath, 'utf8'))
const current = JSON.parse(readFileSync(currentPath, 'utf8'))
const { failures, results } = compareBenchmarks(baseline, current)
console.log(results.join('\n'))
if (failures > 0) {
  console.error(`::error::${failures} benchmark(s) missing, invalid, or regressed >${regressionThresholdPercent}% with separated 95% intervals`)
  process.exitCode = 1
} else {
  console.log(`All benchmarks passed the ${regressionThresholdPercent}% regression check`)
}
