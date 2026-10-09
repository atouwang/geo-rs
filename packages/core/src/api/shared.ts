import { GeoEngine } from '../engine'

let sharedEngine: GeoEngine | null = null
let initPromise: Promise<GeoEngine> | null = null

export async function getSharedEngine(): Promise<GeoEngine> {
  if (sharedEngine) return sharedEngine
  if (!initPromise) {
    const pending = GeoEngine.init().then((engine) => {
      if (initPromise !== pending) {
        engine.destroy()
        throw new Error('Shared engine was shut down during initialization')
      }
      sharedEngine = engine
      return engine
    }).catch((error) => {
      if (initPromise === pending) initPromise = null
      throw error
    })
    initPromise = pending
  }
  return initPromise
}

// Handles are freed by each API; the engine stays alive for reuse.
export function releaseSharedEngine(): void {}

export function shutdownSharedEngine(): void {
  initPromise = null
  sharedEngine?.destroy()
  sharedEngine = null
}
