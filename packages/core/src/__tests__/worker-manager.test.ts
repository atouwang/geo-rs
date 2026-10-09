import { afterEach, beforeEach, describe, expect, it, vi } from 'vitest'
import { WorkerManager } from '../worker-manager'

class FakePort extends EventTarget {
  onmessage: ((event: MessageEvent) => void) | null = null
  onmessageerror: (() => void) | null = null
  postMessage = vi.fn()
  start = vi.fn()
  close = vi.fn()
  emit(data: unknown) {
    const event = new MessageEvent('message', { data })
    this.onmessage?.(event)
    this.dispatchEvent(event)
  }
}
class FakeWorker extends FakePort {
  static instances: FakeWorker[] = []
  onerror: ((event: ErrorEvent) => void) | null = null
  terminate = vi.fn()
  port = new FakePort()
  constructor(public url: URL | string) {
    super()
    FakeWorker.instances.push(this)
  }
}

describe('WorkerManager', () => {
  beforeEach(() => {
    vi.useFakeTimers()
    FakeWorker.instances = []
    vi.stubGlobal('Worker', FakeWorker)
    vi.stubGlobal('SharedWorker', FakeWorker)
  })
  afterEach(() => { vi.useRealTimers(); vi.unstubAllGlobals() })

  it.each([false, true])('initializes dedicated/shared=%s on the correct message target', async (shared) => {
    const manager = new WorkerManager({ shared, memoryLimit: 0 })
    const worker = FakeWorker.instances[0]
    const target = shared ? worker.port : worker
    expect(String(worker.url)).toContain(shared ? 'engine.shared.worker.ts' : 'engine.worker.ts')
    expect(target.postMessage).toHaveBeenCalledWith({ type: 'init', memoryLimit: 0 }, { transfer: [] })
    target.emit({ type: 'awaiting_init' })
    target.emit({ type: 'ready' })
    await manager.ensureReady()
    expect(vi.getTimerCount()).toBe(0)
    const result = manager.call('stats', [])
    await Promise.resolve()
    const request = target.postMessage.mock.calls.at(-1)![0] as { id: number }
    target.emit({ id: request.id, ok: true, result: 'stats' })
    await expect(result).resolves.toBe('stats')
    manager.destroy()
    if (shared) expect(target.close).toHaveBeenCalledOnce()
    else expect(worker.terminate).toHaveBeenCalledOnce()
  })

  it('rejects initialization promptly on a worker error and clears its timer', async () => {
    const manager = new WorkerManager()
    const pending = manager.ensureReady()
    FakeWorker.instances[0].onerror?.({ message: 'startup failed' } as ErrorEvent)
    await expect(pending).rejects.toThrow('startup failed')
    expect(vi.getTimerCount()).toBe(0)
  })

  it('rejects and terminates initialization on timeout', async () => {
    const manager = new WorkerManager()
    const pending = expect(manager.ensureReady()).rejects.toThrow('timed out')
    await vi.advanceTimersByTimeAsync(10000)
    await pending
    expect(FakeWorker.instances[0].terminate).toHaveBeenCalledOnce()
  })

  it('rejects in-flight and subsequent calls after destroy', async () => {
    const manager = new WorkerManager()
    FakeWorker.instances[0].emit({ type: 'ready' })
    const pending = manager.call('load', [])
    await Promise.resolve()
    manager.destroy()
    await expect(pending).rejects.toThrow('destroyed')
    await expect(manager.call('stats', [])).rejects.toThrow('destroyed')
    manager.destroy()
    expect(FakeWorker.instances[0].terminate).toHaveBeenCalledOnce()
  })

  it('rejects calls on a crash without silently reusing lost handles in a new arena', async () => {
    const manager = new WorkerManager()
    const worker = FakeWorker.instances[0]
    worker.emit({ type: 'ready' })
    const pending = manager.call('read', [1n])
    await Promise.resolve()
    worker.onerror?.({ message: 'crash' } as ErrorEvent)
    await expect(pending).rejects.toThrow('reinitialize required')
    await expect(manager.call('load', [])).rejects.toThrow('reinitialize required')
    expect(FakeWorker.instances).toHaveLength(1)
  })

  it('deduplicates transferred buffers and recovers from a postMessage exception', async () => {
    const manager = new WorkerManager()
    const worker = FakeWorker.instances[0]
    worker.emit({ type: 'ready' })
    const data = new Uint8Array([1, 2])
    worker.postMessage.mockImplementationOnce(() => { throw new Error('clone failed') })
    await expect(manager.call('load', [data, data])).rejects.toThrow('clone failed')
    expect(worker.postMessage.mock.calls.at(-1)![1]).toEqual({ transfer: [data.buffer] })
    const pending = manager.call('stats', [])
    await Promise.resolve()
    const { id } = worker.postMessage.mock.calls.at(-1)![0] as { id: number }
    worker.emit({ id, ok: true, result: 1 })
    await expect(pending).resolves.toBe(1)
    manager.destroy()
  })

  it.each([-1, 0.5, Infinity, NaN])('rejects invalid memoryLimit %s before starting a worker', (memoryLimit) => {
    expect(() => new WorkerManager({ memoryLimit })).toThrow('safe integer')
    expect(FakeWorker.instances).toHaveLength(0)
  })
})
