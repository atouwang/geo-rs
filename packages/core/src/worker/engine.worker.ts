let engine: any = null

async function init() {
  try {
    const wasm = await import('../../../../crates/geo-wasm/pkg/geo_wasm')
    await wasm.init()
    engine = new wasm.Engine()
    self.postMessage({ type: 'ready' })
  } catch (err) {
    self.postMessage({
      type: 'error',
      message: err instanceof Error ? err.message : String(err),
    })
  }
}

self.onmessage = (e: MessageEvent) => {
  const { id, method, args } = e.data
  if (!engine) {
    self.postMessage({ id, ok: false, error: 'Engine not initialized' })
    return
  }
  try {
    const result = engine[method](...args)
    const transfer: Transferable[] = []
    if (result instanceof Uint8Array) {
      transfer.push(result.buffer)
    }
    self.postMessage({ id, ok: true, result }, { transfer })
  } catch (err) {
    self.postMessage({
      id, ok: false,
      error: err instanceof Error ? err.message : String(err),
    })
  }
}

init()
