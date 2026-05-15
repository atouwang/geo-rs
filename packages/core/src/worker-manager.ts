type PendingRequest = {
  resolve: (value: bigint | number | boolean | string) => void
  reject: (error: Error) => void
}

let nextId = 0

export class WorkerManager {
  private worker: Worker | SharedWorker
  private workerUrl: string | URL
  private pending = new Map<number, PendingRequest>()
  private ready = false
  private readyPromise: Promise<void>
  private sharedMode: boolean
  private memoryLimit?: number

  constructor(options?: { workerUrl?: string | URL; canvas?: HTMLCanvasElement; shared?: boolean; memoryLimit?: number }) {
    this.sharedMode = options?.shared ?? false
    this.memoryLimit = options?.memoryLimit
    const url = options?.workerUrl ?? new URL('./worker/engine.worker.ts', import.meta.url)
    this.workerUrl = url

    if (options?.canvas) {
      const offscreen = options.canvas.transferControlToOffscreen()
      this.worker = new Worker(url, { type: 'module' })
      ;(this.worker as Worker).postMessage({ type: 'init_canvas', canvas: offscreen }, [offscreen])
    } else if (this.sharedMode) {
      this.worker = new SharedWorker(url, { type: 'module' })
      ;(this.worker as SharedWorker).port.onmessage = this.onMessage.bind(this)
      ;(this.worker as SharedWorker).port.start()
    } else {
      this.worker = new Worker(url, { type: 'module' })
      ;(this.worker as Worker).onmessage = this.onMessage.bind(this)
      ;(this.worker as Worker).onerror = this.onError.bind(this)
    }
    this.readyPromise = this.waitForReady()
  }

  private sendInit(): void {
    const initMsg: Record<string, unknown> = { type: 'init' }
    if (this.memoryLimit) initMsg.memoryLimit = this.memoryLimit
    this.postMessage(initMsg)
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
      ;(this.worker as any).addEventListener('message', handler)
      setTimeout(() => {
        if (!this.ready) reject(new Error('WASM engine initialization timed out'))
      }, 10000)
      this.sendInit()
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

  private postMessage(msg: unknown, transfer?: Transferable[]): void {
    if (this.sharedMode) {
      (this.worker as SharedWorker).port.postMessage(msg, transfer ? { transfer } : undefined)
    } else {
      (this.worker as Worker).postMessage(msg, transfer ? { transfer } : undefined)
    }
  }

  private async reconnect(): Promise<void> {
    const maxDelay = 30000
    for (let delay = 1000; delay <= maxDelay; delay *= 2) {
      try {
        if (this.sharedMode) {
          (this.worker as SharedWorker).port.close()
          this.worker = new SharedWorker(this.workerUrl, { type: 'module' })
          ;(this.worker as SharedWorker).port.onmessage = this.onMessage.bind(this)
          ;(this.worker as SharedWorker).port.start()
        } else {
          (this.worker as Worker).terminate()
          this.worker = new Worker(this.workerUrl, { type: 'module' })
          ;(this.worker as Worker).onmessage = this.onMessage.bind(this)
          ;(this.worker as Worker).onerror = this.onError.bind(this)
        }
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
      const target = this.sharedMode
        ? (this.worker as SharedWorker).port
        : (this.worker as Worker)
      ;(target as any).addEventListener('message', handler, { once: true })
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
      this.postMessage({ id, method, args }, transfer)
    })
  }

  destroy(): void {
    if (this.sharedMode) {
      (this.worker as SharedWorker).port.close()
    } else {
      (this.worker as Worker).terminate()
    }
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
