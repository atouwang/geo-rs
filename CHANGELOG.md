# Changelog

## [Unreleased]

### Added
- MessagePack binary transport (replaces JSON string transfer)
- ArrayBuffer zero-copy transfer between main thread and Worker
- Geometry dedup with reference counting in MemoryArena
- OffscreenCanvas + SharedWorker support in WorkerManager
- dissolve() set operation
- voronoi() and isolines() in WASM engine
- CI benchmark regression detection (>10% threshold)
- Property-based tests for geo-algo and geo-bool
- TypeScript vitest tests (MemoryManager, computeBBox)
- GitHub Actions CI (Rust test/clippy/fmt + WASM + TS + Bench)

### Fixed
- centroid/bbox/crosses API returning placeholder data
- KDTree using brute-force instead of kdbush crate
- Vue composables (useBuffer, useVoronoi) as dead code
- GeoCanvas empty component (now has Canvas 2D rendering)
- Worker crash without auto-reconnect (now exponential backoff)

### Changed
- FlatBuffers schema retained but transport uses MessagePack
- SharedArrayBuffer deferred; ArrayBuffer transfer is sufficient
- Turf.js fallback removed (WASM coverage 97%+)
- comlink dependency removed (manual RPC is simpler)
- Playground now loads real WASM engine (not JS fallback)

## [0.1.0] - 2026-05-15

### Initial Release
- 7 Rust crates: geo-core, geo-algo, geo-bool, geo-set, geo-index, geo-grid, geo-wasm
- TypeScript SDK: @geo-rs/core, @geo-rs/vue
- Vue 3 Playground: Home, Benchmarks, Playground pages
- 20+ spatial operations: buffer, simplify, contains, intersects, union, intersect, dissolve, centroid, bbox, area, length, bearing, destination, convex_hull, hex_grid, voronoi, isolines
- Web Worker + WASM engine with handle-based batch API
- 8 criterion benchmarks (4x-150x vs Turf.js baseline)
