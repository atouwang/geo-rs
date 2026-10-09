import { test } from 'node:test'
import assert from 'node:assert/strict'
import { parseCriterionOutput, compareBenchmarks } from './bench-utils.mjs'

test('parses Criterion inline, multiline and mixed-unit intervals', () => {
  const output = 'area_simple time: [60.0 ns 66.2 ns 70.0 ns]\n' +
    'simplify_1000pts\n                        time: [0.060 ms 68.005 µs 0.080 ms]\n' +
    'union_two_squares time: [9.0 us 9.3732 us 10.0 us]\n' +
    'Benchmarking unrelated: Warming up for 3.0000 s\n'
  assert.deepEqual(parseCriterionOutput(output), {
    area_simple: { median_ns: 66.2 }, simplify_1000pts: { median_ns: 68005 },
    union_two_squares: { median_ns: 9373.2 },
  })
})

test('missing benchmark output must fail the regression check', () => {
  const result = compareBenchmarks({ area: { median_ns: 10 } }, {})
  assert.equal(result.failures, 1)
  assert.match(result.results[0], /MISSING/)
})

test('uses the reported five-percent threshold without rounding the comparison', () => {
  const baseline = { area: { median_ns: 100 } }
  assert.equal(compareBenchmarks(baseline, { area: { median_ns: 105 } }).failures, 0)
  assert.equal(compareBenchmarks(baseline, { area: { median_ns: 105.01 } }).failures, 1)
})

test('invalid results and baselines cannot pass', () => {
  assert.equal(compareBenchmarks({ area: { median_ns: 1 } }, { area: { median_ns: NaN } }).failures, 1)
  assert.throws(() => compareBenchmarks({}, {}), /empty/)
  assert.throws(() => compareBenchmarks({ area: { median_ns: 0 } }, { area: { median_ns: 1 } }), /Invalid/)
})
