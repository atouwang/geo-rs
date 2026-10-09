# Project instructions

## Architecture

Rust workspace → wasm-bindgen Engine → dedicated/shared worker RPC → TypeScript SDK → Vue adapters and site. Read docs/ONBOARDING.md and docs/OPTIMIZATION.md for current entry points and priorities. ARCHITECTURE.md and IMPLEMENTATION.md retain original proposals, not completed-feature inventories.

Use CodeGraph for structural symbol, dependency and flow queries. Use native text search for literals and focused reads of identified files. Do not delegate architecture exploration that the existing graph can answer directly.

## Setup and verification

Use Rust stable, Node.js 22+ and pnpm 10.32.1. Build the generated WASM package before pnpm installation, and rebuild after Rust edits:

```sh
wasm-pack build --dev --target web crates/geo-wasm
pnpm install --frozen-lockfile
cargo fmt --all -- --check
cargo clippy --workspace --all-targets -- -D warnings
cargo test --workspace
cargo check --workspace --target wasm32-unknown-unknown
pnpm test
pnpm lint
pnpm typecheck
pnpm build
```

For browser changes, run pnpm dev and exercise the affected Playground states and /tests/browser-smoke.html. Check packaged output with a production build and preview as well. Node WASM tests and successful builds do not prove browser worker/asset behavior.

## Runtime contracts

- Wire data is GeoJSON-shaped MessagePack; read exports a GeoJSON Feature.
- Geometry operations retain references even when deduplication returns identical handles. Release every acquired reference once; clean partial failures in finally blocks.
- Raw WASM release(handle) releases geometry; free() destroys the wrapper. SDK free(...handles) keeps its public name.
- SharedWorker clients share the worker/module but own separate arenas and budgets. A crash loses geometry state and requires explicit reinitialization.
- Measurements and polygon buffers currently use planar input coordinates. Do not claim meter/geodesic conversion, exact Voronoi or current browser speedups without implementation and evidence.

Use conventional, logically scoped commits. Preserve user changes and record tested behavior and material verification gaps. Generated pkg/, dist/ and target/ directories stay untracked.
