<script setup lang="ts">
import { ref, watch, onMounted } from 'vue'

const props = defineProps<{
  width?: number
  height?: number
  geometry?: Record<string, unknown> | null
}>()

const canvasRef = ref<HTMLCanvasElement>()

function walkCoords(g: unknown, fn: (c: [number, number]) => void) {
  const gm = g as Record<string, unknown>
  const t = gm.type as string
  if (t === 'Point') fn(gm.coordinates as [number, number])
  else if (t === 'MultiPoint' || t === 'LineString')
    for (const c of (gm.coordinates as number[][])) fn([c[0], c[1]])
  else if (t === 'MultiLineString' || t === 'Polygon')
    for (const ring of (gm.coordinates as number[][][]))
      for (const c of ring) fn([c[0], c[1]])
  else if (t === 'MultiPolygon')
    for (const poly of (gm.coordinates as number[][][][]))
      for (const ring of poly) for (const c of ring) fn([c[0], c[1]])
  else if (t === 'GeometryCollection')
    for (const sub of (gm.geometries as Record<string, unknown>[])) walkCoords(sub, fn)
}

function render() {
  const canvas = canvasRef.value
  if (!canvas || !props.geometry) return
  const ctx = canvas.getContext('2d')
  if (!ctx) return

  const w = canvas.width, h = canvas.height
  ctx.clearRect(0, 0, w, h)

  let minX = Infinity, minY = Infinity, maxX = -Infinity, maxY = -Infinity
  walkCoords(props.geometry, ([x, y]) => {
    if (x < minX) minX = x; if (y < minY) minY = y
    if (x > maxX) maxX = x; if (y > maxY) maxY = y
  })
  if (!isFinite(minX)) return

  const pad = 20
  const scale = Math.min(
    (w - pad * 2) / (maxX - minX || 1),
    (h - pad * 2) / (maxY - minY || 1)
  )
  const tx = (x: number) => pad + (x - minX) * scale
  const ty = (y: number) => h - pad - (y - minY) * scale

  const type = props.geometry.type as string
  ctx.strokeStyle = '#58a6ff'
  ctx.fillStyle = 'rgba(88, 166, 255, 0.12)'
  ctx.lineWidth = 1.5

  if (type === 'Point') {
    const c = props.geometry.coordinates as [number, number]
    ctx.beginPath(); ctx.arc(tx(c[0]), ty(c[1]), 4, 0, Math.PI * 2)
    ctx.fillStyle = '#58a6ff'; ctx.fill()
  } else if (type === 'Polygon') {
    const rings = props.geometry.coordinates as number[][][]
    for (const ring of rings) drawRing(ctx, ring, tx, ty)
  } else if (type === 'MultiPolygon') {
    const polys = props.geometry.coordinates as number[][][][]
    for (const rings of polys)
      for (const ring of rings) drawRing(ctx, ring, tx, ty)
  } else if (type === 'LineString') {
    const coords = props.geometry.coordinates as number[][]
    ctx.beginPath(); ctx.moveTo(tx(coords[0][0]), ty(coords[0][1]))
    for (let i = 1; i < coords.length; i++) ctx.lineTo(tx(coords[i][0]), ty(coords[i][1]))
    ctx.stroke()
  }
}

function drawRing(ctx: CanvasRenderingContext2D, ring: number[][], tx: (x: number) => number, ty: (y: number) => number) {
  ctx.beginPath()
  ctx.moveTo(tx(ring[0][0]), ty(ring[0][1]))
  for (let i = 1; i < ring.length; i++) ctx.lineTo(tx(ring[i][0]), ty(ring[i][1]))
  ctx.closePath(); ctx.fill(); ctx.stroke()
}

watch(() => props.geometry, render)
onMounted(render)
</script>

<template>
  <canvas ref="canvasRef" :width="width ?? 800" :height="height ?? 600" class="geo-canvas" />
</template>

<style scoped>
.geo-canvas { display: block; border: 1px solid #30363d; border-radius: 4px; background: #0d1117; }
</style>
