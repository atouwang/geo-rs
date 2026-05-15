import type { Feature, Polygon, MultiPolygon } from 'geojson'

export async function union(_a: Feature, _b: Feature): Promise<Feature<Polygon | MultiPolygon> | null> {
  throw new Error('Not yet implemented')
}

export async function intersect(_a: Feature, _b: Feature): Promise<Feature<Polygon | MultiPolygon> | null> {
  throw new Error('Not yet implemented')
}

export async function difference(_a: Feature, _b: Feature): Promise<Feature<Polygon | MultiPolygon> | null> {
  throw new Error('Not yet implemented')
}
