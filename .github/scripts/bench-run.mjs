import { spawn, execFileSync } from 'node:child_process'
import { createHash } from 'node:crypto'
import { appendFileSync, copyFileSync, mkdirSync, mkdtempSync, readFileSync, unlinkSync, writeFileSync, rmdirSync } from 'node:fs'
import { cpus, tmpdir, platform, arch, release } from 'node:os'
import { dirname, join, resolve, sep } from 'node:path'
import { fileURLToPath } from 'node:url'
import { compareBenchmarks, readCriterionEstimates, regressionThresholdPercent } from './bench-utils.mjs'

const root = resolve(dirname(fileURLToPath(import.meta.url)), '../..')
const git = args => execFileSync('git', args, { cwd: root, encoding: 'utf8' }).trim()
const cliArgs = process.argv.slice(2)
if (cliArgs[0] === '--') cliArgs.shift()
const requestedBase = cliArgs[0] || process.env.BENCH_BASE_SHA || 'HEAD^'
const base = git(['rev-parse', '--verify', `${/^0+$/.test(requestedBase) ? 'HEAD^' : requestedBase}^{commit}`])
const head = git(['rev-parse', 'HEAD'])
const suite = JSON.parse(readFileSync(join(root, '.github/benchmarks/suite.json'), 'utf8'))
// Validate names before constructing paths or spawning a costly benchmark.
if (!Array.isArray(suite) || !suite.length || new Set(suite).size !== suite.length
    || suite.some(name => !/^[a-zA-Z0-9_]+$/.test(name))) throw new Error('Invalid benchmark suite')
const reportsRoot = join(root, 'target/performance')
mkdirSync(reportsRoot, { recursive: true })
const reportDir = mkdtempSync(join(reportsRoot, 'run-'))
const ownedTemp = mkdtempSync(join(tmpdir(), 'geo-rs-bench-'))
const baselineRoot = join(ownedTemp, 'baseline')
const baselineTarget = join(root, 'target/bench-baseline')
const currentTarget = join(root, 'target/bench-current')
const json = (name, data) => writeFileSync(join(reportDir, name), JSON.stringify(data, null, 2) + '\n')
const metadata = {
  baseline_commit: base, current_commit: head, requested_base: requestedBase,
  dirty_worktree: git(['status', '--porcelain']).length > 0,
  platform: platform(), arch: arch(), os: release(), cpu: cpus()[0]?.model,
  rustc: execFileSync('rustc', ['-Vv'], { encoding: 'utf8' }).trim(),
  suite, sample_size: 30, warm_up_seconds: 1, measurement_seconds: 2,
  threshold_percent: regressionThresholdPercent, confidence_level: 0.95,
  same_harness: true, started_at: new Date().toISOString(),
  harness_sha256: createHash('sha256').update(readFileSync(join(root, 'benches/rust/core_ops.rs'))).digest('hex'),
  lockfile_sha256: createHash('sha256').update(readFileSync(join(root, 'Cargo.lock'))).digest('hex'),
  pairing: 'per-case, alternating order', confirmation_order: 'reverse initial pair order',
}
json('metadata.json', metadata)
console.log(`Comparing ${base} -> ${head}; reports: ${reportDir}`)

async function runCargo(cwd, target, label, args) {
  const log = join(reportDir, `${label}.log`)
  writeFileSync(log, '')
  await new Promise((accept, reject) => {
    const child = spawn('cargo', args, { cwd, env: { ...process.env, CARGO_TARGET_DIR: target }, stdio: ['ignore', 'pipe', 'pipe'] })
    for (const stream of [child.stdout, child.stderr]) stream.on('data', chunk => {
      appendFileSync(log, chunk)
      process.stdout.write(chunk)
    })
    child.on('error', reject)
    child.on('close', code => code === 0 ? accept() : reject(new Error(`${label}: cargo bench exited ${code}`)))
  })
}

async function bench(cwd, target, label, names) {
  // Cached targets may contain old Criterion output. Remove only the exact
  // estimates about to be measured, so missing output cannot pass as fresh.
  for (const name of names) {
    try { unlinkSync(join(target, 'criterion', name, 'new', 'estimates.json')) } catch (error) {
      if (error.code !== 'ENOENT') throw error
    }
  }
  const args = ['bench', '--locked', '-p', 'geo-bench', '--bench', 'core_ops', '--',
    '--noplot', '--sample-size', '30', '--warm-up-time', '1', '--measurement-time', '2']
  args.push(`^(${names.join('|')})$`)
  await runCargo(cwd, target, label, args)
  return readCriterionEstimates(target, names)
}

let added = false
try {
  git(['worktree', 'add', '--detach', baselineRoot, base])
  added = true
  // Both implementations use today's workloads/configuration, including new
  // cases. A baseline API/build incompatibility fails explicitly.
  for (const file of ['core_ops.rs', 'Cargo.toml', 'lib.rs']) {
    copyFileSync(join(root, 'benches/rust', file), join(baselineRoot, 'benches/rust', file))
  }
  // A new harness dependency also changes geo-bench's entry in Cargo.lock.
  // Pin the same resolved dependencies for both builds; --locked still rejects
  // incompatibility with the baseline implementation's own manifests.
  copyFileSync(join(root, 'Cargo.lock'), join(baselineRoot, 'Cargo.lock'))
  // Compile both first. Interleave the two implementations per case rather
  // than separating full suites by minutes of compilation/thermal drift.
  const buildArgs = ['bench', '--locked', '-p', 'geo-bench', '--bench', 'core_ops', '--no-run']
  await runCargo(baselineRoot, baselineTarget, 'baseline-build', buildArgs)
  await runCargo(root, currentTarget, 'current-build', buildArgs)
  let baseline = {}, current = {}
  async function pair(name, confirm = false) {
    const candidateFirst = (suite.indexOf(name) % 2 === 1) !== confirm
    const jobs = [
      async () => Object.assign(baseline, await bench(baselineRoot, baselineTarget, `baseline-${name}${confirm ? '-confirm' : ''}`, [name])),
      async () => Object.assign(current, await bench(root, currentTarget, `current-${name}${confirm ? '-confirm' : ''}`, [name])),
    ]
    if (candidateFirst) jobs.reverse()
    for (const run of jobs) await run()
    json('baseline.json', baseline)
    json('current.json', current)
  }
  for (const name of suite) await pair(name)
  json('baseline.json', baseline)
  json('current.json', current)
  const first = compareBenchmarks(baseline, current)
  json('first-comparison.json', first)
  const suspects = first.rows.filter(row => row.status === 'FAIL').map(row => row.name)
  if (suspects.length) {
    console.log(`Confirming ${suspects.length} potential regressions with fresh measurements of both commits`)
    for (const name of suspects) await pair(name, true)
    json('baseline.json', baseline)
    json('current.json', current)
  }
  const comparison = compareBenchmarks(baseline, current)
  json('comparison.json', comparison)
  const summary = [
    '## Native benchmark regression check', '',
    `Baseline: ${base}; current: ${head}. Same runner and current harness, paired per case.`,
    `Median threshold: >${regressionThresholdPercent}%, separated 95% confidence intervals; potential failures rerun once on both commits.`, '',
    '| Benchmark | Baseline ns | Current ns | Change | Status |',
    '|---|---:|---:|---:|---|',
    ...comparison.rows.map(row => `| ${row.name} | ${row.baseline_ns?.toFixed(1) ?? '-'} | ${row.current_ns?.toFixed(1) ?? '-'} | ${row.percent?.toFixed(1) ?? '-'}% | ${row.status} |`), '',
    `Confirmation cases: ${suspects.join(', ') || 'none'}. NOISE means the intervals overlap.`,
    'Native timings exclude browser/WASM/Worker/RPC costs. See uploaded JSON, metadata and raw logs.', '',
  ].join('\n')
  writeFileSync(join(reportDir, 'summary.md'), summary)
  if (process.env.GITHUB_STEP_SUMMARY) appendFileSync(process.env.GITHUB_STEP_SUMMARY, summary)
  console.log(comparison.results.join('\n'))
  if (comparison.failures) {
    console.error(`::error::${comparison.failures} confirmed native benchmark regression(s)`)
    process.exitCode = 1
  }
  metadata.finished_at = new Date().toISOString()
  json('metadata.json', metadata)
} finally {
  // Only our own fresh temp worktree can be force-removed. Git also removes the
  // copied harness changes; an existing user checkout is never a cleanup target.
  if (resolve(baselineRoot) !== resolve(join(ownedTemp, 'baseline'))
      || !resolve(baselineRoot).startsWith(resolve(ownedTemp) + sep)) throw new Error('Unsafe worktree cleanup path')
  if (added) git(['worktree', 'remove', '--force', baselineRoot])
  rmdirSync(ownedTemp)
}
