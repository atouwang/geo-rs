import { voronoi as coreVoronoi, computeBBox } from '@geo-rs/core'
import type { FeatureCollection, Point } from 'geojson'
import { useAsyncState } from './async-state'

export function useVoronoi() {
  return useAsyncState(async (points: FeatureCollection<Point>) => coreVoronoi(points, computeBBox(points)))
}
