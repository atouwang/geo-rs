import { beforeEach, describe, expect, it, vi } from 'vitest'
import { effectScope } from 'vue'
import { useBuffer } from '../useBuffer'
import { useVoronoi } from '../useVoronoi'
const api = vi.hoisted(() => ({ buffer: vi.fn(), voronoi: vi.fn() }))
vi.mock('@geo-rs/core', () => ({ ...api, computeBBox: () => ({ minX:0, minY:0, maxX:1, maxY:1 }) }))
function deferred() {
  let resolve!: (value: unknown) => void
  let reject!: (reason: Error) => void
  const promise = new Promise((res, rej) => { resolve = res; reject = rej })
  return { promise, resolve, reject }
}
const polygon = { type: 'Feature' as const, properties: {}, geometry: { type: 'Polygon' as const, coordinates: [] } }
const points = { type: 'FeatureCollection' as const, features: [] }
it('reports unsupported metric units instead of silently ignoring them', async () => {
  api.buffer.mockClear()
  const state = useBuffer()
  await state.execute(polygon, { radius: 1, units: 'meters' })
  expect(state.error.value?.message).toContain('not supported')
  expect(api.buffer).not.toHaveBeenCalled()
})
describe.each(['buffer', 'voronoi'] as const)('%s concurrent execution', kind => {
  beforeEach(() => vi.resetAllMocks())
  function create() {
    if (kind === 'buffer') { const state = useBuffer(); return { ...state, run: () => state.execute(polygon, { radius: 1 }) } }
    const state = useVoronoi(); return { ...state, run: () => state.execute(points) }
  }
  it('keeps the latest result and stays loading until all executions settle', async () => {
    const old = deferred(), latest = deferred()
    api[kind].mockReturnValueOnce(old.promise).mockReturnValueOnce(latest.promise)
    const state = create(), first = state.run(), second = state.run()
    latest.resolve({ tag: 'latest' }); await second
    expect(state.loading.value).toBe(true)
    old.resolve({ tag: 'old' }); await first
    expect(state.result.value).toEqual({ tag: 'latest' })
    expect(state.loading.value).toBe(false)
  })
  it('ignores stale errors after a newer success', async () => {
    const old = deferred(), latest = deferred()
    api[kind].mockReturnValueOnce(old.promise).mockReturnValueOnce(latest.promise)
    const state = create(), first = state.run(), second = state.run()
    latest.resolve({ tag: 'latest' }); await second
    old.reject(new Error('stale')); await first
    expect(state.error.value).toBeNull()
  })
  it('ignores completion after the owning Vue scope is disposed', async () => {
    const pending = deferred(); api[kind].mockReturnValue(pending.promise)
    const scope = effectScope(); const state = scope.run(create)!
    const work = state.run(); scope.stop(); pending.resolve({ tag: 'late' }); await work
    expect(state.result.value).toBeNull()
    expect(state.loading.value).toBe(false)
    await state.run(); expect(api[kind]).toHaveBeenCalledTimes(1)
  })
})
