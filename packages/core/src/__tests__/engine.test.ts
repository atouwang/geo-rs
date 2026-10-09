import { beforeEach, describe, expect, it, vi } from 'vitest'
import { encode } from '@msgpack/msgpack'

const mock = vi.hoisted(() => ({ call: vi.fn(), destroy: vi.fn(), ready: vi.fn(), support: vi.fn() }))
vi.mock('../worker-manager', () => ({
  WorkerManager: class {
    call = mock.call
    destroy = mock.destroy
    ensureReady = mock.ready
  },
  checkWasmSupport: mock.support,
  WasmNotSupportedError: class extends Error {},
}))
import { GeoEngine } from '../engine'

const point = { type: 'Point' as const, coordinates: [1, 2] }
describe('GeoEngine handle lifecycle', () => {
  beforeEach(() => {
    vi.resetAllMocks()
    mock.support.mockResolvedValue(true)
    mock.ready.mockResolvedValue(undefined)
    mock.call.mockImplementation(async (method: string) => {
      if (method === 'load') return 1n
      if (method === 'read') return encode(point)
      if (method === 'execute_unary' || method === 'execute_binary' || method === 'voronoi') return 2n
      return 0
    })
  })

  it.each(['buffer', 'simplify', 'centroid', 'union', 'intersect', 'difference', 'voronoi'] as const)(
    'registers %s results for reading and releasing', async (method) => {
      const engine = await GeoEngine.init()
      const h = await engine.load(point)
      let result: bigint
      if (method === 'buffer' || method === 'simplify') result = await engine[method](h, 1)
      else if (method === 'centroid') result = await engine.centroid(h)
      else if (method === 'voronoi') result = await engine.voronoi(h, { minX: 0, minY: 0, maxX: 5, maxY: 5 })
      else result = await engine[method](h, h)
      expect(await engine.read(result)).toEqual(point)
      engine.free(h, result)
      await Promise.resolve()
      expect(mock.call).toHaveBeenCalledWith('free', [result])
      expect(engine.getHandleStats().active).toBe(0)
    },
  )

  it('balances references when identical inputs or outputs share a handle', async () => {
    mock.call.mockResolvedValue(1n)
    const engine = await GeoEngine.init()
    const a = await engine.load(point)
    const b = await engine.load(point)
    const c = await engine.centroid(a)
    engine.free(a, b)
    expect(engine.getHandleStats().active).toBe(1)
    engine.free(c)
    await expect(engine.area(a)).rejects.toThrow('already freed')
    expect(engine.getHandleStats().active).toBe(0)
  })

  it('rejects unknown inputs before dispatching an operation', async () => {
    const engine = await GeoEngine.init()
    await expect(engine.buffer(999n, 1)).rejects.toThrow('not found')
    expect(mock.call).not.toHaveBeenCalled()
  })

  it('terminates workers when initialization fails', async () => {
    mock.ready.mockRejectedValue(new Error('init failed'))
    await expect(GeoEngine.init()).rejects.toThrow('init failed')
    expect(mock.destroy).toHaveBeenCalledOnce()
  })

  it('sends the Rust bbox field names', async () => {
    const engine = await GeoEngine.init()
    const h = await engine.load(point)
    await engine.voronoi(h, { minX: 0, minY: 1, maxX: 2, maxY: 3 })
    expect(mock.call).toHaveBeenCalledWith('voronoi', [h, '{"min_x":0,"min_y":1,"max_x":2,"max_y":3}'])
  })

  it('does not register a stale result after freeAll runs during a load', async () => {
    let resolve!: (value: bigint) => void
    mock.call.mockImplementation((method: string) => method === 'load'
      ? new Promise<bigint>(r => { resolve = r }) : Promise.resolve())
    const engine = await GeoEngine.init()
    const pending = engine.load(point)
    engine.freeAll()
    resolve(1n)
    await expect(pending).rejects.toThrow('cleared')
    expect(engine.getHandleStats().active).toBe(0)
  })
})
