import { voronoi as coreVoronoi, computeBBox } from '@geo-rs/core'
import type { FeatureCollection, Point } from 'geojson'
import { useAsyncState } from './async-state'

export function useVoronoi() {
  return useAsyncState(async (points: FeatureCollection<Point>, bounds?: ReturnType<typeof computeBBox>) => {
    const bbox = bounds ?? computeBBox(points)
    if (!bounds) {
      // Keep collinear/coincident sites usable with automatically chosen bounds.
      const padding = (Math.max(bbox.maxX - bbox.minX, bbox.maxY - bbox.minY) || 1) * 0.05
      if (bbox.minX === bbox.maxX) { bbox.minX -= padding; bbox.maxX += padding }
      if (bbox.minY === bbox.maxY) { bbox.minY -= padding; bbox.maxY += padding }
    }
    return coreVoronoi(points, bbox)
  })
}
