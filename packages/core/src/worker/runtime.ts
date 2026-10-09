import init, { Engine } from '../../../../crates/geo-wasm/pkg/geo_wasm'

let initialization: Promise<unknown> | undefined
const methods = new Set([
  'load', 'read', 'execute_unary', 'execute_binary', 'execute_bool', 'execute_measure',
  'free', 'free_all', 'stats', 'points_within', 'transform', 'voronoi', 'isolines',
])

export interface EnginePort {
  postMessage(message: unknown, options?: StructuredSerializeOptions): void
  onmessage: ((event: MessageEvent) => void) | null
}

export function attachEngine(port: EnginePort): void {
  let engine: Engine | undefined
  let pending: Promise<void> | undefined
  let disposed = false

  const dispose = () => {
    disposed = true
    engine?.free()
    engine = undefined
    port.onmessage = null
  }

  port.onmessage = (event: MessageEvent) => {
    const { id, method, args, type, memoryLimit } = event.data ?? {}
    if (type === 'dispose') { dispose(); return }
    if (type === 'init') {
      if (disposed) return
      if (!pending) {
        pending = (async () => {
          initialization ??= init().catch((error) => { initialization = undefined; throw error })
          await initialization
          if (!disposed) engine = new Engine(memoryLimit == null ? undefined : BigInt(memoryLimit))
        })()
      }
      void pending.then(() => {
        if (!disposed) port.postMessage({ type: 'ready' })
      }, (error: unknown) => {
        if (!disposed) port.postMessage({ type: 'error', message: String(error) })
      })
      return
    }
    if (!engine) {
      port.postMessage({ id, ok: false, error: 'Engine not initialized' })
      return
    }
    try {
      if (!methods.has(method)) throw new Error('Unknown engine method: ' + method)
      const name = method === 'free' ? 'release' : method
      const fn = (engine as unknown as Record<string, (...args: unknown[]) => unknown>)[name]
      const result = fn.apply(engine, args)
      const transfer = result instanceof Uint8Array ? [result.buffer as ArrayBuffer] : []
      port.postMessage({ id, ok: true, result }, { transfer })
    } catch (error) {
      port.postMessage({ id, ok: false, error: error instanceof Error ? error.message : String(error) })
    }
  }
}
