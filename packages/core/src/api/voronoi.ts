import type { FeatureCollection, Point, Polygon } from 'geojson'
import { getSharedEngine, releaseSharedEngine } from './shared'

export async function voronoi(
  points: FeatureCollection<Point>,
  bbox: { minX: number; minY: number; maxX: number; maxY: number },
): Promise<FeatureCollection<Polygon>> {
  const engine = await getSharedEngine()
  try {
    const multiPoint = {
      type: 'MultiPoint' as const,
      coordinates: points.features.map(f => f.geometry.coordinates),
    }
    const h = await engine.load(multiPoint)
    const vh = await engine.voronoi(h, bbox)
    const result = await engine.read(vh) as unknown as FeatureCollection<Polygon>
    engine.free(h, vh)
    return result
  } finally {
    releaseSharedEngine()
  }
}
