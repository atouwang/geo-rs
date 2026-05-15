import type { Engine, EngineConfig, GeoJSON } from './types'

export class GeoEngine {
  private engine: Engine | null = null

  static async init(_config?: EngineConfig): Promise<GeoEngine> {
    const instance = new GeoEngine()
    const supported = await instance.checkWasm()
    if (!supported) {
      console.warn('[geo-rs] WASM not available, using Turf.js fallback')
    }
    return instance
  }

  private async checkWasm(): Promise<boolean> {
    if (typeof WebAssembly === 'undefined') return false
    try {
      const mod = await WebAssembly.compile(new Uint8Array([0, 97, 115, 109, 1, 0, 0, 0]))
      return mod instanceof WebAssembly.Module
    } catch {
      return false
    }
  }

  get isReady(): boolean { return this.engine !== null }

  async load(_geojson: GeoJSON): Promise<bigint> {
    throw new Error('Engine not initialized')
  }

  async buffer(_handle: bigint, _radius: number): Promise<bigint> {
    throw new Error('Engine not initialized')
  }

  async intersect(_a: bigint, _b: bigint): Promise<bigint> {
    throw new Error('Engine not initialized')
  }

  async union(_a: bigint, _b: bigint): Promise<bigint> {
    throw new Error('Engine not initialized')
  }

  async contains(_a: bigint, _b: bigint): Promise<boolean> {
    throw new Error('Engine not initialized')
  }

  async read(_handle: bigint): Promise<GeoJSON> {
    throw new Error('Engine not initialized')
  }

  free(..._handles: bigint[]): void {}
  destroy(): void { this.engine = null }
}
