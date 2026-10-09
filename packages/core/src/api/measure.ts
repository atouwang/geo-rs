import type { Feature, Point, MultiPolygon } from 'geojson'
import type { GeoJSON } from '../types'
import { getSharedEngine, releaseSharedEngine } from './shared'

export async function buffer(
  geom: GeoJSON,
  radius: number,
): Promise<Feature<MultiPolygon>> {
  const engine = await getSharedEngine()
  const handles: bigint[] = []
  try {
    const h = await engine.load(geom)
    handles.push(h)
    const resultH = await engine.buffer(h, radius)
    handles.push(resultH)
    const result = await engine.read(resultH) as Feature<MultiPolygon>
    return result
  } finally {
    engine.free(...handles)
    releaseSharedEngine()
  }
}

export async function area(geom: GeoJSON): Promise<number> {
  const engine = await getSharedEngine()
  const handles: bigint[] = []
  try {
    const h = await engine.load(geom)
    handles.push(h)
    const result = await engine.area(h)
    return result
  } finally {
    engine.free(...handles)
    releaseSharedEngine()
  }
}

export async function length(geom: GeoJSON): Promise<number> {
  const engine = await getSharedEngine()
  const handles: bigint[] = []
  try {
    const h = await engine.load(geom)
    handles.push(h)
    const result = await engine.length(h)
    return result
  } finally {
    engine.free(...handles)
    releaseSharedEngine()
  }
}

export async function centroid(geom: GeoJSON): Promise<Feature<Point>> {
  const engine = await getSharedEngine()
  const handles: bigint[] = []
  try {
    const h = await engine.load(geom)
    handles.push(h)
    const ch = await engine.centroid(h)
    handles.push(ch)
    const result = await engine.read(ch) as Feature<Point>
    return result
  } finally {
    engine.free(...handles)
    releaseSharedEngine()
  }
}

export async function bbox(geom: GeoJSON): Promise<{ minX: number; minY: number; maxX: number; maxY: number }> {
  const engine = await getSharedEngine()
  const handles: bigint[] = []
  try {
    const h = await engine.load(geom)
    handles.push(h)
    const result = await engine.bbox(h)
    return result
  } finally {
    engine.free(...handles)
    releaseSharedEngine()
  }
}
