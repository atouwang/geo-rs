# geo-rs

A high-performance browser-native geospatial analysis engine. Rust core compiled to WebAssembly, TypeScript SDK for developers, with a Vue 3 interactive playground.

## Why

[Turf.js](https://turfjs.org) (~850K weekly downloads) is the de facto geospatial library for the browser, but it's written in pure JavaScript. On real-world datasets it chokes — 1000-point polygon intersection hangs the main thread, union of 1000 circles takes 5+ seconds.

Six Rust+WASM alternatives were attempted. All died at the prototype stage, killed by JSON serialization overhead across the JS-WASM boundary.

**geo-rs** solves this by keeping data resident in WASM memory — load once, operate many times, pay serialization only at import/export.

## Project Status

Early development. See [docs/ARCHITECTURE.md](docs/ARCHITECTURE.md) for the full design.

## Tech Stack

| Layer | Technology |
|---|---|
| Compute Engine | Rust (`geo` crate ecosystem) |
| WASM Bridge | wasm-bindgen + FlatBuffers |
| TypeScript SDK | TypeScript 5.x |
| Framework Integration | Vue 3 composables (`@geo-rs/vue`) |
| Playground | Vue 3 + Vite + OpenLayers |

## Quick Preview (Target API)

```typescript
// Level 1 — Simple
import { buffer, intersect } from '@geo-rs/core'
const zone = await buffer(center, { radius: 500, units: 'meters' })
const overlap = await intersect(zone, buildings)

// Level 2 — Batch (data stays in WASM)
import { GeoEngine } from '@geo-rs/core'
const engine = await GeoEngine.init()
const h1 = await engine.load(cityBoundary)
const h2 = await engine.buffer(h1, 1000)
const result = await engine.read(h2)
engine.free(h1, h2)
```

## License

MIT
