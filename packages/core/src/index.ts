export { GeoEngine } from './engine'
export type { Engine, EngineConfig, GeoResult } from './types'

export { buffer, area, length, centroid, bbox } from './api/measure'
export { contains, intersects, crosses } from './api/predicates'
export { union, intersect, difference } from './api/set-ops'
