import { unref, type MaybeRef } from 'vue'
import { buffer as coreBuffer } from '@geo-rs/core'
import type { Feature } from 'geojson'
import { useAsyncState } from './async-state'

export function useBuffer() {
  return useAsyncState(async (
    geom: MaybeRef<Feature>,
    // Units are reserved until the API supports metric/geodesic conversion.
    options: MaybeRef<{ radius: number; units?: 'meters' | 'kilometers' | 'miles' }>,
  ) => {
    const config = unref(options)
    if (config.units !== undefined) throw new Error('Buffer uses input coordinate units; metric units are not supported')
    return coreBuffer(unref(geom), config.radius)
  })
}
