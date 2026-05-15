import { ref, shallowRef, unref, type MaybeRef } from 'vue'
import type { Feature, Polygon } from 'geojson'

export function useBuffer() {
  const loading = ref(false)
  const error = ref<Error | null>(null)
  const result = shallowRef<Feature<Polygon> | null>(null)

  async function execute(
    _geom: MaybeRef<Feature>,
    _options: MaybeRef<{ radius: number; units?: 'meters' | 'kilometers' | 'miles' }>,
  ) {
    loading.value = true
    error.value = null
    try {
      // pending geo-rs core implementation
      result.value = null
    } catch (e) {
      error.value = e as Error
    } finally {
      loading.value = false
    }
  }

  return { execute, result, loading, error }
}
