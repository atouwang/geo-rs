// SharedWorker: multiple tabs share one WASM engine instance
export {}
let engine: any = null
const ports = new Set<MessagePort>()

async function init() {
  try {
    const wasm = await import('../../../../crates/geo-wasm/pkg/geo_wasm')
    await wasm.init()
    engine = new wasm.Engine()
    for (const port of ports) {
      port.postMessage({ type: 'ready' })
    }
  } catch (err) {
    for (const port of ports) {
      port.postMessage({
        type: 'error',
        message: err instanceof Error ? err.message : String(err),
      })
    }
  }
}

;(self as any).onconnect = (e: MessageEvent) => {
  const port = e.ports[0]
  ports.add(port)

  port.onmessage = (ev: MessageEvent) => {
    const { id, method, args } = ev.data
    if (!engine) {
      port.postMessage({ id, ok: false, error: 'Engine not initialized' })
      return
    }
    try {
      const result = engine[method](...args)
      const transfer: Transferable[] = []
      if (result instanceof Uint8Array) transfer.push(result.buffer)
      port.postMessage({ id, ok: true, result }, { transfer })
    } catch (err) {
      port.postMessage({
        id, ok: false,
        error: err instanceof Error ? err.message : String(err),
      })
    }
  }

  if (!engine) init()
  else port.postMessage({ type: 'ready' })
}
