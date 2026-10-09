// Run from repo root: cargo bench --bench core_ops 2>&1 | node .github/scripts/bench-compare.mjs
import { readFileSync } from 'node:fs'
import { parseCriterionOutput, compareBenchmarks } from './bench-utils.mjs'

const baseline = JSON.parse(readFileSync(new URL('../benchmarks/baseline.json', import.meta.url), 'utf8'))
const chunks = []
for await (const chunk of process.stdin) chunks.push(chunk)
const { failures, results } = compareBenchmarks(baseline, parseCriterionOutput(Buffer.concat(chunks).toString('utf8')))
console.log(results.join('\n'))
if (failures > 0) {
  console.log('::warning::' + failures + ' benchmark(s) missing, invalid, or regressed >5%')
  process.exitCode = 1
} else {
  console.log('All benchmarks within 5% of baseline')
}
