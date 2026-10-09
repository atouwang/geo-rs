import type { Feature, Polygon, MultiPolygon } from 'geojson'
import type { GeoJSON } from '../types'
import { getSharedEngine, releaseSharedEngine } from './shared'

export async function union(
  a: GeoJSON,
  b: GeoJSON,
): Promise<Feature<Polygon | MultiPolygon> | null> {
  const engine = await getSharedEngine()
  const handles: bigint[] = []
  try {
    const ha = await engine.load(a)
    handles.push(ha)
    const hb = await engine.load(b)
    handles.push(hb)
    const resultH = await engine.union(ha, hb)
    handles.push(resultH)
    const result = await engine.read(resultH) as Feature<Polygon | MultiPolygon>
    return result
  } finally {
    engine.free(...handles)
    releaseSharedEngine()
  }
}

export async function intersect(
  a: GeoJSON,
  b: GeoJSON,
): Promise<Feature<Polygon | MultiPolygon> | null> {
  const engine = await getSharedEngine()
  const handles: bigint[] = []
  try {
    const ha = await engine.load(a)
    handles.push(ha)
    const hb = await engine.load(b)
    handles.push(hb)
    const resultH = await engine.intersect(ha, hb)
    handles.push(resultH)
    const result = await engine.read(resultH) as Feature<Polygon | MultiPolygon>
    return result
  } finally {
    engine.free(...handles)
    releaseSharedEngine()
  }
}

export async function difference(
  a: GeoJSON,
  b: GeoJSON,
): Promise<Feature<Polygon | MultiPolygon> | null> {
  const engine = await getSharedEngine()
  const handles: bigint[] = []
  try {
    const ha = await engine.load(a)
    handles.push(ha)
    const hb = await engine.load(b)
    handles.push(hb)
    const resultH = await engine.difference(ha, hb)
    handles.push(resultH)
    const result = await engine.read(resultH) as Feature<Polygon | MultiPolygon>
    return result
  } finally {
    engine.free(...handles)
    releaseSharedEngine()
  }
}
