# geo-rs

A browser geospatial engine with a Rust core, WebAssembly workers, a TypeScript SDK and Vue 3 integrations.

Geometries remain in WASM memory between operations. Import and export use GeoJSON encoded as MessagePack; ArrayBuffers transfer between the main thread and workers.

## Current functionality

- TypeScript: area, length, centroid, bounding boxes, polygon buffers, simplify, contains/intersects/crosses, union/intersection/difference and approximate Voronoi.
- Rust: additional predicates, XOR/dissolve, coordinate transforms, point spatial indexes, hex grids and contour segments.
- Vue: useBuffer, useVoronoi and GeoCanvas.
- Playground: real SDK operations in a dedicated worker, with results and memory statistics.
- SharedWorker: compiled module shared between clients; each client has its own arena and memory budget.

This is a development repository. The API is not a Turf.js drop-in replacement. Measurements and polygon buffers are planar; buffer distance uses input coordinate units and returns a MultiPolygon. Vue rejects explicit metric units until conversion is supported. Metric/geodesic buffering, exact Voronoi and end-to-end benchmarks remain follow-up work. See [the current audit](docs/OPTIMIZATION.md) for priorities and [the onboarding guide](docs/ONBOARDING.md) for code entry points.

GeoCanvas accepts geometries, Features and FeatureCollections, including polygon holes. Import @geo-rs/vue/style.css for its default appearance. Composables keep the latest request result/error and remain loading until all pending requests finish.

Rust hex_grid and voronoi now return Result. Hex side lengths use Web Mercator projected units and grids are capped at 100,000 cells. Sampled Voronoi stays inside a finite ordered bbox, accepts at most 10,000 points and may omit small/duplicate/unsampled regions. It does not create an exact gap-free partition.

## Local setup

Requires Rust stable, Node.js 22+ and pnpm 10.32.1.

```sh
rustup target add wasm32-unknown-unknown
# Install wasm-pack if it is not already available.
cargo install wasm-pack --locked
wasm-pack build --dev --target web crates/geo-wasm
pnpm install --frozen-lockfile
pnpm build
pnpm dev
```

Build WASM before installing the workspace: the site depends on its generated package.
Use a release WASM build for production; see [CONTRIBUTING.md](CONTRIBUTING.md).

## SDK example

```typescript
import { GeoEngine } from '@geo-rs/core'

const engine = await GeoEngine.init()
try {
  const input = await engine.load({
    type: 'Polygon',
    coordinates: [[[0, 0], [1, 0], [1, 1], [0, 1], [0, 0]]],
  })
  const center = await engine.centroid(input)
  console.log(await engine.read(center)) // GeoJSON Feature
  engine.free(input, center)
} finally {
  engine.destroy()
}
```

Every load or geometry-producing operation owns a reference, including identical handles returned by deduplication. Free each reference once. Handles belong to their engine; clear/destroy invalidates them. A worker crash rejects outstanding calls and requires explicit reinitialization because geometry state has been lost.

## Validation

```sh
cargo test --workspace
cargo clippy --workspace --all-targets -- -D warnings
cargo fmt --all -- --check
pnpm test
pnpm lint
pnpm typecheck
pnpm build
```

Tests include JavaScript-to-real-WASM transport, handle lifetimes, worker failures, Vue concurrency/rendering and benchmark-gate validation. CI also runs both packaged browser suites twice in Chromium. Locally, install Chromium with pnpm exec playwright install chromium, then run pnpm test:browser. For interactive checks, run the site and open /tests/browser-smoke.html and /tests/browser-vue.html; see [CONTRIBUTING.md](CONTRIBUTING.md) for preview commands and failure reports.

Native performance CI compares 20 workloads, including arena storage, deduplication and full/sparse statistics, with a baseline commit on the same runner for push/PR/manual runs. Confirmed regressions fail the workflow, and measurements/logs are uploaded. Run pnpm bench -- HEAD^ locally; see [PERFORMANCE.md](docs/PERFORMANCE.md) for thresholds, reproduction and measurement boundaries.

## License

MIT
