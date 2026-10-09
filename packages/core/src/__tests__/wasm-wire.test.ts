import { readFileSync } from 'node:fs'
import { beforeAll, describe, expect, it } from 'vitest'
import { encode, decode } from '@msgpack/msgpack'
import { Engine, initSync } from '../../../../crates/geo-wasm/pkg/geo_wasm'

beforeAll(() => {
  initSync({ module: readFileSync(new URL('../../../../crates/geo-wasm/pkg/geo_wasm_bg.wasm', import.meta.url)) })
})

describe('real WASM and JavaScript wire format', () => {
  it('imports JS GeoJSON, operates, exports Features, and reuses released slots', () => {
    const engine = new Engine()
    try {
      for (let i = 0; i < 20; i++) {
        const polygon = { type: 'Polygon', coordinates: [[[0, 0], [1, 0], [1, 1], [0, 1], [0, 0]]] }
        const h = engine.load(encode(polygon))
        expect(engine.execute_measure(0x01, h)).toBeCloseTo(1)
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
