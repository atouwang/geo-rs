import { test } from 'node:test'
import assert from 'node:assert/strict'
import { spawnSync } from 'node:child_process'
import { mkdirSync, mkdtempSync, writeFileSync, unlinkSync, rmdirSync } from 'node:fs'
import { tmpdir } from 'node:os'
import { join } from 'node:path'
import { fileURLToPath } from 'node:url'
import { readCriterionEstimates, compareBenchmarks } from './bench-utils.mjs'

const estimate = (median, lower = median, upper = median) => ({ median_ns: median, lower_ns: lower, upper_ns: upper })

function fixture(t) {
  const root = mkdtempSync(join(tmpdir(), 'geo-rs-bench-test-'))
  const dirs = [], files = []
  t.after(() => {
    for (const file of files) unlinkSync(file)
    for (const dir of dirs.reverse()) rmdirSync(dir)
    rmdirSync(root)
  })
  return {
    root,
    write(name, data) {
      const path = join(root, name)
      writeFileSync(path, JSON.stringify(data))
      files.push(path)
      return path
    },
    criterion(name, data) {
      let path = root
      for (const part of ['criterion', name, 'new']) {
        path = join(path, part)
        mkdirSync(path)
        dirs.push(path)
      }
      const file = join(path, 'estimates.json')
      writeFileSync(file, JSON.stringify(data))
      files.push(file)
    },
  }
}

test('reads actual median and confidence interval, not the console mean', t => {
  const f = fixture(t)
  f.criterion('area', {
    mean: { point_estimate: 900 },
    median: { point_estimate: 100, confidence_interval: { confidence_level: 0.95, lower_bound: 95, upper_bound: 105 } },
  })
  assert.deepEqual(readCriterionEstimates(f.root, ['area']), { area: estimate(100, 95, 105) })
})

test('missing Criterion output fails instead of reusing incomplete coverage', t => {
  const f = fixture(t)
  assert.throws(() => readCriterionEstimates(f.root, ['missing']), /ENOENT/)
})

test('rejects unsafe, empty and duplicate suite names', t => {
  const f = fixture(t)
  for (const names of [[], ['area', 'area'], ['../escape']]) {
    assert.throws(() => readCriterionEstimates(f.root, names), /suite|name/)
  }
})

test('malformed and wrong-confidence Criterion output cannot pass', t => {
  const f = fixture(t)
  f.criterion('area', { median: { point_estimate: 100, confidence_interval: { confidence_level: 0.90, lower_bound: 95, upper_bound: 105 } } })
  assert.throws(() => readCriterionEstimates(f.root, ['area']), /95%/)
})

test('missing benchmark results fail the regression check', () => {
  const result = compareBenchmarks({ area: estimate(100) }, {})
  assert.equal(result.failures, 1)
  assert.match(result.results[0], /MISSING/)
})

test('enforces the fifteen-percent threshold without rounding', () => {
  const base = { area: estimate(100) }
  assert.equal(compareBenchmarks(base, { area: estimate(115) }).failures, 0)
  assert.equal(compareBenchmarks(base, { area: estimate(115.01) }).failures, 1)
})

test('overlapping confidence intervals are reported as noise', () => {
  const result = compareBenchmarks({ area: estimate(100, 80, 120) }, { area: estimate(125, 110, 140) })
  assert.equal(result.failures, 0)
  assert.equal(result.rows[0].status, 'NOISE')
  assert.match(result.results[0], /overlap/)
})

test('a significant slowdown fails and an improvement passes', () => {
  const base = { area: estimate(100, 95, 105) }
  assert.equal(compareBenchmarks(base, { area: estimate(130, 125, 135) }).failures, 1)
  assert.equal(compareBenchmarks(base, { area: estimate(80, 75, 85) }).failures, 0)
})

test('invalid intervals, numbers, baselines and thresholds cannot pass', () => {
  for (const value of [estimate(NaN), estimate(0), estimate(100, 110, 120), estimate(100, 90, 95)]) {
    assert.equal(compareBenchmarks({ area: estimate(100) }, { area: value }).failures, 1)
    assert.throws(() => compareBenchmarks({ area: value }, {}), /Invalid/)
  }
  assert.throws(() => compareBenchmarks({}, {}), /empty/)
  assert.throws(() => compareBenchmarks({ area: estimate(100) }, {}, NaN), /threshold/)
})

test('CLI exits nonzero on injected slowdown or missing results and zero on success', t => {
  const f = fixture(t)
  const base = f.write('baseline.json', { area: estimate(100, 95, 105) })
  for (const [name, current, expected] of [
    ['slow', { area: estimate(130, 125, 135) }, 1],
    ['missing', {}, 1],
    ['ok', { area: estimate(101, 96, 106) }, 0],
  ]) {
    const path = f.write(name + '.json', current)
    const result = spawnSync(process.execPath, [fileURLToPath(new URL('./bench-compare.mjs', import.meta.url)), base, path], { encoding: 'utf8' })
    assert.equal(result.status, expected, result.stderr)
    assert.match(result.stdout + result.stderr, expected ? /FAIL|::error::/ : /passed/)
  }
})
