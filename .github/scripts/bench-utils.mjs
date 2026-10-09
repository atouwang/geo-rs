const factor = { ns: 1, 'µs': 1_000, us: 1_000, ms: 1_000_000, s: 1_000_000_000 }

export function parseCriterionOutput(input) {
  const current = {}
  let name
  for (const line of input.split(/\r?\n/)) {
    // Criterion prints long benchmark names on a line before the interval.
    const interval = line.match(/^\s*(?:(\S+)\s+)?time:\s*\[[\d.eE+-]+\s+\S+\s+([\d.eE+-]+)\s+(ns|µs|us|ms|s)\s+/)
    if (interval) {
      const benchmark = interval[1] ?? name
      if (benchmark) current[benchmark] = { median_ns: Number(interval[2]) * factor[interval[3]] }
      name = undefined
    } else if (/^\S+$/.test(line.trim()) && !line.startsWith('Benchmarking')) {
      name = line.trim()
    }
  }
  return current
}

export function compareBenchmarks(baseline, current) {
  let failures = 0
  const results = []
  for (const [name, base] of Object.entries(baseline)) {
    const curr = current[name]
    if (!curr || !Number.isFinite(curr.median_ns) || curr.median_ns <= 0) {
      failures++
      results.push(`  FAIL ${name}: MISSING or invalid result`)
      continue
    }
    if (!Number.isFinite(base.median_ns) || base.median_ns <= 0) throw new Error(`Invalid baseline: ${name}`)
    const pct = (curr.median_ns - base.median_ns) / base.median_ns * 100
    if (pct > 5) failures++
    results.push(`  ${pct > 5 ? 'FAIL' : 'OK'} ${name}: ${base.median_ns} -> ${curr.median_ns.toFixed(1)}ns (${pct.toFixed(1)}%)`)
  }
  if (results.length === 0) throw new Error('Benchmark baseline is empty')
  return { failures, results }
}
