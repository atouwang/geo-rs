import { describe, expect, it, vi } from 'vitest'
import { useVoronoi } from '../useVoronoi'
const api = vi.hoisted(() => ({ voronoi: vi.fn().mockResolvedValue({ type: 'FeatureCollection', features: [] }) }))
vi.mock('@geo-rs/core', async importOriginal => ({ ...await importOriginal<object>(), voronoi: api.voronoi }))
const points = (coordinates: number[][]) => ({ type: 'FeatureCollection' as const, features: coordinates.map(coordinates => ({
  type: 'Feature' as const, properties: {}, geometry: { type: 'Point' as const, coordinates },
})) })
describe('Vue Voronoi bounds', () => {
  it('pads a degenerate axis for collinear sites', async () => {
    const state = useVoronoi(); await state.execute(points([[0,0],[1,0],[2,0]]))
    expect(api.voronoi).toHaveBeenLastCalledWith(expect.anything(), { minX:0, minY:-0.1, maxX:2, maxY:0.1 })
    expect(state.error.value).toBeNull()
  })
  it('chooses a nonzero box for coincident sites', async () => {
    await useVoronoi().execute(points([[1,1],[1,1],[1,1]]))
    expect(api.voronoi).toHaveBeenLastCalledWith(expect.anything(), { minX:0.95, minY:0.95, maxX:1.05, maxY:1.05 })
  })
  it('preserves caller-supplied bounds', async () => {
    const bbox = { minX:-5, minY:-5, maxX:5, maxY:5 }
    await useVoronoi().execute(points([[0,0],[1,0],[2,0]]), bbox)
    expect(api.voronoi).toHaveBeenLastCalledWith(expect.anything(), bbox)
    expect(bbox).toEqual({ minX:-5, minY:-5, maxX:5, maxY:5 })
  })
})
