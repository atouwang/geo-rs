import { GeoEngine } from '../engine'

let sharedEngine: GeoEngine | null = null
let refCount = 0
let initPromise: Promise<GeoEngine> | null = null

export async function getSharedEngine(): Promise<GeoEngine> {
  refCount++
  if (sharedEngine) return sharedEngine
  if (!initPromise) {
    initPromise = GeoEngine.init().then((engine) => {
      sharedEngine = engine
      return engine
    })
  }
  return initPromise
}

export function releaseSharedEngine(): void {
  refCount--
  if (refCount <= 0 && sharedEngine) {
    refCount = 0
    // Keep the shared engine alive for reuse
    // sharedEngine.destroy() would be called on explicit shutdown
  }
}

export function shutdownSharedEngine(): void {
  if (sharedEngine) {
    sharedEngine.destroy()
    sharedEngine = null
    initPromise = null
  }
}
