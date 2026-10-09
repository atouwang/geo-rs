import type { Feature, FeatureCollection, Geometry } from 'geojson'
import { computeBBox } from '@geo-rs/core'

export type CanvasGeometry = Geometry | Feature | FeatureCollection

function* geometries(input: CanvasGeometry | null): Generator<Geometry> {
  if (!input) return
  if (input.type === 'Feature') { yield* geometries(input.geometry); return }
  if (input.type === 'FeatureCollection') {
    for (const feature of input.features) yield* geometries(feature)
    return
  }
  if (input.type === 'GeometryCollection') {
    for (const geometry of input.geometries) yield* geometries(geometry)
    return
  }
  yield input
}

export function renderGeometry(ctx: CanvasRenderingContext2D, width: number, height: number, input: CanvasGeometry | null): void {
  ctx.clearRect(0, 0, width, height)
  if (!input || width <= 40 || height <= 40) return
  let bounds: ReturnType<typeof computeBBox>
  try { bounds = computeBBox(input) } catch { return }
  const { minX, minY, maxX, maxY } = bounds
  const scale = Math.min((width - 40) / (maxX - minX || 1), (height - 40) / (maxY - minY || 1))
  const tx = (x: number) => (width - (maxX - minX) * scale) / 2 + (x - minX) * scale
  const ty = (y: number) => (height + (maxY - minY) * scale) / 2 - (y - minY) * scale
  ctx.strokeStyle = '#58a6ff'
  ctx.lineWidth = 1.5
  const path = (coords: number[][]): boolean => {
    if (!coords.length) return false
    ctx.moveTo(tx(coords[0][0]), ty(coords[0][1]))
    for (const [x, y] of coords.slice(1)) ctx.lineTo(tx(x), ty(y))
    return true
  }
  const point = ([x, y]: number[]) => {
    ctx.beginPath(); ctx.arc(tx(x), ty(y), 4, 0, Math.PI * 2)
    ctx.fillStyle = '#58a6ff'; ctx.fill()
  }
  const line = (coords: number[][]) => { ctx.beginPath(); if (path(coords)) ctx.stroke() }
  const polygon = (rings: number[][][]) => {
    ctx.beginPath()
    for (const ring of rings) if (path(ring)) ctx.closePath()
    ctx.fillStyle = 'rgba(88, 166, 255, 0.12)'
    ctx.fill('evenodd'); ctx.stroke()
  }
  for (const geometry of geometries(input)) {
    switch (geometry.type) {
      case 'Point': point(geometry.coordinates); break
      case 'MultiPoint': geometry.coordinates.forEach(point); break
      case 'LineString': line(geometry.coordinates); break
      case 'MultiLineString': geometry.coordinates.forEach(line); break
      case 'Polygon': polygon(geometry.coordinates); break
      case 'MultiPolygon': geometry.coordinates.forEach(polygon); break
    }
  }
}
