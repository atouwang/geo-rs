import { ref, shallowRef } from 'vue'
import { voronoi as coreVoronoi, computeBBox } from '@geo-rs/core'
import type { FeatureCollection, Point, Polygon } from 'geojson'

export function useVoronoi() {
  const loading = ref(false)
  const error = ref<Error | null>(null)
  const result = shallowRef<FeatureCollection<Polygon> | null>(null)

  async function execute(points: FeatureCollection<Point>) {
    loading.value = true
    error.value = null
    try {
      const bbox = computeBBox(points)
      result.value = await coreVoronoi(points, bbox)
    } catch (e) {
      error.value = e as Error
    } finally {
      loading.value = false
    }
  }

  return { execute, result, loading, error }
}
