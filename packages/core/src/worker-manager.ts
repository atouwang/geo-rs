type PendingRequest = {
  resolve: (value: bigint | number | boolean | string) => void
  reject: (error: Error) => void
}

let nextId = 0

export class WorkerManager {
  private worker: Worker
  private workerUrl: string | URL
  private pending = new Map<number, PendingRequest>()
  private ready = false
  private readyPromise: Promise<void>

  constructor(workerUrl?: string | URL) {
    this.workerUrl = workerUrl ?? new URL('./worker/engine.worker.ts', import.meta.url)
    this.worker = new Worker(this.workerUrl, { type: 'module' })
    this.worker.onmessage = this.onMessage.bind(this)
    this.worker.onerror = this.onError.bind(this)
    this.readyPromise = this.waitForReady()
  }

  private waitForReady(): Promise<void> {
    return new Promise((resolve, reject) => {
      const handler = (e: MessageEvent) => {
        if (e.data?.type === 'ready') {
          this.ready = true
          resolve()
        } else if (e.data?.type === 'error') {
          reject(new Error(e.data.message))
        }
      }
      this.worker.addEventListener('message', handler)
      // Timeout after 10s
      setTimeout(() => {
        if (!this.ready) reject(new Error('WASM engine initialization timed out'))
      }, 10000)
    })
  }

  async ensureReady(): Promise<void> {
    if (this.ready) return
    await this.readyPromise
  }

  private onMessage(e: MessageEvent) {
    const { id, ok, result, error } = e.data
    const pending = this.pending.get(id)
    if (!pending) return
    this.pending.delete(id)
    if (ok) {
      pending.resolve(result)
    } else {
      pending.reject(new Error(error))
    }
  }

  private onError(e: ErrorEvent) {
    const msg = e.message || 'Worker error'
    for (const [, p] of this.pending) {
      p.reject(new Error(`${msg}. WASM engine state lost — reinitialize required.`))
    }
    this.pending.clear()
    this.ready = false
    this.readyPromise = this.reconnect()
  }

  private async reconnect(): Promise<void> {
    const maxDelay = 30000
    for (let delay = 1000; delay <= maxDelay; delay *= 2) {
      try {
        this.worker.terminate()
        this.worker = new Worker(this.workerUrl, { type: 'module' })
        this.worker.onmessage = this.onMessage.bind(this)
        this.worker.onerror = this.onError.bind(this)
        await this.waitForReadyInternal()
        this.ready = true
        return
      } catch {
        await new Promise(r => setTimeout(r, delay))
      }
    }
    throw new Error('Worker reconnect failed after max retries')
  }

  private waitForReadyInternal(): Promise<void> {
    return new Promise((resolve, reject) => {
      const handler = (e: MessageEvent) => {
        if (e.data?.type === 'ready') resolve()
        else if (e.data?.type === 'error') reject(new Error(e.data.message))
      }
      this.worker.addEventListener('message', handler, { once: true })
      setTimeout(() => reject(new Error('Worker init timeout')), 10000)
    })
  }

  async call(method: string, args: unknown[]): Promise<unknown> {
    await this.ensureReady()
    const id = ++nextId
    return new Promise((resolve, reject) => {
      this.pending.set(id, { resolve, reject })
      const transfer: Transferable[] = []
      for (const arg of args) {
        if (arg instanceof Uint8Array) transfer.push(arg.buffer)
      }
      this.worker.postMessage({ id, method, args }, { transfer })
    })
  }

  destroy(): void {
    this.worker.terminate()
    this.pending.clear()
    this.ready = false
  }
}

export async function checkWasmSupport(): Promise<boolean> {
  if (typeof WebAssembly === 'undefined') return false
  try {
    const mod = await WebAssembly.compile(
      new Uint8Array([0, 97, 115, 109, 1, 0, 0, 0])
    )
    return mod instanceof WebAssembly.Module
  } catch {
    return false
  }
}

export class WasmNotSupportedError extends Error {
  constructor() {
    super(
      'WebAssembly is not supported in this environment. ' +
      'geo-rs requires WASM support (Chrome 57+, Firefox 52+, Safari 11+, Edge 16+).'
    )
    this.name = 'WasmNotSupportedError'
  }
}
