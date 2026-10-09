import { beforeEach, describe, expect, it, vi } from 'vitest'

const mock = vi.hoisted(() => ({ init: vi.fn(), load: vi.fn(), read: vi.fn(),
  buffer: vi.fn(), union: vi.fn(), contains: vi.fn(), voronoi: vi.fn(),
  free: vi.fn(), destroy: vi.fn(),
}))
vi.mock('../engine', () => ({ GeoEngine: { init: mock.init } }))
import { buffer } from '../api/measure'
import { union } from '../api/set-ops'
import { contains } from '../api/predicates'
import { voronoi } from '../api/voronoi'
import { getSharedEngine, shutdownSharedEngine } from '../api/shared'

const point = { type: 'Point' as const, coordinates: [1, 2] }
describe('stateless API resource cleanup', () => {
  beforeEach(() => {
    shutdownSharedEngine()
    vi.resetAllMocks()
    mock.init.mockResolvedValue(mock)
    mock.load.mockResolvedValue(1n)
    mock.buffer.mockResolvedValue(2n)
    mock.union.mockResolvedValue(3n)
    mock.read.mockRejectedValue(new Error('read failed'))
  })

  it('frees input and output when reading the result fails', async () => {
    await expect(buffer(point, 10)).rejects.toThrow('read failed')
    expect(mock.free).toHaveBeenCalledWith(1n, 2n)
  })

  it('frees the first input when loading the second fails', async () => {
    mock.load.mockResolvedValueOnce(1n).mockRejectedValueOnce(new Error('load failed'))
    await expect(union(point, point)).rejects.toThrow('load failed')
    expect(mock.free).toHaveBeenCalledWith(1n)
  })

  it('frees both references when a predicate fails on identical inputs', async () => {
    mock.contains.mockRejectedValue(new Error('predicate failed'))
    await expect(contains(point, point)).rejects.toThrow('predicate failed')
    expect(mock.free).toHaveBeenCalledWith(1n, 1n)
  })

  it('turns the WASM Voronoi collection into GeoJSON features', async () => {
    const polygon = { type: 'Polygon', coordinates: [[[0, 0], [1, 0], [0, 1], [0, 0]]] }
    mock.voronoi.mockResolvedValue(2n)
    mock.read.mockResolvedValue({ type: 'Feature', geometry: { type: 'GeometryCollection', geometries: [polygon] } })
    const result = await voronoi({ type: 'FeatureCollection', features: [] }, { minX: 0, minY: 0, maxX: 1, maxY: 1 })
    expect(result).toEqual({ type: 'FeatureCollection', features: [{ type: 'Feature', properties: {}, geometry: polygon }] })
    expect(mock.free).toHaveBeenCalledWith(1n, 2n)
  })

  it('retries after initialization fails, sharing concurrent initialization', async () => {
    mock.init.mockRejectedValueOnce(new Error('init failed')).mockResolvedValue(mock)
    await expect(getSharedEngine()).rejects.toThrow('init failed')
    const [a, b] = await Promise.all([getSharedEngine(), getSharedEngine()])
    expect(a).toBe(b)
    expect(mock.init).toHaveBeenCalledTimes(2)
  })

  it('destroys an engine whose initialization finishes after shutdown', async () => {
    let resolve!: (engine: unknown) => void
    mock.init.mockReturnValue(new Promise(r => { resolve = r }))
    const pending = getSharedEngine()
    shutdownSharedEngine()
    resolve(mock)
    await expect(pending).rejects.toThrow('shut down')
    expect(mock.destroy).toHaveBeenCalledOnce()
  })
})
