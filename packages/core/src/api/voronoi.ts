import type { Feature, FeatureCollection, GeometryCollection, Point, Polygon } from 'geojson'
import { getSharedEngine, releaseSharedEngine } from './shared'

export async function voronoi(
  points: FeatureCollection<Point>,
  bbox: { minX: number; minY: number; maxX: number; maxY: number },
): Promise<FeatureCollection<Polygon>> {
  const engine = await getSharedEngine()
  const handles: bigint[] = []
  try {
    const multiPoint = {
      type: 'MultiPoint' as const,
      coordinates: points.features.map(f => f.geometry.coordinates),
    }
    const h = await engine.load(multiPoint)
    handles.push(h)
    const vh = await engine.voronoi(h, bbox)
    handles.push(vh)
    const result = await engine.read(vh) as Feature<GeometryCollection>
    return {
      type: 'FeatureCollection',
      features: result.geometry.geometries.map(geometry => ({
        type: 'Feature', properties: {}, geometry: geometry as Polygon,
      })),
    }
  } finally {
    engine.free(...handles)
    releaseSharedEngine()
  }
}
