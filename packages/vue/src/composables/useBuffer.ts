import { ref, shallowRef, unref, type MaybeRef } from 'vue'
import { buffer as coreBuffer } from '@geo-rs/core'
import type { Feature, Polygon } from 'geojson'

export function useBuffer() {
  const loading = ref(false)
  const error = ref<Error | null>(null)
  const result = shallowRef<Feature<Polygon> | null>(null)

  async function execute(
    geom: MaybeRef<Feature>,
    options: MaybeRef<{ radius: number; units?: 'meters' | 'kilometers' | 'miles' }>,
  ) {
    loading.value = true
    error.value = null
    try {
      result.value = await coreBuffer(unref(geom), unref(options).radius)
    } catch (e) {
      error.value = e as Error
    } finally {
      loading.value = false
    }
  }

  return { execute, result, loading, error }
}
