import { GeoEngine, centroid, contains, union, voronoi, type GeoJSON } from '@geo-rs/core'

const button = document.querySelector<HTMLButtonElement>('#run')!
const output = document.querySelector<HTMLPreElement>('#results')!
const point: GeoJSON = { type: 'Point', coordinates: [1, 2] }
const polygon: GeoJSON = { type: 'Polygon', coordinates: [[[0, 0], [4, 0], [4, 4], [0, 4], [0, 0]]] }
function check(condition: unknown, message: string): asserts condition {
  if (!condition) throw new Error(message)
}

button.onclick = async () => {
  button.disabled = true
  output.textContent = 'Running'
  const passed: string[] = []
  const engines: GeoEngine[] = []
  try {
    const engine = await GeoEngine.init()
    engines.push(engine)
    for (let i = 0; i < 10; i++) {
      const h = await engine.load(point)
      const duplicate = await engine.load(point)
      check(h === duplicate, 'deduplication failed')
      engine.free(h)
      check((await engine.read(duplicate)).type === 'Feature', 'remaining reference invalid')
      engine.free(duplicate)
    }
    check((await engine.stats()).active === 0, 'dedicated arena leaked')
    for (const geometry of [
      { type: 'MultiPoint', coordinates: [[0,0],[1,1]] },
      { type: 'LineString', coordinates: [[0,0],[1,1]] },
      { type: 'MultiLineString', coordinates: [[[0,0],[1,1]]] },
      polygon,
      { type: 'MultiPolygon', coordinates: [polygon.coordinates] },
      { type: 'GeometryCollection', geometries: [{ type: 'GeometryCollection', geometries: [point] }] },
    ] as GeoJSON[]) {
      const first = await engine.load(geometry)
      const duplicate = await engine.load(geometry)
      check(first === duplicate, 'geometry variant deduplication failed')
      engine.free(first)
      check((await engine.read(duplicate)).type === 'Feature', 'geometry variant reference invalid')
      engine.free(duplicate)
    }
    check((await engine.stats()).active === 0, 'geometry variant arena leaked')
    const retained = await Promise.all(Array.from({ length: 64 }, (_, i) =>
      engine.load({ type: 'Point', coordinates: [i, 0] })))
    check((await engine.stats()).active === 64, 'bulk stats active count invalid')
    engine.free(...retained.slice(1))
    check((await engine.stats()).active === 1 && (await engine.stats()).allocated === 32,
      'sparse stats invalid after bulk release')
    engine.freeAll()
    check((await engine.stats()).active === 0 && (await engine.stats()).allocated === 0,
      'clear stats invalid')
    const fresh = await engine.load(point)
    check((await engine.stats()).active === 1, 'stats invalid after clear/reuse')
    engine.free(fresh)
    check((await engine.stats()).active === 0, 'stats invalid after final release')
    passed.push('PASS dedicated worker: repeated imports, deduplication, export and release')

    for (const invalid of [
      { type: 'Polygon', coordinates: [[]] },
      { type: 'MultiPolygon', coordinates: [[[[0,0],[1,0],[1,1],[0,0]]], []] },
      { type: 'Point', coordinates: [NaN, 1] },
    ] as GeoJSON[]) {
      let rejected = false
      try { await engine.load(invalid) } catch { rejected = true }
      check(rejected, 'invalid input accepted')
      const valid = await engine.load(polygon)
      check(await engine.area(valid) === 16, 'failed import poisoned the worker')
      engine.free(valid)
    }
    passed.push('PASS invalid input rejection and same-worker recovery')

    const result = await centroid(polygon)
    check(result.geometry.coordinates[0] === 2 && result.geometry.coordinates[1] === 2, 'wrong centroid')
    check(await contains(polygon, point), 'point containment failed')
    check((await union(polygon, polygon))?.type === 'Feature', 'union result invalid')
    const cells = await voronoi({ type: 'FeatureCollection', features: [
      [0, 0], [4, 0], [2, 4],
    ].map(coordinates => ({ type: 'Feature', properties: {}, geometry: { type: 'Point', coordinates } })) },
    { minX: 0, minY: 0, maxX: 4, maxY: 4 })
    check(cells.type === 'FeatureCollection' && cells.features.length === 3, 'Voronoi API result invalid')
    const coordinates = cells.features.flatMap(f => f.geometry.coordinates.flat())
    check(coordinates.every(([x,y]) => x >= 0 && x <= 4 && y >= 0 && y <= 4), 'Voronoi exceeded caller bbox')
    passed.push('PASS stateless APIs: centroid, contains, union and Voronoi')

    const [a, b] = await Promise.all([GeoEngine.init({ shared: true }), GeoEngine.init({ shared: true })])
    engines.push(a, b)
    const [ha, hb] = await Promise.all([a.load(point), b.load(polygon)])
    a.freeAll()
    check((await b.area(hb)) === 16, 'client A cleared client B arena')
    b.free(hb)
    a.destroy()
    engines.splice(engines.indexOf(a), 1)
    const next = await b.load(point)
    check((await b.read(next)).type === 'Feature', 'disconnect broke another shared client')
    b.free(next)
    check((await b.stats()).active === 0, 'shared arena leaked')
    check(ha > 0n, 'invalid shared handle')
    passed.push('PASS shared worker: concurrent clients, isolated freeAll and independent disconnect')

    const limited = await GeoEngine.init({ memoryLimit: 0 })
    engines.push(limited)
    let rejected = false
    try { await limited.load(point) } catch { rejected = true }
    check(rejected, 'zero budget was ignored')
    passed.push('PASS zero memory budget')
    output.textContent = passed.join('\n') + '\nALL PASSED'
  } catch (error) {
    output.textContent = passed.join('\n') + '\nFAIL ' + String(error)
  } finally {
    for (const engine of engines) engine.destroy()
    button.disabled = false
  }
}
