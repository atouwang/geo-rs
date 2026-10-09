import { readFileSync } from 'node:fs'
import { beforeAll, describe, expect, it } from 'vitest'
import { encode, decode } from '@msgpack/msgpack'
import { Engine, initSync } from '../../../../crates/geo-wasm/pkg/geo_wasm'

beforeAll(() => {
  initSync({ module: readFileSync(new URL('../../../../crates/geo-wasm/pkg/geo_wasm_bg.wasm', import.meta.url)) })
})

describe('real WASM and JavaScript wire format', () => {
  it('deduplicates signed-zero coordinates even with a full memory budget', () => {
    const engine = new Engine(32n)
    try {
      // Force floating-point encoding so MessagePack preserves the sign of zero.
      const data = encode({ type: 'Point', coordinates: [-0, 0] }, { forceIntegerToFloat: true })
      expect(Object.is((decode(data) as { coordinates: number[] }).coordinates[0], -0)).toBe(true)
      const first = engine.load(data)
      const duplicate = engine.load(encode({ type: 'Point', coordinates: [0, -0] }, { forceIntegerToFloat: true }))
      expect(duplicate).toBe(first)
      expect(JSON.parse(engine.stats())).toMatchObject({ active: 1, refs: 2, allocated: 32 })
      engine.release(first)
      expect(decode(engine.read(duplicate))).toMatchObject({ type: 'Feature' })
      engine.release(duplicate)
      expect(JSON.parse(engine.stats())).toMatchObject({ active: 0, refs: 0, allocated: 0 })
    } finally { engine.free() }
  })
  it('balances arena stats after budget failure, partial release, clear and slot reuse', () => {
    const engine = new Engine(64n)
    const data = (x: number) => encode({ type: 'Point', coordinates: [x, 0] })
    const stats = () => JSON.parse(engine.stats())
    try {
      const a = engine.load(data(1)), duplicate = engine.load(data(1)), b = engine.load(data(2))
      expect(stats()).toEqual({ active: 2, refs: 3, allocated: 64, max: 64 })
      expect(() => engine.load(data(3))).toThrow('Memory')
      expect(stats()).toMatchObject({ active: 2, refs: 3, allocated: 64 })
      engine.release(a)
      expect(stats()).toMatchObject({ active: 2, refs: 2, allocated: 64 })
      engine.release(b)
      const reused = engine.load(data(3))
      expect(reused).not.toBe(b)
      expect(stats()).toMatchObject({ active: 2, refs: 2, allocated: 64 })
      expect(() => engine.release(b)).toThrow()
      expect(stats().refs).toBe(2)
      engine.free_all()
      expect(stats()).toEqual({ active: 0, refs: 0, allocated: 0, max: 64 })
      expect(() => engine.read(duplicate)).toThrow()
      const fresh = engine.load(data(1))
      expect(fresh).not.toBe(a)
      expect(stats()).toMatchObject({ active: 1, refs: 1, allocated: 32 })
      engine.release(fresh)
      expect(stats()).toMatchObject({ active: 0, refs: 0, allocated: 0 })
    } finally { engine.free() }
  })
  it('rejects invalid isolines without allocating output or poisoning the engine', () => {
    const engine = new Engine()
    try {
      const h = engine.load(encode({ type: 'MultiPoint', coordinates: [[0,0],[1,0],[0,1]] }))
      const before = JSON.parse(engine.stats())
      expect(() => engine.isolines(h, '[0,1]', '[0.25]')).toThrow('values')
      expect(JSON.parse(engine.stats())).toEqual(before)
      for (const [values, breaks] of [
        ['[0,1,null]', '[0.25]'],
        ['[0,1,1]', '[null]'],
        ['[0,1,1]', JSON.stringify(Array.from({ length: 100001 }, () => 0.25))],
      ]) {
        expect(() => engine.isolines(h, values, breaks)).toThrow()
        expect(JSON.parse(engine.stats())).toEqual(before)
      }
      const result = engine.isolines(h, '[0,1e-13,1e-13]', '[2.5e-14]')
      const feature = decode(engine.read(result)) as { geometry: { coordinates: number[][][] } }
      expect(feature.geometry.coordinates).toHaveLength(1)
      for (const [x,y] of feature.geometry.coordinates[0]) expect(x+y).toBeCloseTo(0.25, 12)
      engine.release(result)
      expect(JSON.parse(engine.stats())).toEqual(before)
      expect(decode(engine.read(h))).toMatchObject({ geometry: { type: 'MultiPoint' } })
      engine.release(h)
      expect(JSON.parse(engine.stats())).toMatchObject({ active: 0, refs: 0, allocated: 0 })
    } finally { engine.free() }
  })
  it('exports finite isoline coordinates for extreme finite inputs', () => {
    const engine = new Engine()
    try {
      for (const [coordinates, expectedX] of [
        [[[-Number.MAX_VALUE,0],[Number.MAX_VALUE,0],[-Number.MAX_VALUE,1]], 0],
        [[[Number.MAX_VALUE,0],[Number.MAX_VALUE,0],[Number.MAX_VALUE,1]], Number.MAX_VALUE],
      ] as [number[][], number][]) {
        const h = engine.load(encode({ type: 'MultiPoint', coordinates }))
        const result = engine.isolines(h, JSON.stringify([-Number.MAX_VALUE, Number.MAX_VALUE, -Number.MAX_VALUE]), '[0]')
        const feature = decode(engine.read(result)) as { geometry: { coordinates: number[][][] } }
        expect(feature.geometry.coordinates).toHaveLength(1)
        for (const [x,y] of feature.geometry.coordinates[0]) {
          expect(x).toBe(expectedX)
          expect(Number.isFinite(y)).toBe(true)
          expect(y).toBeGreaterThanOrEqual(0)
          expect(y).toBeLessThanOrEqual(1)
        }
        engine.release(result); engine.release(h)
        expect(JSON.parse(engine.stats())).toMatchObject({ active: 0, refs: 0, allocated: 0 })
      }
    } finally { engine.free() }
  })
  it('bounds Voronoi output and preserves input after invalid bounds', () => {
    const engine = new Engine()
    try {
      const h = engine.load(encode({ type: 'MultiPoint', coordinates: [[0,0],[5,0],[2.5,5]] }))
      expect(() => engine.voronoi(h, JSON.stringify({ min_x: 0, min_y: 0, max_x: 0, max_y: 6 }))).toThrow('bbox')
      const bounds = JSON.stringify({ min_x: -1, min_y: -1, max_x: 6, max_y: 6 })
      const a = engine.voronoi(h, bounds), b = engine.voronoi(h, bounds)
      expect(decode(engine.read(a))).toEqual(decode(engine.read(b)))
      const bbox = JSON.parse(engine.bbox(a))
      expect(bbox.min_x).toBeGreaterThanOrEqual(-1); expect(bbox.min_y).toBeGreaterThanOrEqual(-1)
      expect(bbox.max_x).toBeLessThanOrEqual(6); expect(bbox.max_y).toBeLessThanOrEqual(6)
      engine.release(a); engine.release(b); engine.release(h)
      expect(JSON.parse(engine.stats())).toMatchObject({ active: 0, refs: 0, allocated: 0 })
    } finally { engine.free() }
  })
  it('rejects malformed inputs without poisoning the engine', () => {
    const engine = new Engine()
    try {
      for (const geometry of [
        { type: 'Polygon', coordinates: [[]] },
        { type: 'MultiPolygon', coordinates: [[[[0, 0], [1, 0], [1, 1], [0, 0]]], []] },
        { type: 'MultiPoint', coordinates: [[0, 0], [NaN, 1]] },
      ]) {
        expect(() => engine.load(encode(geometry))).toThrow()
        const valid = engine.load(encode({ type: 'Point', coordinates: [1, 2] }))
        expect(decode(engine.read(valid))).toMatchObject({ geometry: { coordinates: [1, 2] } })
        engine.release(valid)
        expect(JSON.parse(engine.stats())).toMatchObject({ active: 0, allocated: 0 })
      }
      const polygon = engine.load(encode({ type: 'Polygon', coordinates: [[[0, 0], [1, 0], [1, 1], [0, 0]]] }))
      expect(() => engine.execute_unary(0x10, polygon, NaN)).toThrow('finite')
      expect(engine.execute_measure(0x01, polygon)).toBeCloseTo(0.5)
      engine.release(polygon)
    } finally { engine.free() }
  })
  it('imports JS GeoJSON, operates, exports Features, and reuses released slots', () => {
    const engine = new Engine()
    try {
      for (let i = 0; i < 20; i++) {
        const polygon = { type: 'Polygon', coordinates: [[[0, 0], [1, 0], [1, 1], [0, 1], [0, 0]]] }
        const h = engine.load(encode(polygon))
        expect(engine.execute_measure(0x01, h)).toBeCloseTo(1)
        expect(JSON.parse(engine.bbox(h))).toEqual({ min_x: 0, min_y: 0, max_x: 1, max_y: 1 })
        const centroid = engine.execute_unary(0x03, h, 0)
        expect(decode(engine.read(centroid))).toMatchObject({
          type: 'Feature', geometry: { type: 'Point', coordinates: [0.5, 0.5] },
        })
        engine.release(h)
        engine.release(centroid)
        expect(JSON.parse(engine.stats())).toMatchObject({ active: 0, refs: 0, allocated: 0 })
      }
    } finally { engine.free() }
  })

  it('accepts Features and balances deduplicated inputs', () => {
    const engine = new Engine()
    try {
      const data = encode({ type: 'Feature', properties: {}, geometry: { type: 'Point', coordinates: [1, 2] } })
      const a = engine.load(data)
      const b = engine.load(data)
      expect(a).toBe(b)
      engine.release(a)
      expect(decode(engine.read(b))).toMatchObject({ geometry: { coordinates: [1, 2] } })
      engine.release(b)
      expect(() => engine.read(a)).toThrow()
      expect(() => engine.read(0n)).toThrow()
    } finally { engine.free() }
  })

  it('enforces an explicit zero memory budget', () => {
    const engine = new Engine(0n)
    try {
      expect(() => engine.load(encode({ type: 'Point', coordinates: [1, 2] }))).toThrow()
      expect(JSON.parse(engine.stats()).allocated).toBe(0)
    } finally { engine.free() }
  })
})
