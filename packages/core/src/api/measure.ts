import { GeoEngine } from '../engine'
import type { Feature, Point, Polygon } from 'geojson'
import type { GeoJSON } from '../types'
import { getSharedEngine, releaseSharedEngine } from './shared'

export async function buffer(
  geom: GeoJSON,
  radius: number,
): Promise<Feature<Polygon>> {
  const engine = await getSharedEngine()
  try {
    const h = await engine.load(geom)
    const resultH = await engine.buffer(h, radius)
    const result = await engine.read(resultH) as Feature<Polygon>
    engine.free(h, resultH)
    return result
  } finally {
    releaseSharedEngine()
  }
}

export async function area(geom: GeoJSON): Promise<number> {
  const engine = await getSharedEngine()
  try {
    const h = await engine.load(geom)
    const result = await engine.area(h)
    engine.free(h)
    return result
  } finally {
    releaseSharedEngine()
  }
}

export async function length(geom: GeoJSON): Promise<number> {
  const engine = await getSharedEngine()
  try {
    const h = await engine.load(geom)
    const result = await engine.length(h)
    engine.free(h)
    return result
  } finally {
    releaseSharedEngine()
  }
}

export async function centroid(geom: GeoJSON): Promise<Feature<Point>> {
  const engine = await getSharedEngine()
  try {
    const h = await engine.load(geom)
    const ch = await engine.centroid(h)
    const result = await engine.read(ch) as Feature<Point>
    engine.free(h, ch)
    return result
  } finally {
    releaseSharedEngine()
  }
}

export async function bbox(geom: GeoJSON): Promise<{ minX: number; minY: number; maxX: number; maxY: number }> {
  const engine = await getSharedEngine()
  try {
    const h = await engine.load(geom)
    const result = await engine.bbox(h)
    engine.free(h)
    return result
  } finally {
    releaseSharedEngine()
  }
}
