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
    passed.push('PASS dedicated worker: repeated imports, deduplication, export and release')

    const result = await centroid(polygon)
    check(result.geometry.coordinates[0] === 2 && result.geometry.coordinates[1] === 2, 'wrong centroid')
    check(await contains(polygon, point), 'point containment failed')
    check((await union(polygon, polygon))?.type === 'Feature', 'union result invalid')
    const cells = await voronoi({ type: 'FeatureCollection', features: [
      [0, 0], [4, 0], [2, 4],
    ].map(coordinates => ({ type: 'Feature', properties: {}, geometry: { type: 'Point', coordinates } })) },
    { minX: 0, minY: 0, maxX: 4, maxY: 4 })
    check(cells.type === 'FeatureCollection' && cells.features.length === 3, 'Voronoi API result invalid')
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
