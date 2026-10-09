import { describe, expect, it, vi } from 'vitest'
import type { CanvasGeometry } from '../render-geometry'
import { renderGeometry } from '../render-geometry'

function context() {
  return { clearRect: vi.fn(), beginPath: vi.fn(), moveTo: vi.fn(), lineTo: vi.fn(), closePath: vi.fn(),
    fill: vi.fn(), stroke: vi.fn(), arc: vi.fn() } as unknown as CanvasRenderingContext2D
}
const donut: CanvasGeometry = { type: 'Feature', properties: {}, geometry: { type: 'Polygon', coordinates: [
  [[0,0],[10,0],[10,10],[0,10],[0,0]], [[3,3],[7,3],[7,7],[3,7],[3,3]],
] } }
describe('GeoCanvas', () => {
  it('fills outer and inner rings as one evenodd path', () => {
    const ctx = context(); renderGeometry(ctx, 800, 600, donut)
    expect(ctx.closePath).toHaveBeenCalledTimes(2)
    expect(ctx.fill).toHaveBeenCalledExactlyOnceWith('evenodd')
  })
  it('renders Features, nested collections, MultiPoint and MultiLineString', () => {
    const ctx = context()
    renderGeometry(ctx, 800, 600, { type: 'FeatureCollection', features: [
      { type: 'Feature', properties: {}, geometry: null },
      { type: 'Feature', properties: {}, geometry: { type: 'GeometryCollection', geometries: [
        { type: 'MultiPoint', coordinates: [[0,0],[2,2]] },
        { type: 'MultiLineString', coordinates: [[[0,0],[2,2]]] },
      ] } },
    ] })
    expect(ctx.arc).toHaveBeenCalledTimes(2)
    expect(ctx.stroke).toHaveBeenCalledOnce()
  })
  it('clears null, empty and nonfinite updates safely', () => {
    const ctx = context()
    for (const input of [null, { type: 'LineString', coordinates: [] }, { type: 'Point', coordinates: [NaN,1] }] as (CanvasGeometry | null)[]) {
      renderGeometry(ctx, 800, 600, input)
    }
    expect(ctx.clearRect).toHaveBeenCalledTimes(3)
    expect(ctx.arc).not.toHaveBeenCalled()
  })
})

