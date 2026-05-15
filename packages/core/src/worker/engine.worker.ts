export {}
let engine: any = null
let wasm: any = null

async function loadWasm() {
  if (wasm) return wasm
  wasm = await import('../../../../crates/geo-wasm/pkg/geo_wasm')
  await wasm.init()
  return wasm
}

async function createEngine(memoryLimit?: number) {
  const w = await loadWasm()
  engine = new w.Engine(memoryLimit ?? undefined)
  self.postMessage({ type: 'ready' })
}

self.onmessage = (e: MessageEvent) => {
  const { id, method, args, type, memoryLimit } = e.data

  if (type === 'init') {
    createEngine(memoryLimit).catch((err: Error) => {
      self.postMessage({ type: 'error', message: err.message })
    })
    return
  }

  if (!engine) {
    self.postMessage({ id, ok: false, error: 'Engine not initialized' })
    return
  }
  try {
    const result = engine[method](...args)
    const transfer: Transferable[] = []
    if (result instanceof Uint8Array) transfer.push(result.buffer)
    self.postMessage({ id, ok: true, result }, { transfer })
  } catch (err) {
    self.postMessage({
      id, ok: false,
      error: err instanceof Error ? err.message : String(err),
    })
  }
}
