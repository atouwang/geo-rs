import type { Feature, Geometry, Polygon, MultiPolygon, Point, LineString } from 'geojson'

export type { Feature, Geometry, Polygon, MultiPolygon, Point, LineString }

export type GeoJSON = Feature | Geometry

export interface EngineConfig {
  /** Memory limit for the WASM arena in bytes (default: 256MB) */
  memoryLimit?: number
  /** Custom URL for the WASM engine worker */
  workerUrl?: string | URL
  /** Reserved for future OffscreenCanvas rendering; currently rejected. */
  canvas?: HTMLCanvasElement
  /** Share the worker and compiled module, with a separate arena per client. */
  shared?: boolean
}

export interface WasmEngine {
  load(geojson: Uint8Array): bigint
  read(handle: bigint): Uint8Array
  execute_unary(op_code: number, handle: bigint, param: number): bigint
  execute_binary(op_code: number, handle_a: bigint, handle_b: bigint): bigint
  execute_bool(op_code: number, handle_a: bigint, handle_b: bigint): boolean
  execute_measure(op_code: number, handle: bigint): number
  bbox(handle: bigint): string
  release(handle: bigint): void
  free(): void
  free_all(): void
  stats(): string
}

export enum Units {
  Meters = 'meters',
  Kilometers = 'kilometers',
  Miles = 'miles',
}
