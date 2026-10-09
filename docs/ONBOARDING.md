# geo-rs onboarding

This monorepo keeps geometry in Rust/WASM memory and exposes asynchronous browser APIs. The demo calls the same SDK that library consumers use.

| Layer | Entry points | Responsibility |
|---|---|---|
| Geometry core | crates/geo-core/src/{types,convert,measure,coords}.rs | Internal types, GeoJSON/MessagePack conversion, planar measures and projections |
| Algorithms | crates/geo-{algo,bool,set,index,grid}/src | Buffers/simplify, predicates, set operations, point indexes and grid approximations |
| WASM | crates/geo-wasm/src/{lib,dispatcher,arena}.rs | JS exports, opcode dispatch, budgets, unique handles and reference-counted deduplication |
| SDK | packages/core/src/{engine,worker-manager,memory-manager}.ts | Initialization, asynchronous RPC, input/result reference ownership and invalidation |
| Worker | packages/core/src/worker/{runtime,engine.worker,engine.shared.worker}.ts | Load WASM once, create client arenas, route/transfer messages and dispose engines |
| Stateless APIs | packages/core/src/api | Share an SDK engine; release operation-owned handles in finally blocks |
| Vue | packages/vue/src/composables and components | Reactive operation wrappers and canvas drawing |
| Demo | packages/site/src/views/Playground.vue | Run SDK operations and display output/error/memory statistics |

## One operation

GeoEngine.load encodes GeoJSON as MessagePack. WorkerManager transfers the bytes and matches the response by request ID. The worker invokes Engine.load; Rust converts GeoJSON to internal Geometry and stores it in MemoryArena. Later operations pass bigint handles, not geometry payloads. Geometry results retain a reference in both the arena and SDK tracker. Read returns a GeoJSON Feature encoded as MessagePack. Free releases one reference. Bounding boxes are computed in WASM and return only four bounds.

Dedicated workers own one arena. Shared workers share a compiled module and worker thread, with an independent Engine per port. Destroying one client does not clear another client's geometry. Explicit destroy sends disposal before closing the port. Worker crashes are terminal for an engine; callers must initialize a new engine and reload data.

## Conventions and checks

Rust errors use GeoError/Result and become rejected SDK promises. Existing Rust unit tests live beside the implementation; SDK tests use Vitest in packages/core/src/__tests__. Raw WASM release(handle) frees geometry, while free() destroys the generated wasm-bindgen Engine wrapper. The SDK keeps its public free(...handles) method.

See CONTRIBUTING.md for fresh setup and the full gates. Always rebuild WASM after Rust edits. Validate the actual browser worker chain in addition to unit tests and builds.

The architecture and implementation roadmap describe original targets; they are not evidence that every proposed feature exists. Read OPTIMIZATION.md for current limitations before extending algorithm semantics or making performance claims.
