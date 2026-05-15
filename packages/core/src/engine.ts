import { WorkerManager, checkWasmSupport, WasmNotSupportedError } from './worker-manager'
import type { EngineConfig, GeoJSON } from './types'

const OP_CODES = {
  AREA: 0x01, LENGTH: 0x02, CENTROID: 0x03, BBOX: 0x04,
  BUFFER: 0x10, SIMPLIFY: 0x11, CONVEX_HULL: 0x12,
  CONTAINS: 0x20, INTERSECTS: 0x21, CROSSES: 0x22,
  WITHIN: 0x23, DISJOINT: 0x24, OVERLAPS: 0x25,
  TOUCHES: 0x26, EQUALS: 0x27,
  UNION: 0x30, INTERSECT: 0x31, DIFFERENCE: 0x32, XOR: 0x33,
} as const

export class GeoEngine {
  private worker: WorkerManager
  private static wasmChecked = false

  private constructor(worker: WorkerManager) {
    this.worker = worker
  }

  static async init(config?: EngineConfig): Promise<GeoEngine> {
    if (!GeoEngine.wasmChecked) {
      const supported = await checkWasmSupport()
      GeoEngine.wasmChecked = true
      if (!supported) throw new WasmNotSupportedError()
    }
    const wm = new WorkerManager(config?.workerUrl)
    await wm.ensureReady()
    return new GeoEngine(wm)
  }

  async load(geojson: GeoJSON): Promise<bigint> {
    const json = typeof geojson === 'string' ? geojson : JSON.stringify(geojson)
    return (await this.call('load', [json])) as bigint
  }

  async read(handle: bigint): Promise<GeoJSON> {
    const json = (await this.call('read', [handle])) as string
    return JSON.parse(json)
  }

  async buffer(handle: bigint, radius: number): Promise<bigint> {
    return (await this.call('execute_unary', [OP_CODES.BUFFER, handle, radius])) as bigint
  }

  async simplify(handle: bigint, tolerance: number): Promise<bigint> {
    return (await this.call('execute_unary', [OP_CODES.SIMPLIFY, handle, tolerance])) as bigint
  }

  async area(handle: bigint): Promise<number> {
    return (await this.call('execute_measure', [OP_CODES.AREA, handle])) as number
  }

  async length(handle: bigint): Promise<number> {
    return (await this.call('execute_measure', [OP_CODES.LENGTH, handle])) as number
  }

  async contains(a: bigint, b: bigint): Promise<boolean> {
    return (await this.call('execute_bool', [OP_CODES.CONTAINS, a, b])) as boolean
  }

  async intersects(a: bigint, b: bigint): Promise<boolean> {
    return (await this.call('execute_bool', [OP_CODES.INTERSECTS, a, b])) as boolean
  }

  async union(a: bigint, b: bigint): Promise<bigint> {
    return (await this.call('execute_binary', [OP_CODES.UNION, a, b])) as bigint
  }

  async intersect(a: bigint, b: bigint): Promise<bigint> {
    return (await this.call('execute_binary', [OP_CODES.INTERSECT, a, b])) as bigint
  }

  async difference(a: bigint, b: bigint): Promise<bigint> {
    return (await this.call('execute_binary', [OP_CODES.DIFFERENCE, a, b])) as bigint
  }

  free(...handles: bigint[]): void {
    for (const h of handles) {
      this.worker.call('free', [h]).catch(() => {})
    }
  }

  freeAll(): void {
    this.worker.call('free_all', []).catch(() => {})
  }

  async stats(): Promise<{ active: number; allocated: number; max: number }> {
    const json = (await this.call('stats', [])) as string
    return JSON.parse(json)
  }

  destroy(): void {
    this.worker.destroy()
  }

  private async call(method: string, args: unknown[]): Promise<unknown> {
    return this.worker.call(method, args)
  }
}
