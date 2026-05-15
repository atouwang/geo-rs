# Architecture Deviation Analysis

## Severity & Difficulty Legend

| Severity | Meaning |
|---|---|
| **Critical** | Undermines core value proposition |
| **High** | Significant gap in promised capability |
| **Medium** | Missing feature, but core still works |
| **Low** | Minor gap, polish-level |

| Difficulty | Meaning |
|---|---|
| Easy | 1-2 hours |
| Medium | 1-2 days |
| Hard | 3-5 days |
| Very Hard | 1-2 weeks |

---

## A. Structural Deviations

### A1. Transport: JSON instead of FlatBuffers — RESOLVED (MessagePack)

**Design**: JS GeoJSON → FlatBuffers binary → WASM. Zero-copy via SharedArrayBuffer.

**Resolution**: Implemented MessagePack binary transport (May 2026). FlatBuffers replaced by `rmp-serde` (Rust) + `@msgpack/msgpack` (JS). Same binary format goals — no schema compiler required, pure Rust+JS. The `geometry.fbs` schema is retained for future FlatBuffers migration if benchmark data justifies it.

**Implementation details**:
- `geo-core/src/convert.rs`: `from_msgpack(&[u8])` / `to_msgpack() -> Vec<u8>`
- `geo-wasm/src/dispatcher.rs`: `load_bytes(&[u8])` / `read_bytes() -> Vec<u8>`
- `geo-wasm/src/lib.rs`: wasm-bindgen `load(&[u8])` / `read() -> Vec<u8>` (Uint8Array on JS)
- TS Worker: Uint8Array result transferred via `postMessage({...}, {transfer})`
- TS WorkerManager: `call()` auto-detects Uint8Array args, transfers their buffers
- TS GeoEngine: `encode()`/`decode()` via `@msgpack/msgpack`

---

### A2. SharedArrayBuffer not used — RESOLVED (ArrayBuffer transfer)

**Design**: Worker writes directly into SharedArrayBuffer for concurrent read.

**Resolution**: ArrayBuffer transfer via `postMessage({...}, {transfer: [buffer]})` achieves zero-copy between threads. Ownership transfers — main thread can't access the buffer after sending to Worker (and vice versa). This is sufficient for our one-directional data flow (import → compute → export). SAB would only benefit concurrent read scenarios, which we don't have. No COOP/COEP headers required.

---

### A3. WASM monolithic instead of 7 chunks — MEDIUM, Very Hard

**Design**: 7 separate `.wasm` files. Core 50KB + per-operation modules. First load with only `buffer` = 130KB.

**Actual**: Single 457KB `.wasm`. All code loads upfront.

**What it breaks**: First-load is 3.5x larger than promised (457KB vs 130KB). Gzipped (~120KB) is acceptable but not the "50KB minimum" claimed. For comparison, Turf.js full bundle is ~200KB min+gzip.

**To implement**: Split geo-wasm into separate crates per function group. Each produces its own `.wasm`. They must share the same `MemoryArena` across modules — this is the hard part. Requires WASM Component Model or shared memory table, both still experimental in wasm-bindgen.

**Realistic assessment**: Skip for v1. 457KB/120KB gzipped is competitive. Revisit when WASM component model stabilizes.

**Effort**: 1-2 weeks (high risk)

---

### A4. comlink replaced by manual RPC — LOW, Easy (decided)

**Impact**: Minimal. Manual `{id, method, args}` protocol works correctly, adds zero dependencies, and is trivially debuggable. ~30 lines of boilerplate saved vs comlink. **Recommendation**: keep manual. Not a real gap.

---

## B. Missing Components

### B1. memory-manager.ts — MEDIUM, Easy

**Design**: `MemoryManager` class tracks all active handles, validates operations, provides client-side stats.

**Actual**: Handles are raw `bigint`. No validation. Wrong handle → string error from WASM.

**To implement**: Create `memory-manager.ts`, integrate into `GeoEngine`, validate handles before Worker call, add `getActiveHandles()`.

**Effort**: 2-3 hours

---

### B2. Turf.js fallback — REMOVED (decided)

Explicit decision. WASM 97%+ coverage. Fallback added complexity for no real users. Done.

---

### B3. `examples/` directory empty — LOW, Medium

**Design**: vanilla-js, vue3, react, leaflet-integration examples.

**Actual**: Directory doesn't exist.

**To implement**: Create 4 minimal projects showing geo-rs usage in each framework.

**Effort**: 2-3 days

---

### B4. OpenLayers not in Playground — MEDIUM, Medium

**Design**: Playground uses OpenLayers for interactive map. Monaco for code editor.

**Actual**: Playground is JSON textarea with JS fallback area calculator. No WASM actually loaded.

**To implement**: Integrate OpenLayers map component. Draw input geometry on map. Show results visually. Bundle WASM into Vite build.

**Effort**: 1-2 days

---

## C. Quality Gaps

### C1. Worker error recovery — LOW, Easy

Design says exponential backoff on Worker crash. Not implemented.

**Effort**: 1-2 hours

### C2. Reference counting in Arena — LOW, Medium

Design says shared sub-geometry dedup. Not implemented.

**Effort**: 1 day. Recommend skip for v1.

### C3. OffscreenCanvas / SharedWorker — LOW, Medium

Design mentions both. Neither implemented.

**Effort**: 2-3 days each. Recommend v2.

---

## D. Testing Gaps

| # | Gap | Severity | Difficulty |
|---|---|---|---|
| D1 | Property-based tests (proptest) | Medium | Medium |
| D2 | Snapshot tests (insta) | Low | Easy |
| D3 | wasm-bindgen-test in browser | Medium | Medium |
| D4 | TypeScript vitest tests | Medium | Medium |
| D5 | CI benchmark regression detection | Low | Easy |

**Total testing effort**: 5-6 days for all

---

## Summary (Updated May 2026)

### Resolved

| # | Gap | Resolution |
|---|---|---|
| A1 | FlatBuffers transport | MessagePack + ArrayBuffer transfer implemented |
| A2 | SharedArrayBuffer | ArrayBuffer transfer achieves zero-copy; SAB not needed |
| B1 | memory-manager.ts | Implemented + integrated into GeoEngine |
| C1 | Worker error recovery | Exponential backoff reconnect implemented |

### Remaining (before v1.0)

| # | Gap | Effort |
|---|---|---|
| E | WASM loaded in Playground | 1d |
| D1 | Property tests for geo-algo | 2-3d |
| D4 | TypeScript vitest tests | 1-2d |

**Subtotal: 4-6 days**

### Defer to v1.1+

| # | Gap |
|---|---|
| A3 | WASM module splitting |
| B3 | Framework examples |
| C2 | Reference counting |
| C3 | OffscreenCanvas / SharedWorker |
| D2 | Snapshot tests |
| D3 | wasm-bindgen-test in browser |
| D5 | CI benchmark regression |

### Explicitly Skipped

| # | Gap | Reason |
|---|---|---|
| A4 | comlink | Manual RPC works fine |
| B2 | Turf.js fallback | WASM 97%+ coverage |

**Resolved: 4 gaps. Remaining: 3 gaps (4-6 days).**
