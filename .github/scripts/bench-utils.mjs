import { readFileSync } from 'node:fs'
import { join } from 'node:path'

export const regressionThresholdPercent = 15

// Criterion's console central estimate is the mean. Read the actual median
// and its bootstrap 95% confidence interval from the machine-readable output.
export function readCriterionEstimates(targetDir, names) {
  if (!Array.isArray(names) || names.length === 0 || new Set(names).size !== names.length) {
    throw new Error('Benchmark suite must be nonempty and unique')
  }
  return Object.fromEntries(names.map(name => {
    if (!/^[a-zA-Z0-9_]+$/.test(name)) throw new Error(`Invalid benchmark name: ${name}`)
    const { median } = JSON.parse(readFileSync(join(targetDir, 'criterion', name, 'new', 'estimates.json'), 'utf8'))
    if (median?.confidence_interval?.confidence_level !== 0.95) throw new Error(`Expected a 95% median confidence interval: ${name}`)
    const estimate = {
      median_ns: median.point_estimate,
      lower_ns: median.confidence_interval.lower_bound,
      upper_ns: median.confidence_interval.upper_bound,
    }
    validateEstimate(estimate, name)
    return [name, estimate]
  }))
}

function validateEstimate(value, name) {
  if (!value || ![value.median_ns, value.lower_ns, value.upper_ns].every(n => Number.isFinite(n) && n > 0)
      || value.lower_ns > value.median_ns || value.median_ns > value.upper_ns) {
    throw new Error(`Invalid benchmark estimate: ${name}`)
  }
}

export function compareBenchmarks(baseline, current, thresholdPercent = regressionThresholdPercent) {
  if (!Number.isFinite(thresholdPercent) || thresholdPercent < 0) throw new Error('Invalid regression threshold')
  const names = Object.keys(baseline)
  if (names.length === 0) throw new Error('Benchmark baseline is empty')
  const rows = names.map(name => {
    validateEstimate(baseline[name], name)
    const base = baseline[name]
    const curr = current[name]
    try { validateEstimate(curr, name) } catch {
      return { name, status: 'FAIL', reason: 'MISSING or invalid result' }
    }
    const percent = (curr.median_ns - base.median_ns) / base.median_ns * 100
    const aboveThreshold = percent > thresholdPercent
    const separated = curr.lower_ns > base.upper_ns
    return {
      name, baseline_ns: base.median_ns, current_ns: curr.median_ns, percent,
      status: aboveThreshold && separated ? 'FAIL' : aboveThreshold ? 'NOISE' : 'OK',
      reason: aboveThreshold && !separated ? '95% confidence intervals overlap' : undefined,
    }
  })
  return {
    failures: rows.filter(row => row.status === 'FAIL').length,
    rows,
    results: rows.map(row => row.percent === undefined
      ? `FAIL ${row.name}: ${row.reason}`
      : `${row.status} ${row.name}: ${row.baseline_ns.toFixed(1)} -> ${row.current_ns.toFixed(1)} ns (${row.percent.toFixed(1)}%)${row.reason ? `; ${row.reason}` : ''}`),
  }
}
