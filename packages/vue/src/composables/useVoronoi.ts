import { ref, shallowRef } from 'vue'
import type { Feature, FeatureCollection, Point, Polygon } from 'geojson'

export function useVoronoi() {
  const loading = ref(false)
  const error = ref<Error | null>(null)
  const result = shallowRef<FeatureCollection<Polygon> | null>(null)

  async function execute(_points: FeatureCollection<Point>) {
    loading.value = true
    error.value = null
    try {
      result.value = null
    } catch (e) {
      error.value = e as Error
    } finally {
      loading.value = false
    }
  }

  return { execute, result, loading, error }
}
