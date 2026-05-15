import type { Feature, Polygon, Point } from 'geojson'

export async function buffer(_geom: Feature, _radius: number): Promise<Feature<Polygon>> {
  throw new Error('Not yet implemented')
}

export async function area(_geom: Feature): Promise<number> {
  throw new Error('Not yet implemented')
}

export async function length(_geom: Feature): Promise<number> {
  throw new Error('Not yet implemented')
}

export async function centroid(_geom: Feature): Promise<Feature<Point>> {
  throw new Error('Not yet implemented')
}

export async function bbox(_geom: Feature): Promise<[number, number, number, number]> {
  throw new Error('Not yet implemented')
}
