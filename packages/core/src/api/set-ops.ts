import { GeoEngine } from '../engine'
import type { Feature, Polygon, MultiPolygon } from 'geojson'
import type { GeoJSON } from '../types'
import { getSharedEngine, releaseSharedEngine } from './shared'

export async function union(
  a: GeoJSON,
  b: GeoJSON,
): Promise<Feature<Polygon | MultiPolygon> | null> {
  const engine = await getSharedEngine()
  try {
    const ha = await engine.load(a)
    const hb = await engine.load(b)
    const resultH = await engine.union(ha, hb)
    const result = await engine.read(resultH) as Feature<Polygon | MultiPolygon>
    engine.free(ha, hb, resultH)
    return result
  } finally {
    releaseSharedEngine()
  }
}

export async function intersect(
  a: GeoJSON,
  b: GeoJSON,
): Promise<Feature<Polygon | MultiPolygon> | null> {
  const engine = await getSharedEngine()
  try {
    const ha = await engine.load(a)
    const hb = await engine.load(b)
    const resultH = await engine.intersect(ha, hb)
    const result = await engine.read(resultH) as Feature<Polygon | MultiPolygon>
    engine.free(ha, hb, resultH)
    return result
  } finally {
    releaseSharedEngine()
  }
}

export async function difference(
  a: GeoJSON,
  b: GeoJSON,
): Promise<Feature<Polygon | MultiPolygon> | null> {
  const engine = await getSharedEngine()
  try {
    const ha = await engine.load(a)
    const hb = await engine.load(b)
    const resultH = await engine.difference(ha, hb)
    const result = await engine.read(resultH) as Feature<Polygon | MultiPolygon>
    engine.free(ha, hb, resultH)
    return result
  } finally {
    releaseSharedEngine()
  }
}
