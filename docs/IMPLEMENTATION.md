# Implementation Roadmap — geo-rs

## Phase 0: Project Scaffolding (1-2 days)

### Task 0.1 — Rust workspace setup
- [ ] Create root `Cargo.toml` with workspace members
- [ ] Create `crates/geo-core/Cargo.toml` with `geo-types`, `geojson`, `serde`, `serde_json`
- [ ] Create all 7 crate skeletons with correct dependency edges
- [ ] Verify `cargo build` passes for entire workspace

### Task 0.2 — TypeScript monorepo setup
- [ ] Create root `package.json` with workspaces
- [ ] Create `packages/core/package.json` → `@geo-rs/core`
- [ ] Create `packages/vue/package.json` → `@geo-rs/vue`
- [ ] Create `packages/site/package.json` → playground
- [ ] Set up shared `tsconfig.json`, `eslint.config.mjs`, `prettier.config.mjs`
- [ ] Verify `pnpm install` passes, `pnpm typecheck` passes

### Task 0.3 — CI/CD pipeline
- [ ] GitHub Actions: Rust `cargo test`, `cargo clippy`, `cargo fmt --check`
- [ ] GitHub Actions: TypeScript `pnpm typecheck`, `pnpm lint`, `pnpm test`
- [ ] GitHub Actions: `cargo build --target wasm32-unknown-unknown` for geo-wasm
- [ ] Branch protection rules for `main`

### Task 0.4 — Git repo & docs
- [ ] Add `.gitignore` (Rust `target/`, `node_modules/`, `dist/`, `*.wasm`)
- [ ] Add `LICENSE` (MIT)
- [ ] Add `CONTRIBUTING.md`
- [ ] Initial commit

---

## Phase 1: Rust Core — Types & Measurement (3-5 days)

### Task 1.1 — `geo-core` types
- [ ] `types.rs`: `Point`, `LineString`, `Polygon`, `Multi*`, `Geometry` enum
- [ ] `coords.rs`: `CoordSystem` enum, WGS84 ↔ WebMercator transform
- [ ] `measure.rs`: `area()`, `length()`, `distance()`, `centroid()`, `bbox()`
- [ ] `convert.rs`: `Geometry::from_geojson()`, `Geometry::to_geojson()`
- [ ] `error.rs`: `GeoError` enum with `Display` + `std::error::Error`
- [ ] Unit tests for each module

### Task 1.2 — `geo-algo` operations
- [ ] `buffer.rs`: delegate to `geo::algorithm::buffer::Buffer`
- [ ] `simplify.rs`: delegate to `geo::algorithm::simplify::Simplify`
- [ ] `centroid.rs`: delegate to `geo::algorithm::centroid::Centroid`
- [ ] `convex_hull.rs`: delegate to `geo::algorithm::convex_hull::ConvexHull`
- [ ] `line_ops.rs`: `along()`, `bearing()`, `destination()`, `nearest_point_on_line()`
- [ ] Unit + property tests

### Task 1.3 — `geo-bool` predicates
- [ ] `contains()`, `crosses()`, `disjoint()`, `intersects()`
- [ ] `overlaps()`, `within()`, `touches()`, `equal()`
- [ ] DE-9IM reference case tests

### Task 1.4 — `geo-set` operations
- [ ] `union()`, `intersect()`, `difference()`, `xor()`
- [ ] `dissolve()` — multi-polygon merge
- [ ] Fixture-based tests

---

## Phase 2: Memory Arena & WASM Bridge (5-7 days)

### Task 2.1 — FlatBuffers schema
- [ ] Write `geometry.fbs` schema
- [ ] Generate Rust + TypeScript bindings
- [ ] Round-trip tests (Rust → FBS → Rust, JS → FBS → Rust)

### Task 2.2 — Memory arena
- [ ] `MemoryArena`: slot-based storage, `store/get/remove/clear`
- [ ] Handle-based API (`u64` handles)
- [ ] Memory budget tracking + enforcement

### Task 2.3 — Operation dispatcher
- [ ] `WasmEngine::new()` — init arena, register op handlers
- [ ] `load()` / `execute()` / `read()` / `free()` methods
- [ ] Op code routing table

### Task 2.4 — wasm-bindgen integration
- [ ] `#[wasm_bindgen]` public API surface
- [ ] `wasm-pack build --target web` configuration
- [ ] End-to-end: JS → WASM → GeoJSON → area → result

---

## Phase 3: TypeScript SDK (5-7 days)

### Task 3.1 — Worker infrastructure
- [ ] `engine.worker.ts` — Worker entry, WASM loading, RPC interface
- [ ] `worker-manager.ts` — lifecycle, request/response matching, error recovery
- [ ] WASM detection + graceful degradation

### Task 3.2 — Level 1 API (stateless)
- [ ] `buffer()`, `area()`, `length()`, `centroid()`, `bbox()`
- [ ] `contains()`, `intersects()`, `crosses()`
- [ ] `union()`, `intersect()`, `difference()`
- [ ] Vitest tests with mocked Worker

### Task 3.3 — Level 2 GeoEngine (stateful)
- [ ] `GeoEngine.init()`, `load()`, `read()`, `free()`, `destroy()`
- [ ] Chained operations on handles
- [ ] `memory-manager.ts` — handle tracking

### Task 3.4 — Fallback engine
- [ ] Turf.js wrappers for each operation
- [ ] Auto-detection + transparent switch

### Task 3.5 — Package build
- [ ] Vite library mode configuration
- [ ] `.d.ts` generation
- [ ] `package.json` exports map for tree-shaking

---

## Phase 4: Advanced Features (5-7 days)

### Task 4.1 — Spatial indexing
- [ ] `geo-index` crate: `RTree` (rstar), `KDTree` (kdbush)
- [ ] WASM bindings + TypeScript API
- [ ] Benchmarks: indexed vs brute-force

### Task 4.2 — Grid generation
- [ ] `geo-grid` crate: hex/square/point/triangle grids
- [ ] `voronoi()` from Delaunay triangulation
- [ ] `isolines()` / `isobands()` via Marching Squares

### Task 4.3 — Batch operations
- [ ] `points_within_polygon()` — single WASM call
- [ ] `buffer_multi()` — batch buffer
- [ ] `nearest_multi()` — batch nearest neighbor

### Task 4.4 — Coordinate transforms
- [ ] WGS84 ↔ Web Mercator
- [ ] Proj4-style transform support

---

## Phase 5: Benchmark Suite (3-4 days)

### Task 5.1 — Rust criterion benchmarks
- [ ] Per-operation benchmarks with varying input sizes
- [ ] Memory arena throughput benchmarks
- [ ] FlatBuffers vs JSON serialization comparison

### Task 5.2 — JS vs Turf benchmarks
- [ ] benchmark.js framework with real-world datasets
- [ ] Natural Earth 1:10m, 1:50m, 1:110m test data
- [ ] Side-by-side comparison on playground

### Task 5.3 — CI regression detection
- [ ] GitHub Actions benchmark job
- [ ] >5% regression = PR comment warning

---

## Phase 6: Playground & Ecosystem (5-7 days)

### Task 6.1 — Playground website
- [ ] Vue 3 + Vite + OpenLayers setup
- [ ] Interactive API docs with live code editor
- [ ] Benchmark visualization page
- [ ] Draw-on-map playground

### Task 6.2 — `@geo-rs/vue` package
- [ ] `useBuffer`, `useVoronoi`, `useIntersect` composables
- [ ] `GeoCanvas` + `GeoMap` components

### Task 6.3 — Framework examples
- [ ] Vanilla JS, Vue 3, React, Leaflet integration examples

### Task 6.4 — npm publish
- [ ] Package metadata + automated publish via CI
- [ ] First `v0.1.0` release

---

## Phase 7: Future (post-v1)

- GeoArrow zero-copy integration
- WebGPU compute shader for raster ops
- SharedWorker multi-tab support
- Node.js native bindings (napi-rs)
- H3 hexagon indexing
- Streaming API for massive datasets
