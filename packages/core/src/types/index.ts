import type { Feature, Polygon, MultiPolygon, Point, LineString, Geometry } from 'geojson'

export type GeoJSON = Feature | Geometry

export { Feature, Polygon, MultiPolygon, Point, LineString }

export interface EngineConfig {
  memoryLimit?: number
  workerUrl?: string | URL
}

export interface GeoResult<T = GeoJSON> {
  data: T
  duration: number
}

export interface Engine {
  load(geojson: GeoJSON): Promise<bigint>
  buffer(handle: bigint, radius: number): Promise<bigint>
  intersect(a: bigint, b: bigint): Promise<bigint>
  union(a: bigint, b: bigint): Promise<bigint>
  contains(a: bigint, b: bigint): Promise<boolean>
  read(handle: bigint): Promise<GeoJSON>
  free(...handles: bigint[]): void
  destroy(): void
}
