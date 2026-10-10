import { execFileSync } from 'node:child_process'
import { createHash } from 'node:crypto'
import { copyFileSync, cpSync, existsSync, mkdirSync, mkdtempSync, readFileSync, realpathSync, rmSync, unlinkSync, writeFileSync } from 'node:fs'
import { createRequire } from 'node:module'
import { tmpdir } from 'node:os'
import { basename, dirname, join, resolve, sep } from 'node:path'
import { fileURLToPath } from 'node:url'

const root = resolve(dirname(fileURLToPath(import.meta.url)), '../..')
const reportsRoot = join(root, 'target/package-tests')
mkdirSync(reportsRoot, { recursive: true })
const latest = join(reportsRoot, 'latest.json')
if (existsSync(latest)) unlinkSync(latest)
const reportDir = mkdtempSync(join(reportsRoot, 'run-'))
const tempRoot = realpathSync(tmpdir())
const consumer = mkdtempSync(join(tempRoot, 'geo-rs-package-consumer-'))
const pnpmEntry = process.env.npm_execpath
const json = (path, data) => writeFileSync(path, JSON.stringify(data, null, 2) + '\n')
const metadata = { reportDir, consumer, started_at: new Date().toISOString(), packages: [] }

function pnpm(cwd, label, args) {
  if (!pnpmEntry || !existsSync(pnpmEntry)) throw new Error('Run through pnpm package:prepare')
  try {
    const output = execFileSync(process.execPath, [pnpmEntry, ...args], {
      cwd, encoding: 'utf8', stdio: ['ignore', 'pipe', 'pipe'], timeout: 180_000,
    })
    writeFileSync(join(reportDir, `${label}.log`), output)
    return output
  } catch (error) {
    writeFileSync(join(reportDir, `${label}.log`), `${error.stdout ?? ''}\n${error.stderr ?? ''}`)
    throw new Error(`${label} failed; see ${join(reportDir, `${label}.log`)}`, { cause: error })
  }
}

function version(packageDir, name) {
  const require = createRequire(join(root, packageDir, 'package.json'))
  return JSON.parse(readFileSync(require.resolve(`${name}/package.json`), 'utf8')).version
}

// Restricted local environments may not permit pnpm's global store writes.
// Copy already installed dependencies without symlinks; the geo-rs packages
// still come exclusively from their tarballs. CI uses the real installer.
function copyExistingDependencies(tarballs) {
  const installed = new Map()
  const queue = []
  const destination = name => join(consumer, 'node_modules', ...name.split('/'))
  const enqueue = (name, source, packed = false) => {
    const manifest = JSON.parse(readFileSync(join(source, 'package.json'), 'utf8'))
    if (installed.has(name)) {
      if (installed.get(name) !== manifest.version) throw new Error(`Conflicting cached dependency: ${name}`)
      return
    }
    installed.set(name, manifest.version)
    if (!packed) cpSync(source, destination(name), {
      recursive: true, dereference: true,
      filter: path => path === source || basename(path) !== 'node_modules',
    })
    queue.push({ manifest, source })
  }
  for (const name of ['core', 'vue']) {
    const dir = destination(`@geo-rs/${name}`)
    mkdirSync(dir, { recursive: true })
    execFileSync('tar', ['-xzf', tarballs[name].slice(5), '--strip-components=1', '-C', dir])
    enqueue(`@geo-rs/${name}`, dir, true)
    // Resolve dependencies from the existing project installation, but use
    // the packed manifest to decide which dependencies may be supplied.
    queue.at(-1).source = join(root, 'packages', name)
  }
  const find = (from, name) => {
    const require = createRequire(join(from, 'package.json'))
    let path
    try { path = require.resolve(`${name}/package.json`) } catch { path = require.resolve(name) }
    let dir = dirname(realpathSync(path))
    while (dirname(dir) !== dir) {
      const manifest = join(dir, 'package.json')
      if (existsSync(manifest) && JSON.parse(readFileSync(manifest, 'utf8')).name === name) return dir
      dir = dirname(dir)
    }
    throw new Error(`Cannot locate cached dependency: ${name}`)
  }
  for (const [from, name] of [['packages/vue', 'vue'], ['packages/core', 'typescript'], ['packages/site', 'vite']]) {
    enqueue(name, find(join(root, from), name))
  }
  for (let i = 0; i < queue.length; i++) {
    const { manifest, source } = queue[i]
    for (const name of Object.keys(manifest.dependencies ?? {})) {
      if (!installed.has(name)) enqueue(name, find(source, name))
    }
    for (const name of Object.keys(manifest.optionalDependencies ?? {})) {
      try { enqueue(name, find(source, name)) } catch (error) {
        if (error.code !== 'MODULE_NOT_FOUND' && error.code !== 'ERR_PACKAGE_PATH_NOT_EXPORTED') throw error
      }
    }
  }
  metadata.dependency_mode = 'copied installed dependencies; geo-rs packages extracted from tarballs'
  metadata.copied_dependencies = Object.fromEntries(installed)
}

try {
  if (consumer.startsWith(root + sep)) throw new Error('Consumer must be outside the workspace')
  const tarballs = {}
  for (const name of ['core', 'vue']) {
    const packed = JSON.parse(pnpm(join(root, 'packages', name), `pack-${name}`, ['pack', '--json', '--pack-destination', reportDir]))
    const tarball = resolve(reportDir, basename(packed.filename))
    if (!existsSync(tarball)) throw new Error(`Missing ${name} tarball`)
    tarballs[name] = `file:${tarball.replaceAll('\\', '/')}`
    metadata.packages.push({
      name: `@geo-rs/${name}`, tarball,
      sha256: createHash('sha256').update(readFileSync(tarball)).digest('hex'),
      files: packed.files.map(file => file.path),
    })
  }
  json(join(consumer, 'package.json'), {
    name: 'geo-rs-isolated-consumer', private: true, type: 'module',
    dependencies: { '@geo-rs/core': tarballs.core, '@geo-rs/vue': tarballs.vue, vue: version('packages/vue', 'vue') },
    devDependencies: { typescript: version('packages/core', 'typescript'), vite: version('packages/site', 'vite') },
    // Resolve the packed Vue dependency to the same local core tarball; neither
    // geo-rs package is installed from registry or linked to workspace source.
    pnpm: { overrides: { '@geo-rs/core': tarballs.core } },
  })
  mkdirSync(join(consumer, 'tests'))
  for (const name of ['browser-smoke', 'browser-vue']) {
    for (const extension of ['html', 'ts']) {
      copyFileSync(join(root, 'packages/site/tests', `${name}.${extension}`), join(consumer, 'tests', `${name}.${extension}`))
    }
  }
  json(join(consumer, 'tsconfig.json'), {
    compilerOptions: {
      target: 'ES2022', module: 'ESNext', moduleResolution: 'Bundler',
      strict: true, skipLibCheck: false, noEmit: true, types: [], lib: ['ES2022', 'DOM'],
    },
    include: ['tests/*.ts'],
  })
  writeFileSync(join(consumer, 'vite.config.mjs'), `import { defineConfig } from 'vite'
import { resolve } from 'node:path'
export default defineConfig({
  base: '/package-consumer/',
  build: { target: 'esnext', rollupOptions: { input: {
    smoke: resolve('tests/browser-smoke.html'), vue: resolve('tests/browser-vue.html'),
  } } },
})
`)
  const installArgs = ['install', '--ignore-scripts', '--strict-peer-dependencies']
  if (process.env.GEO_RS_PACKAGE_OFFLINE === '1') installArgs.push('--offline')
  if (process.env.GEO_RS_PACKAGE_COPY_DEPS === '1') {
    copyExistingDependencies(tarballs)
  } else {
    pnpm(consumer, 'install', installArgs)
    metadata.dependency_mode = 'pnpm install with local tarballs'
    copyFileSync(join(consumer, 'pnpm-lock.yaml'), join(reportDir, 'consumer-lock.yaml'))
  }
  const require = createRequire(join(consumer, 'package.json'))
  metadata.resolved = {}
  for (const name of ['@geo-rs/core', '@geo-rs/vue']) {
    const packageDir = join(consumer, 'node_modules', ...name.split('/'))
    const manifest = JSON.parse(readFileSync(join(packageDir, 'package.json'), 'utf8'))
    const entry = realpathSync(resolve(packageDir, manifest.exports['.'].import))
    if (!entry.startsWith(consumer + sep)) throw new Error(`Package escaped isolated consumer: ${name}`)
    metadata.resolved[name] = entry
  }
  const tsc = require.resolve('typescript/lib/tsc.js')
  try {
    execFileSync(process.execPath, [tsc, '--noEmit'], { cwd: consumer, encoding: 'utf8', stdio: ['ignore', 'pipe', 'pipe'] })
  } catch (error) {
    writeFileSync(join(reportDir, 'typecheck.log'), `${error.stdout ?? ''}\n${error.stderr ?? ''}`)
    throw new Error(`Consumer typecheck failed; see ${join(reportDir, 'typecheck.log')}`, { cause: error })
  }
  writeFileSync(join(reportDir, 'typecheck.log'), 'Consumer typecheck passed with skipLibCheck=false\n')
  // Only distribution artifacts belong in the published packages. Keep this
  // after typechecking so missing public declaration dependencies fail first.
  for (const packed of metadata.packages) {
    if (packed.files.some(path => path !== 'package.json' && !path.startsWith('dist/'))) {
      throw new Error(`${packed.name} contains files outside dist/ and package.json`)
    }
  }
  const vite = join(dirname(require.resolve('vite/package.json')), 'bin/vite.js')
  try {
    const output = execFileSync(process.execPath, [vite, 'build'], { cwd: consumer, encoding: 'utf8', stdio: ['ignore', 'pipe', 'pipe'] })
    writeFileSync(join(reportDir, 'build.log'), output)
  } catch (error) {
    writeFileSync(join(reportDir, 'build.log'), `${error.stdout ?? ''}\n${error.stderr ?? ''}`)
    throw new Error(`Consumer build failed; see ${join(reportDir, 'build.log')}`, { cause: error })
  }
  cpSync(join(consumer, 'dist'), join(reportDir, 'dist'), { recursive: true })
  metadata.status = 'success'
  json(latest, metadata)
  console.log(`Packed consumer typecheck/build passed; reports: ${reportDir}`)
} catch (error) {
  metadata.status = 'failure'
  metadata.error = error.message
  throw error
} finally {
  metadata.finished_at = new Date().toISOString()
  json(join(reportDir, 'metadata.json'), metadata)
  // This is our fresh owned directory, never a user checkout or node_modules.
  if (dirname(consumer) !== tempRoot || !basename(consumer).startsWith('geo-rs-package-consumer-')
    || realpathSync(consumer) !== consumer) throw new Error('Unsafe consumer cleanup path')
  rmSync(consumer, { recursive: true, force: true })
}
