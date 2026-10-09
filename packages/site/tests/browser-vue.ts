import { createApp, effectScope, h, nextTick, reactive } from 'vue'
import { GeoCanvas, useBuffer, useVoronoi } from '@geo-rs/vue'
import '@geo-rs/vue/style.css'
import { computeBBox } from '@geo-rs/core'

const donut = { type: 'Feature' as const, properties: {}, geometry: { type: 'Polygon' as const, coordinates: [
  [[0,0],[10,0],[10,10],[0,10],[0,0]], [[3,3],[7,3],[7,7],[3,7],[3,3]],
] } }
const state = reactive<{ width: number; geometry: typeof donut | null }>({ width: 800, geometry: structuredClone(donut) })
createApp({ render: () => h(GeoCanvas, { ...state, height: 400 }) }).mount('#demo')
const button = document.querySelector<HTMLButtonElement>('#run')!
const output = document.querySelector<HTMLPreElement>('#results')!
const canvas = () => document.querySelector('canvas')!
const alpha = (x: number, y: number) => canvas().getContext('2d')!.getImageData(x, y, 1, 1).data[3]
function paintCount() {
  const data = canvas().getContext('2d')!.getImageData(0, 0, canvas().width, canvas().height).data
  let count = 0
  for (let i = 3; i < data.length; i += 4) if (data[i]) count++
  return count
}
function check(value: unknown, message: string): asserts value { if (!value) throw new Error(message) }
button.onclick = async () => {
  button.disabled = true
  const passed: string[] = []
  try {
    state.width = 800; state.geometry = structuredClone(donut); await nextTick()
    check(alpha(400, 200) === 0 && alpha(300, 100) > 0, 'polygon hole filled or Feature missing')
    passed.push('PASS Feature rendering and transparent polygon hole (actual canvas pixels)')
    state.width = 1000; await nextTick()
    check(canvas().width === 1000 && paintCount() > 0, 'resize cleared canvas without redraw')
    passed.push('PASS resize redraw')
    const before = paintCount(); state.geometry.geometry.coordinates[0][1][0] = 12; await nextTick()
    check(paintCount() !== before, 'deep geometry update did not redraw')
    state.geometry = null; await nextTick(); check(paintCount() === 0, 'null did not clear canvas')
    passed.push('PASS nested coordinate updates and null clearing')
    const buffer = useBuffer()
    await Promise.all([buffer.execute(donut, { radius: 0.1 }), buffer.execute(donut, { radius: 0.2 })])
    check(buffer.result.value?.geometry.type === 'MultiPolygon' && !buffer.error.value, 'incorrect buffer type')
    check(computeBBox(buffer.result.value).minX < -0.15 && !buffer.loading.value, 'latest buffer result lost')
    passed.push('PASS concurrent real-WASM buffer, latest result and MultiPolygon contract')
    const scope = effectScope(); const disposed = scope.run(useBuffer)!
    const pending = disposed.execute(donut, { radius: 0.1 }); scope.stop(); await pending
    check(disposed.result.value === null && !disposed.loading.value, 'disposed scope mutated')
    passed.push('PASS scope disposal during computation')
    const voronoi = useVoronoi()
    for (const coordinates of [[[0,0],[1,0],[2,0]], [[1,1],[1,1],[1,1]]]) {
      await voronoi.execute({ type: 'FeatureCollection', features: coordinates.map(coordinates => ({
        type: 'Feature', properties: {}, geometry: { type: 'Point', coordinates },
      })) })
      check(!voronoi.error.value && (voronoi.result.value?.features.length ?? 0) > 0, 'degenerate auto-bbox rejected valid sites')
    }
    passed.push('PASS collinear/coincident Voronoi sites with automatically padded bounds')
    state.geometry = structuredClone(donut); await nextTick()
    output.textContent = passed.join('\n') + '\nALL PASSED'
  } catch (error) { output.textContent = passed.join('\n') + '\nFAIL ' + String(error) }
  finally { button.disabled = false }
}
