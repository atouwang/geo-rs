# Architecture Design — geo-rs

## 1. Project Vision

Build the browser-native geospatial analysis library that Turf.js users have been waiting for — **Rust-powered speed with TypeScript ergonomics**. The key insight: previous Rust+WASM attempts died because they serialized geometery data on every operation call. We serialize once at import, once at export, and keep everything else in WASM memory.

### 1.1 Non-Goals (for v1)

- Server-side Node.js deployment (browser-first)
- 3D / terrain analysis
- Raster processing (vector-first)
- Real-time collaboration features

---

## 2. Architecture Overview

```
┌──────────────────────────────────────────────────────────────┐
│                     Playground (Vue 3)                        │
│   Interactive docs · Benchmark runner · Example gallery       │
├──────────────────────────────────────────────────────────────┤
│                     @geo-rs/vue                                │
│   Composables (useBuffer, useVoronoi…) · GeoCanvas component  │
├──────────────────────────────────────────────────────────────┤
│                     @geo-rs/core (TypeScript SDK)              │
│   ┌─────────────┐  ┌──────────────┐  ┌───────────────────┐   │
│   │ WorkerManager│  │ MemoryManager │  │ Fallback Engine   │   │
│   │ (Web Worker  │  │ (handle track │  │ (Turf.js degrade  │   │
│   │  lifecycle)  │  │  + SAB mgmt)  │  │  when WASM fails) │   │
│   └─────────────┘  └──────────────┘  └───────────────────┘   │
│   ┌──────────────────────────────────────────────────────┐    │
│   │              High-Level API Layer                     │    │
│   │  buffer() / intersect() / union() / contains() …     │    │
│   │  GeoEngine class (batch operations with handles)     │    │
│   └──────────────────────────────────────────────────────┘    │
├──────────────────────────────────────────────────────────────┤
│                   WASM Bridge (wasm-bindgen)                   │
│   ┌─────────────┐  ┌──────────────┐  ┌───────────────────┐   │
│   │FlatBuffers   │  │Operation      │  │Memory Arena       │   │
│   │Serializer    │  │Dispatcher     │  │(WASM heap mgmt)   │   │
│   │(zero-copy)   │  │(op routing)   │  │                   │   │
│   └─────────────┘  └──────────────┘  └───────────────────┘   │
├──────────────────────────────────────────────────────────────┤
│                    Rust Core Engine                            │
│   ┌──────────┐ ┌──────────┐ ┌──────────┐ ┌──────────┐       │
│   │ geo-core │ │ geo-algo │ │ geo-bool │ │ geo-set  │       │
│   │ types    │ │ buffer   │ │ contains │ │ union    │       │
│   │ coords   │ │ simplify │ │ crosses  │ │ intersect│       │
│   │ measure  │ │ centroid │ │ overlaps │ │ diff     │       │
│   └──────────┘ └──────────┘ └──────────┘ └──────────┘       │
│   ┌──────────┐ ┌──────────┐ ┌──────────────────────┐        │
│   │geo-index │ │ geo-grid │ │ geo-wasm             │        │
│   │R*-Tree   │ │ hexGrid  │ │ bindings + dispatch  │        │
│   │K-D Tree  │ │ voronoi  │ │ FBS schema           │        │
│   │H3(opt)   │ │ isolines │ │                      │        │
│   └──────────┘ └──────────┘ └──────────────────────┘        │
└──────────────────────────────────────────────────────────────┘
```

---

## 3. Layer Design

### 3.1 Rust Core Engine

#### Crate Dependency Graph

```
geo-core (types, traits, no deps)
  ↑
geo-index (rstar, kdbush) ← depends on geo-core
  ↑
geo-algo (buffer, simplify, centroid…) ← geo + geo-core
geo-bool (contains, crosses…) ← geo + geo-core
geo-set (union, intersect…) ← geo + geo-core
geo-grid (hexGrid, voronoi…) ← geo + geo-core + geo-index
  ↑
geo-wasm (bindings, dispatch, FBS) ← all crates above
```

#### `geo-core` — Fundamental Types

```rust
// Coordinate system abstraction
pub enum CoordSystem { Wgs84, WebMercator, Cartesian }

// Internal geometry representation (non-GeoJSON, optimized for compute)
pub struct Point { pub x: f64, pub y: f64 }
pub struct MultiPoint { pub points: Vec<Point> }
pub struct LineString { pub coords: Vec<Point> }
pub struct MultiLineString { pub lines: Vec<LineString> }
pub struct Polygon { pub exterior: LineString, pub interiors: Vec<LineString> }
pub struct MultiPolygon { pub polygons: Vec<Polygon> }

// Generic geometry enum used in memory arena
pub enum Geometry {
    Point(Point),
    MultiPoint(MultiPoint),
    LineString(LineString),
    MultiLineString(MultiLineString),
    Polygon(Polygon),
    MultiPolygon(MultiPolygon),
    GeometryCollection(Vec<Geometry>),
}

// Conversions
impl Geometry {
    pub fn from_geojson(json: &str) -> Result<Self>
    pub fn to_geojson(&self) -> Result<String>
    pub fn from_flatbuffers(buf: &[u8]) -> Result<Self>  // zero-copy
    pub fn to_flatbuffers(&self) -> Vec<u8>
}

// Bounding box
pub struct BBox { pub min_x: f64, pub min_y: f64, pub max_x: f64, pub max_y: f64 }

// Units enum
pub enum Units { Meters, Kilometers, Miles, Degrees }
```

#### `geo-algo` — Geometric Algorithms

```rust
// Implementations backed by the `geo` crate

pub fn buffer(geom: &Geometry, distance: f64, units: Units) -> Result<Geometry>
pub fn simplify(geom: &Geometry, tolerance: f64) -> Result<Geometry>
pub fn centroid(geom: &Geometry) -> Point
pub fn bbox(geom: &Geometry) -> BBox
pub fn area(geom: &Geometry) -> f64
pub fn length(geom: &Geometry) -> f64
pub fn distance(p1: &Point, p2: &Point, units: Units) -> f64
pub fn bearing(p1: &Point, p2: &Point) -> f64
pub fn destination(origin: &Point, distance: f64, bearing: f64) -> Point
pub fn along(line: &LineString, distance: f64) -> Point
pub fn nearest_point_on_line(line: &LineString, point: &Point) -> Point
pub fn convex_hull(geom: &Geometry) -> Polygon
pub fn concave_hull(points: &[Point], max_edge: f64) -> Polygon
pub fn center_of_mass(geom: &Geometry) -> Point
pub fn line_offset(line: &LineString, distance: f64) -> LineString
```

#### `geo-bool` — Spatial Predicates

```rust
pub fn contains(a: &Geometry, b: &Geometry) -> bool
pub fn crosses(a: &Geometry, b: &Geometry) -> bool
pub fn disjoint(a: &Geometry, b: &Geometry) -> bool
pub fn intersects(a: &Geometry, b: &Geometry) -> bool
pub fn overlaps(a: &Geometry, b: &Geometry) -> bool
pub fn within(a: &Geometry, b: &Geometry) -> bool
pub fn touches(a: &Geometry, b: &Geometry) -> bool
pub fn equal(a: &Geometry, b: &Geometry) -> bool
pub fn point_on_surface(geom: &Geometry) -> Point
```

#### `geo-set` — Set Operations

```rust
pub fn union(a: &Geometry, b: &Geometry) -> Result<Geometry>
pub fn intersect(a: &Geometry, b: &Geometry) -> Result<Geometry>
pub fn difference(a: &Geometry, b: &Geometry) -> Result<Geometry>
pub fn xor(a: &Geometry, b: &Geometry) -> Result<Geometry>
pub fn dissolve(features: &[Geometry]) -> Result<Geometry>
```

#### `geo-index` — Spatial Indexing

```rust
pub struct RTree { /* rstar::RTree */ }
impl RTree {
    pub fn new() -> Self
    pub fn insert(&mut self, geom: Geometry, id: u64)
    pub fn search(&self, bbox: &BBox) -> Vec<u64>
    pub fn nearest(&self, point: &Point, k: usize) -> Vec<(u64, f64)>
    pub fn within(&self, geom: &Geometry) -> Vec<u64>
}

pub struct KDTree { /* kdbush */ }
impl KDTree {
    pub fn new(points: &[Point]) -> Self
    pub fn within(&self, bbox: &BBox) -> Vec<usize>
    pub fn nearest(&self, point: &Point, k: usize) -> Vec<(usize, f64)>
}
```

#### `geo-grid` — Grid & Interpolation

```rust
pub fn hex_grid(bbox: &BBox, cell_side: f64, units: Units) -> Vec<Polygon>
pub fn point_grid(bbox: &BBox, cell_side: f64, units: Units) -> Vec<Point>
pub fn square_grid(bbox: &BBox, cell_side: f64, units: Units) -> Vec<Polygon>
pub fn triangle_grid(bbox: &BBox, cell_side: f64, units: Units) -> Vec<Polygon>
pub fn isolines(points: &[Point], breaks: &[f64]) -> Vec<LineString>
pub fn isobands(points: &[Point], breaks: &[f64]) -> Vec<Polygon>
pub fn voronoi(points: &[Point], bbox: &BBox) -> Vec<Polygon>
pub fn tin(points: &[Point]) -> Vec<Polygon>
pub fn interpolate(points: &[Point], target: &Point) -> f64
```

### 3.2 WASM Bridge

#### Data Format — FlatBuffers Schema (geometry.fbs)

```
namespace geo_rs;

table Point { x: float64; y: float64; }
table LineString { coords: [Point]; }
table Polygon { exterior: LineString; interiors: [LineString]; }

table Geometry {
    type: ubyte;
    point: Point;
    multi_point: [Point];
    line_string: LineString;
    multi_line_string: [LineString];
    polygon: Polygon;
    multi_polygon: [Polygon];
}

table OpRequest {
    op_code: ubyte;
    handle_a: uint64;
    handle_b: uint64;
    params: [float64];
}

table OpResponse {
    result_handle: uint64;
    error: string;
}

root_type OpRequest;
```

#### Operation Dispatcher

```rust
// geo-wasm/src/dispatcher.rs

#[wasm_bindgen]
pub struct WasmEngine {
    arena: MemoryArena,       // geometry storage
    operations: HashMap<u8, Box<dyn Fn(&[u8]) -> Vec<u8>>>,
}

#[wasm_bindgen]
impl WasmEngine {
    pub fn new() -> Self { /* alloc arena, register op handlers */ }

    // Load geometry into WASM memory, return handle
    pub fn load(flatbuffer_bytes: &[u8]) -> u64 {
        let geoms = parse_flatbuffer_geometries(flatbuffer_bytes);
        self.arena.store(geoms)  // returns handle_id
    }

    // Execute an operation by op_code on handles
    pub fn execute(op_flatbuffer: &[u8]) -> u64 {
        let req = parse_op_request(op_flatbuffer);
        let geom_a = self.arena.get(req.handle_a);
        let geom_b = req.handle_b.map(|h| self.arena.get(h));
        let result = self.dispatch(req.op_code, geom_a, geom_b, req.params);
        self.arena.store(result)  // returns new handle
    }

    // Read result geometry back to JS
    pub fn read(handle: u64) -> Vec<u8> {
        let geom = self.arena.get(handle);
        serialize_to_flatbuffers(geom)
    }

    // Free geometry from arena
    pub fn free(handle: u64) { self.arena.remove(handle); }
    pub fn free_all() { self.arena.clear(); }
}
```

#### Operation Codes

```rust
pub const OP_AREA: u8 = 0x01;
pub const OP_LENGTH: u8 = 0x02;
pub const OP_CENTROID: u8 = 0x03;
pub const OP_BBOX: u8 = 0x04;
pub const OP_BUFFER: u8 = 0x10;
pub const OP_SIMPLIFY: u8 = 0x11;
pub const OP_CONVEX_HULL: u8 = 0x12;
pub const OP_CONTAINS: u8 = 0x20;
pub const OP_INTERSECTS: u8 = 0x21;
pub const OP_CROSSES: u8 = 0x22;
pub const OP_UNION: u8 = 0x30;
pub const OP_INTERSECT: u8 = 0x31;
pub const OP_DIFFERENCE: u8 = 0x32;
pub const OP_VORONOI: u8 = 0x40;
pub const OP_HEX_GRID: u8 = 0x41;
pub const OP_ISOLINES: u8 = 0x42;
```

### 3.3 TypeScript SDK (`@geo-rs/core`)

#### Worker Manager

```typescript
// src/worker-manager.ts

class WorkerManager {
    private worker: Worker
    private pending = new Map<number, { resolve, reject }>()
    private seq = 0

    constructor() {
        this.worker = new Worker(new URL('./engine.worker.ts', import.meta.url))
        this.worker.onmessage = this.handleMessage.bind(this)
    }

    async send(opCode: number, params: Float64Array, handles: bigint[]): Promise<bigint> {
        const id = ++this.seq
        return new Promise((resolve, reject) => {
            this.pending.set(id, { resolve, reject })
            const buf = encodeOpRequest(id, opCode, params, handles)
            this.worker.postMessage(buf)
        })
    }

    terminate() { this.worker.terminate() }
}
```

#### Core API

```typescript
// src/api.ts

// Level 1 — Simple (stateless, auto-manages engine lifecycle)
export async function buffer(
    geom: GeoJSON,
    options: { radius: number; units?: Units }
): Promise<Feature<Polygon>>

export async function intersect(
    a: GeoJSON,
    b: GeoJSON
): Promise<Feature<Polygon | MultiPolygon> | null>

export async function union(
    a: GeoJSON,
    b: GeoJSON
): Promise<Feature<Polygon | MultiPolygon> | null>

// ... etc for each operation

// Level 2 — Engine (batch, data stays in WASM)
export class GeoEngine {
    static async init(): Promise<GeoEngine>

    async load(geojson: GeoJSON): Promise<bigint>     // → handle
    async buffer(handle: bigint, radius: number): Promise<bigint>
    async intersect(a: bigint, b: bigint): Promise<bigint>
    async union(a: bigint, b: bigint): Promise<bigint>
    async contains(a: bigint, b: bigint): Promise<boolean>
    async read(handle: bigint): Promise<GeoJSON>        // export from WASM
    free(...handles: bigint[]): void
    freeAll(): void
    destroy(): void                                     // terminate worker
}
```

#### Memory Manager

```typescript
// src/memory-manager.ts

type HandleState = 'active' | 'freed'

class MemoryManager {
    private handles = new Map<bigint, HandleState>()

    register(handle: bigint): void
    isActive(handle: bigint): boolean
    free(handle: bigint): void
    stats(): { active: number; freed: number; totalBytes: number }
}
```

#### Fallback Engine

```typescript
// src/fallback.ts

// Transparent Turf.js fallback when WASM engine is unavailable
// (e.g., older browsers, certain security policies blocking WASM)

import * as turf from '@turf/turf'

export function buffer(geom: GeoJSON, radius: number): Feature<Polygon> {
    return turf.buffer(geom, radius)
}

// Auto-detection & switch in SDK initializer
export async function createEngine(): Promise<Engine> {
    if (await wasmSupported()) return new WasmEngine()
    console.warn('[geo-rs] WASM unavailable, using Turf.js fallback')
    return new TurfFallback()
}
```

### 3.4 Vue 3 Integration (`@geo-rs/vue`)

```typescript
// src/composables/useBuffer.ts
export function useBuffer() {
    const loading = ref(false)
    const error = ref<Error | null>(null)
    const result = shallowRef<Feature<Polygon> | null>(null)

    async function execute(
        geom: MaybeRef<GeoJSON>,
        options: MaybeRef<{ radius: number; units?: Units }>
    ) {
        loading.value = true
        error.value = null
        try {
            result.value = await buffer(unref(geom), unref(options))
        } catch (e) {
            error.value = e as Error
        } finally {
            loading.value = false
        }
    }

    return { execute, result, loading, error }
}

// src/components/GeoCanvas.vue
// Canvas-based renderer for geometry visualization
```

---

## 4. Data Flow

### 4.1 Simple Operation (Level 1 API)

```
User: buffer(polygon, { radius: 500 })

Main Thread (@geo-rs/core):
  1. Convert GeoJSON → FlatBuffers ArrayBuffer
  2. postMessage → Worker

Web Worker (geo-wasm):
  3. Receive ArrayBuffer (zero-copy via SharedArrayBuffer)
  4. Parse FlatBuffers → Rust Geometry
  5. Execute geo::buffer(geometry, 500.0)
  6. Write result → FlatBuffers → SharedArrayBuffer
  7. postMessage({ handle_id }) → Main Thread

Main Thread:
  8. postMessage({ op: 'read', handle_id }) → Worker
  9. Receive FlatBuffers → parse → GeoJSON
  10. Return Feature<Polygon>
```

### 4.2 Batch Operation (Level 2 API)

```
User:
  h1 = await engine.load(largeGeoJSON)
  h2 = await engine.buffer(h1, 500)
  h3 = await engine.intersect(h2, hOther)

Main Thread:
  1. load(): serialize → Worker → WASM stores → return handle_1
  2. buffer(): only send {op: BUFFER, handle: 1, params: [500]}
     → Worker executes entirely in WASM → return handle_2
  3. intersect(): only send {op: INTERSECT, handle_a: 2, handle_b: 3}
     → Worker executes entirely in WASM → return handle_4
  4. read(h4): Worker serializes final result → Main Thread

Key: Steps 2-3 pass NO geometry data, only handles + params.
```

---

## 5. Performance Strategy

### 5.1 Why Previous Attempts Failed

```
Traditional WASM approach:
  JS GeoJSON → JSON.stringify → WASM → JSON.parse → Compute → JSON.stringify → JS
  └───── 5-20ms overhead per call ────────────────────────────────┘

geo-rs approach:
  Import: JS GeoJSON → FlatBuffers (once) → WASM stores → handle
  Ops:    {handle, op, params} → Compute on resident data → new handle  (~0.1ms)
  Export: WASM → FlatBuffers → JS GeoJSON (only when needed)
```

### 5.2 Memory Strategy

- Arena allocator in WASM for geometry storage
- Handle-based API: JS holds opaque `bigint` handles, not data
- Reference counting for shared sub-geometries
- Explicit `free()` API; auto-cleanup on `destroy()`
- Configurable memory budget (default: 256MB)

### 5.3 Bundle Size Strategy

Split WASM into functional chunks, lazy-loaded by operation:

```
geo_core.wasm       ~50KB  (types, arena, FlatBuffers parser)
geo_measure.wasm    ~30KB  (area, length, centroid, bbox)
geo_ops.wasm        ~80KB  (buffer, simplify, convex_hull)
geo_bool.wasm       ~60KB  (contains, crosses, intersects…)
geo_set.wasm        ~90KB  (union, intersect, difference)
geo_grid.wasm       ~40KB  (hexGrid, voronoi, isolines)
geo_index.wasm      ~50KB  (R-Tree, K-D Tree)
```

First operation loads `geo_core.wasm` + the specific operation module.
Using only `buffer` → ~130KB total first load (50 + 80).

### 5.4 Computation in Web Worker

All WASM execution runs in a dedicated Web Worker:
- Never blocks the UI thread
- Supports `OffscreenCanvas` for direct-to-screen rendering
- Can be shared across multiple browser tabs via `SharedWorker`

---

## 6. Repository Structure

```
geo-rs/
├── README.md
├── LICENSE
├── CONTRIBUTING.md
├── .github/
│   └── workflows/
│       └── ci.yml
│
├── crates/                         # Rust workspace
│   ├── geo-core/
│   │   ├── Cargo.toml
│   │   └── src/
│   │       ├── lib.rs
│   │       ├── types.rs            # Geometry, Point, BBox...
│   │       ├── coords.rs           # CoordSystem, transforms
│   │       ├── measure.rs          # area, length, distance
│   │       ├── convert.rs          # GeoJSON ↔ internal ↔ FlatBuffers
│   │       └── error.rs
│   │
│   ├── geo-algo/
│   │   ├── Cargo.toml
│   │   └── src/
│   │       ├── lib.rs
│   │       ├── buffer.rs
│   │       ├── simplify.rs
│   │       ├── centroid.rs
│   │       ├── convex_hull.rs
│   │       └── ...
│   │
│   ├── geo-bool/
│   │   ├── Cargo.toml
│   │   └── src/
│   │       ├── lib.rs
│   │       └── predicates.rs
│   │
│   ├── geo-set/
│   │   ├── Cargo.toml
│   │   └── src/
│   │       ├── lib.rs
│   │       └── set_ops.rs
│   │
│   ├── geo-index/
│   │   ├── Cargo.toml
│   │   └── src/
│   │       ├── lib.rs
│   │       ├── rtree.rs
│   │       └── kdtree.rs
│   │
│   ├── geo-grid/
│   │   ├── Cargo.toml
│   │   └── src/
│   │       ├── lib.rs
│   │       ├── hex_grid.rs
│   │       ├── voronoi.rs
│   │       └── isolines.rs
│   │
│   └── geo-wasm/
│       ├── Cargo.toml
│       └── src/
│           ├── lib.rs              # wasm-bindgen entry
│           ├── dispatcher.rs       # operation routing
│           ├── arena.rs            # memory arena
│           ├── fbs_schema.rs       # FlatBuffers generated code
│           └── fbs/
│               └── geometry.fbs    # FlatBuffers schema
│
├── packages/                       # JavaScript/TypeScript
│   ├── core/                       # @geo-rs/core
│   │   ├── package.json
│   │   ├── tsconfig.json
│   │   ├── vite.config.ts
│   │   └── src/
│   │       ├── index.ts            # public API exports
│   │       ├── engine.ts           # GeoEngine class
│   │       ├── worker-manager.ts   # Web Worker lifecycle
│   │       ├── memory-manager.ts   # Handle tracking
│   │       ├── fallback.ts         # Turf.js fallback
│   │       ├── api/
│   │       │   ├── buffer.ts
│   │       │   ├── intersect.ts
│   │       │   ├── union.ts
│   │       │   └── ...
│   │       ├── worker/
│   │       │   └── engine.worker.ts # Worker entry point
│   │       └── types/
│   │           └── index.ts        # Public type exports
│   │
│   ├── vue/                        # @geo-rs/vue
│   │   ├── package.json
│   │   ├── tsconfig.json
│   │   └── src/
│   │       ├── index.ts
│   │       ├── composables/
│   │       │   ├── useBuffer.ts
│   │       │   ├── useVoronoi.ts
│   │       │   └── ...
│   │       └── components/
│   │           └── GeoCanvas.vue
│   │
│   └── site/                       # Playground
│       ├── package.json
│       ├── vite.config.ts
│       └── src/
│           ├── App.vue
│           ├── router.ts
│           ├── views/
│           │   ├── Home.vue
│           │   ├── Docs.vue
│           │   ├── Benchmarks.vue
│           │   └── Playground.vue
│           └── components/
│
├── benches/
│   ├── rust/                       # criterion benchmarks
│   └── js/                         # benchmark.js vs Turf
│
├── examples/
│   ├── vanilla-js/
│   ├── vue3/
│   ├── react/
│   └── leaflet-integration/
│
└── docs/
    ├── ARCHITECTURE.md
    ├── IMPLEMENTATION.md
    ├── API.md
    └── BENCHMARKS.md
```

---

## 7. Key Dependencies

### Rust

| Crate | Purpose | Version |
|---|---|---|
| `geo` | Core geometry algorithms | 0.28 |
| `geo-types` | Geometry type system | 0.7 |
| `geojson` | GeoJSON serde | 3.1 |
| `rstar` | R*-Tree spatial index | 0.12 |
| `kdbush` | K-D tree for points | 0.2 |
| `flatbuffers` | Zero-copy serialization | 24.x |
| `wasm-bindgen` | JS-WASM interop | 0.2 |
| `serde` + `serde_json` | JSON (for GeoJSON) | 1.x |
| `console_error_panic_hook` | Error reporting | 0.1 |
| `wee_alloc` | Minimal WASM allocator | 0.4 |

### TypeScript

| Package | Purpose |
|---|---|
| `comlink` | RPC-style Worker communication (optional) |
| `flatbuffers` | JS FlatBuffers runtime |
| `@turf/turf` | Fallback engine, benchmark baseline |
| `vitest` | Testing |
| `vite` | Bundler |

### Vue (Playground)

| Package | Purpose |
|---|---|
| `vue` 3.5+ | Framework |
| `vue-router` | Routing |
| `openlayers` | Map rendering |
| `@shikijs/monaco` | Code editor |

---

## 8. Testing Strategy

### Rust Tests
- Unit tests for each crate (cargo test)
- Property-based tests for geometric invariants (proptest)
- Snapshot tests for known geometry results (insta)

### WASM Tests
- wasm-bindgen-test in Node.js and headless browser
- Cross-browser WASM compatibility tests (Chrome, Firefox, Safari)

### TypeScript Tests
- Vitest unit tests for SDK logic
- Mock WASM worker for deterministic SDK testing
- Integration tests with real WASM in headless Chromium

### Benchmark Suite
- Rust criterion benchmarks for each operation
- JS benchmark.js comparing geo-rs vs Turf.js
- Real-world datasets: Natural Earth 1:10m, OSM extracts
- CI-enforced performance regression detection

---

## 9. Error Handling Philosophy

```rust
// geo-core/src/error.rs
pub enum GeoError {
    InvalidGeometry(String),
    TopologyError(String),
    MemoryLimitExceeded { requested: u64, available: u64 },
    OperationNotSupported { op: String, reason: String },
    HandleNotFound(u64),
    SerializationError(String),
}

// Errors propagate through WASM → Worker → SDK as typed errors
// SDK wraps them in user-friendly messages with recovery suggestions
```

In the TypeScript SDK:
- All operations throw typed `GeoError` with `code`, `message`, and `recoveryHint`
- Fallback engine catches WASM errors and retries with Turf.js
- Worker crashes auto-restart with exponential backoff

---

## 10. Security Considerations

- WASM runs in a sandboxed Web Worker — no DOM/network access
- Memory budget enforced in arena allocator (prevents OOM)
- Input validation in Rust before allocation
- No `unsafe` in geometry processing code (only in FlatBuffers bindings, which are generated)
- No eval, no dynamic code loading
- CSP-compatible (no `unsafe-eval` required)
