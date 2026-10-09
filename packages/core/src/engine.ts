import { WorkerManager, checkWasmSupport, WasmNotSupportedError } from './worker-manager'
import { MemoryManager } from './memory-manager'
import { encode, decode } from '@msgpack/msgpack'
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
  private memory = new MemoryManager()
  private memoryRevision = 0
  private static wasmChecked = false

  private constructor(worker: WorkerManager) {
    this.worker = worker
  }

  static async init(config?: EngineConfig): Promise<GeoEngine> {
    if (!GeoEngine.wasmChecked) {
      const supported = await checkWasmSupport()
      if (!supported) throw new WasmNotSupportedError()
      GeoEngine.wasmChecked = true
    }
    const wm = new WorkerManager({
      workerUrl: config?.workerUrl,
      canvas: config?.canvas,
      shared: config?.shared,
      memoryLimit: config?.memoryLimit,
    })
    try {
      await wm.ensureReady()
    } catch (error) {
      wm.destroy()
      throw error
    }
    return new GeoEngine(wm)
  }

  async load(geojson: GeoJSON): Promise<bigint> {
    const data = encode(geojson)
    const handle = (await this.call('load', [data])) as bigint
    return handle
  }

  async read(handle: bigint): Promise<GeoJSON> {
    this.memory.validate(handle)
    const bytes = (await this.call('read', [handle])) as Uint8Array
    return decode(bytes) as GeoJSON
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

  async centroid(handle: bigint): Promise<bigint> {
    return (await this.call('execute_unary', [OP_CODES.CENTROID, handle, 0])) as bigint
  }

  async bbox(handle: bigint): Promise<{ minX: number; minY: number; maxX: number; maxY: number }> {
    const json = await this.call('bbox', [handle]) as string
    const bounds = JSON.parse(json) as { min_x: number; min_y: number; max_x: number; max_y: number }
    return { minX: bounds.min_x, minY: bounds.min_y, maxX: bounds.max_x, maxY: bounds.max_y }
  }

  async contains(a: bigint, b: bigint): Promise<boolean> {
    return (await this.call('execute_bool', [OP_CODES.CONTAINS, a, b])) as boolean
  }

  async intersects(a: bigint, b: bigint): Promise<boolean> {
    return (await this.call('execute_bool', [OP_CODES.INTERSECTS, a, b])) as boolean
  }

  async crosses(a: bigint, b: bigint): Promise<boolean> {
    return (await this.call('execute_bool', [OP_CODES.CROSSES, a, b])) as boolean
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

  async voronoi(pointsHandle: bigint, bbox: { minX: number; minY: number; maxX: number; maxY: number }): Promise<bigint> {
    return (await this.call('voronoi', [pointsHandle, JSON.stringify({
      min_x: bbox.minX, min_y: bbox.minY, max_x: bbox.maxX, max_y: bbox.maxY,
    })])) as bigint
  }

  free(...handles: bigint[]): void {
    for (const h of handles) {
      if (!this.memory.isActive(h)) continue
      this.memory.free(h)
      this.worker.call('free', [h]).catch(() => {})
    }
  }

  freeAll(): void {
    this.memoryRevision++
    this.worker.call('free_all', []).catch(() => {})
    this.memory.clear()
  }

  getHandleStats(): { active: number; freed: number; total: number } {
    return this.memory.stats()
  }

  async stats(): Promise<{ active: number; allocated: number; max: number }> {
    const json = (await this.call('stats', [])) as string
    return JSON.parse(json)
  }

  destroy(): void {
    this.memoryRevision++
    this.worker.destroy()
    this.memory.clear()
  }

  private async call(method: string, args: unknown[]): Promise<unknown> {
    const revision = this.memoryRevision
    const handles = method === 'execute_binary' || method === 'execute_bool'
      ? args.slice(1, 3)
      : method === 'execute_unary' || method === 'execute_measure'
        ? args.slice(1, 2)
        : method === 'read' || method === 'voronoi' || method === 'bbox' ? args.slice(0, 1) : []
    for (const handle of handles) this.memory.validate(handle as bigint)
    const result = await this.worker.call(method, args)
    if (revision !== this.memoryRevision) throw new Error('Engine memory was cleared while the operation was in flight')
    if (method === 'load' || method === 'execute_unary' || method === 'execute_binary' || method === 'voronoi') {
      this.memory.register(result as bigint)
    }
    return result
  }
}

function forEachCoord(geom: unknown, fn: (c: [number, number]) => void): void {
  if (geom == null) return
  const g = geom as Record<string, unknown>
  const type = g.type as string
  if (type === 'Feature') {
    forEachCoord(g.geometry, fn)
  } else if (type === 'FeatureCollection') {
    for (const feature of g.features as unknown[]) forEachCoord(feature, fn)
  } else if (type === 'Point') {
    fn((g.coordinates as [number, number]))
  } else if (type === 'MultiPoint' || type === 'LineString') {
    for (const c of (g.coordinates as number[][])) fn([c[0], c[1]])
  } else if (type === 'MultiLineString' || type === 'Polygon') {
    for (const ring of (g.coordinates as number[][][])) {
      for (const c of ring) fn([c[0], c[1]])
    }
  } else if (type === 'MultiPolygon') {
    for (const poly of (g.coordinates as number[][][][])) {
      for (const ring of poly) {
        for (const c of ring) fn([c[0], c[1]])
      }
    }
  } else if (type === 'GeometryCollection') {
    for (const sub of (g.geometries as Record<string, unknown>[])) forEachCoord(sub, fn)
  }
}

export function computeBBox(geom: unknown): { minX: number; minY: number; maxX: number; maxY: number } {
  let minX = Infinity, minY = Infinity, maxX = -Infinity, maxY = -Infinity
  forEachCoord(geom, ([x, y]) => {
    if (!Number.isFinite(x) || !Number.isFinite(y)) throw new Error('Coordinates must be finite')
    if (x < minX) minX = x; if (y < minY) minY = y
    if (x > maxX) maxX = x; if (y > maxY) maxY = y
  })
  if (!Number.isFinite(minX) || !Number.isFinite(minY) || !Number.isFinite(maxX) || !Number.isFinite(maxY)) {
    throw new Error('Cannot compute bbox for geometry without finite coordinates')
  }
  return { minX, minY, maxX, maxY }
}
