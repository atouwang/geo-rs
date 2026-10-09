type PendingRequest = {
  resolve: (value: unknown) => void
  reject: (error: Error) => void
}

export class WorkerManager {
  private worker: Worker | SharedWorker
  private target: Worker | MessagePort
  private pending = new Map<number, PendingRequest>()
  private ready = false
  private readyPromise: Promise<void>
  private failure: Error | null = null
  private rejectInit?: (error: Error) => void
  private nextId = 0
  private sharedMode: boolean

  constructor(options?: { workerUrl?: string | URL; canvas?: HTMLCanvasElement; shared?: boolean; memoryLimit?: number }) {
    if (options?.canvas) throw new Error('OffscreenCanvas rendering is not implemented')
    if (options?.memoryLimit !== undefined && (!Number.isSafeInteger(options.memoryLimit) || options.memoryLimit < 0)) {
      throw new Error('memoryLimit must be a non-negative safe integer')
    }
    this.sharedMode = options?.shared ?? false
    if (this.sharedMode) {
      this.worker = options?.workerUrl
        ? new SharedWorker(options.workerUrl, { type: 'module' })
        : new SharedWorker(new URL('./worker/engine.shared.worker.ts', import.meta.url), { type: 'module' })
      this.target = this.worker.port
    } else {
      this.worker = options?.workerUrl
        ? new Worker(options.workerUrl, { type: 'module' })
        : new Worker(new URL('./worker/engine.worker.ts', import.meta.url), { type: 'module' })
      this.target = this.worker
    }
    this.target.onmessage = this.onMessage
    this.target.onmessageerror = () => this.fail(new Error('Worker message could not be decoded'))
    this.worker.onerror = (event: ErrorEvent) => this.fail(new Error(
      (event.message || 'Worker error') + '. WASM engine state lost — reinitialize required.',
    ))
    this.readyPromise = new Promise<void>((resolve, reject) => {
      const timeout = setTimeout(() => this.fail(new Error('WASM engine initialization timed out')), 10000)
      const cleanup = () => {
        clearTimeout(timeout)
        this.target.removeEventListener('message', handler)
        this.rejectInit = undefined
      }
      const handler: EventListener = (event) => {
        const data = (event as MessageEvent).data
        if (data?.type === 'ready') {
          cleanup()
          this.ready = true
          resolve()
        } else if (data?.type === 'error') {
          this.fail(new Error(data.message))
        }
      }
      this.rejectInit = (error) => { cleanup(); reject(error) }
      this.target.addEventListener('message', handler)
      if (this.sharedMode) (this.target as MessagePort).start()
      try {
        this.postMessage({ type: 'init', memoryLimit: options?.memoryLimit })
      } catch (error) {
        this.fail(error instanceof Error ? error : new Error(String(error)))
      }
    })
    void this.readyPromise.catch(() => {})
  }

  async ensureReady(): Promise<void> {
    if (this.failure) throw this.failure
    if (!this.ready) await this.readyPromise
    if (this.failure) throw this.failure
  }

  private onMessage = (event: MessageEvent): void => {
    const { id, ok, result, error } = event.data ?? {}
    const pending = this.pending.get(id)
    if (!pending) return
    this.pending.delete(id)
    if (ok) pending.resolve(result)
    else pending.reject(new Error(error))
  }

  private fail(error: Error): void {
    if (this.failure) return
    this.failure = error
    this.ready = false
    this.rejectInit?.(error)
    for (const request of this.pending.values()) request.reject(error)
    this.pending.clear()
    if (this.sharedMode) {
      try { this.target.postMessage({ type: 'dispose' }) } catch { /* Port may already be closed. */ }
      ;(this.target as MessagePort).close()
    } else {
      ;(this.worker as Worker).terminate()
    }
  }

  private postMessage(message: unknown, transfer: Transferable[] = []): void {
    this.target.postMessage(message, { transfer })
  }

  async call(method: string, args: unknown[]): Promise<unknown> {
    await this.ensureReady()
    const id = ++this.nextId
    return new Promise((resolve, reject) => {
      this.pending.set(id, { resolve, reject })
      const buffers = new Set<ArrayBuffer>()
      for (const arg of args) {
        if (arg instanceof Uint8Array && arg.buffer instanceof ArrayBuffer) buffers.add(arg.buffer)
      }
      try {
        this.postMessage({ id, method, args }, [...buffers])
      } catch (error) {
        this.pending.delete(id)
        reject(error)
      }
    })
  }

  destroy(): void {
    this.fail(new Error('WASM engine has been destroyed'))
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
