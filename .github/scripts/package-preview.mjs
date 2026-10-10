import { readFileSync } from 'node:fs'
import { createRequire } from 'node:module'
import { basename, dirname, join, resolve } from 'node:path'
import { fileURLToPath, pathToFileURL } from 'node:url'

const root = resolve(dirname(fileURLToPath(import.meta.url)), '../..')
const reportsRoot = join(root, 'target/package-tests')
const metadata = JSON.parse(readFileSync(join(reportsRoot, 'latest.json'), 'utf8'))
const reportDir = resolve(metadata.reportDir)
if (metadata.status !== 'success' || dirname(reportDir) !== reportsRoot || !basename(reportDir).startsWith('run-')) {
  throw new Error('No successful packed consumer build; run pnpm package:prepare first')
}
const require = createRequire(join(root, 'packages/site/package.json'))
const { preview } = await import(pathToFileURL(require.resolve('vite')).href)
const server = await preview({
  configFile: false, root: reportDir, base: '/package-consumer/',
  preview: { host: '127.0.0.1', port: 4189, strictPort: true },
})

server.printUrls()
