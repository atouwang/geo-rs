import { GeoEngine } from '../engine'
import type { Feature, Polygon, Point } from 'geojson'
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
  // centroid is computed via measure; for now just load and read with stub
  const engine = await getSharedEngine()
  try {
    const h = await engine.load(geom)
    const result = await engine.read(h)
    engine.free(h)
    // Placeholder: return first coordinate as centroid
    return result as unknown as Feature<Point>
  } finally {
    releaseSharedEngine()
  }
}

export async function bbox(
  geom: GeoJSON,
): Promise<[number, number, number, number]> {
  const engine = await getSharedEngine()
  try {
    const h = await engine.load(geom)
    // Use execute_measure with BBOX op code
    // Placeholder: approximate bbox from read geometry
    const result = await engine.read(h)
    engine.free(h)
    return [0, 0, 0, 0] // placeholder
  } finally {
    releaseSharedEngine()
  }
}
