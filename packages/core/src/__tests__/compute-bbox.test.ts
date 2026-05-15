import { describe, it, expect } from 'vitest'
import { computeBBox } from '../engine'

describe('computeBBox', () => {
  it('computes bbox for a Point', () => {
    const geom = { type: 'Point', coordinates: [10, 20] }
    expect(computeBBox(geom)).toEqual({ minX: 10, minY: 20, maxX: 10, maxY: 20 })
  })

  it('computes bbox for a Polygon', () => {
    const geom = {
      type: 'Polygon',
      coordinates: [[[0, 0], [2, 0], [2, 2], [0, 2], [0, 0]]],
    }
    expect(computeBBox(geom)).toEqual({ minX: 0, minY: 0, maxX: 2, maxY: 2 })
  })

  it('computes bbox for a MultiPolygon', () => {
    const geom = {
      type: 'MultiPolygon',
      coordinates: [
        [[[0, 0], [1, 0], [1, 1], [0, 1], [0, 0]]],
        [[[5, 5], [6, 5], [6, 6], [5, 6], [5, 5]]],
      ],
    }
    expect(computeBBox(geom)).toEqual({ minX: 0, minY: 0, maxX: 6, maxY: 6 })
  })

  it('computes bbox for a LineString', () => {
    const geom = {
      type: 'LineString',
      coordinates: [[0, 0], [1, 2], [3, 1]],
    }
    expect(computeBBox(geom)).toEqual({ minX: 0, minY: 0, maxX: 3, maxY: 2 })
  })

  it('computes bbox for nested GeometryCollection', () => {
    const geom = {
      type: 'GeometryCollection',
      geometries: [
        { type: 'Point', coordinates: [0, 0] },
        { type: 'Point', coordinates: [5, 5] },
      ],
    }
    expect(computeBBox(geom)).toEqual({ minX: 0, minY: 0, maxX: 5, maxY: 5 })
  })
})
