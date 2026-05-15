// Compare criterion bench output against stored baseline
// Run from repo root: cargo bench 2>&1 | node .github/scripts/bench-compare.mjs

import { readFileSync } from 'node:fs'
import { dirname, resolve } from 'node:path'
import { fileURLToPath } from 'node:url'

const repoRoot = resolve(dirname(fileURLToPath(import.meta.url)), '../..')
const baseline = JSON.parse(readFileSync(resolve(repoRoot, '.github/benchmarks/baseline.json'), 'utf-8'))
const chunks = []
for await (const chunk of process.stdin) chunks.push(chunk)
const input = Buffer.concat(chunks).toString('utf-8')

const current = {}
for (const line of input.split('\n')) {
  const m = line.match(/^(\S+)\s+time:\s+\[[\d.]+\s+(?:ns|µs|ms)\s+([\d.]+)\s+(?:ns|µs|ms)/)
  if (!m) continue
  const name = m[1]
  const median = parseFloat(m[2])
  if (line.includes('µs')) current[name] = { median_ns: median * 1000 }
  else if (line.includes('ms')) current[name] = { median_ns: median * 1_000_000 }
  else current[name] = { median_ns: median }
}

let regressions = 0
const results = []

for (const [name, base] of Object.entries(baseline)) {
  const curr = current[name]
  if (!curr) { results.push(`  ${name}: MISSING`); continue }
  const pct = ((curr.median_ns - base.median_ns) / base.median_ns * 100).toFixed(1)
  const icon = parseFloat(pct) > 10 ? 'FAIL' : parseFloat(pct) > 5 ? 'WARN' : 'OK'
  results.push(`  ${icon} ${name}: ${base.median_ns} -> ${curr.median_ns.toFixed(1)}ns (${pct}%)`)
  if (parseFloat(pct) > 10) regressions++
}

console.log(results.join('\n'))
console.log('')
if (regressions > 0) {
  console.log(`::warning::${regressions} benchmark(s) regressed >5%`)
  process.exit(1)
} else {
  console.log('All benchmarks within 5% of baseline')
}
