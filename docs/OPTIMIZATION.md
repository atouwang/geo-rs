# Code and functionality audit — 2026-10-09

## Scope and architecture

Reviewed the Rust workspace, WASM dispatcher/arena, dedicated/shared workers, SDK and stateless APIs, Vue adapters, Playground, packaging, CI and benchmark tooling. [ONBOARDING.md](ONBOARDING.md) maps the current architecture. [ARCHITECTURE.md](ARCHITECTURE.md) remains the original proposal.

## Delivered improvements

| Priority | Failure or cost | Change and evidence |
|---|---|---|
| P0 | JS encoded GeoJSON but Rust decoded its internal Geometry representation | Standardize GeoJSON-shaped MessagePack and Feature exports; JS-to-real-WASM tests pass |
| P0 | Arena used handle-1 as slot index even after slot reuse | Explicit handle-to-slot mapping, monotonic IDs across clear, collision-safe dedup buckets; >16,384 release/reuse/dedup cycles covered |
| P0 | SDK omitted operation results and lost duplicate references | Register every geometry result, count references, validate all operation inputs, invalidate in-flight results on clear/destroy |
| P0 | Worker initialization, ports and generated URLs were broken | Correct wasm-bindgen default initialization, memory-budget bigint conversion, MessagePort listeners and packaged relative worker URLs; actual dedicated/shared browser suite passes |
| P1 | Errors/destroy could leave promises pending or silently restart an empty arena | Reject pending and future calls, clean initialization timers/listeners, require explicit reinitialization after state loss |
| P1 | Stateless APIs leaked handles on failures | Release every acquired reference in finally blocks; partial-load, operation and read failures covered |
| P1 | SharedWorker clients could reset or free one another's data | Independent engine/arena per port; concurrent client clear/disconnect tested in browser |
| P1 | Handle release collided with wasm-bindgen wrapper destruction | Raw WASM uses release(handle) and free() separately; SDK retains free(...handles) |
| P1 | Voronoi SDK sent the wrong bbox keys and asserted the wrong output type | Convert bbox fields and GeometryCollection output to actual FeatureCollection polygons; browser API test passes |
| P1 | Feature bboxes returned infinities; bbox exported entire geometry | Traverse Feature/FeatureCollection in computeBBox; reject empty input; engine bbox returns only four WASM-computed bounds |
| P1 | Coordinate transforms silently left MultiPolygon/nested collections unchanged | Complete recursive transforms; projected/inverse bounds regression test |
| P1 | Vue package had no build config; CI missed tests, lint and site build | Add Vue library/SFC declarations, pin pnpm, frozen installs, lint Vue SFCs and SDK, include full test/build gates |
| P2 | Benchmark parser missed multiline output and could pass missing results | Parse median units independently, fail missing/invalid data, enforce the stated 5% threshold; four parser/comparison tests |
| P2 | Playground bypassed SDK and leaked on errors/unmount | Use asynchronous worker SDK, finally cleanup and route disposal; dev and production-preview interaction evidence |
| P2 | Documentation/demo claimed capabilities and timings beyond evidence | Correct the current API example, feature scope and repository URL; label static timing figures as historical/estimated native data |

Arena deduplication now compares stored geometry directly and caches its hash for release, avoiding repeated serialization during equality checks and removal. No unmeasured speedup is claimed.

## Verification

- Rust: 67 unit tests; strict all-target Clippy; rustfmt; workspace WASM-target check; private/public rustdoc build.
- SDK: 48 Vitest tests, including three tests against the actual generated WASM; four Node benchmark-tool tests.
- TypeScript/Vue: lint, typecheck, browser-fixture typecheck, core/Vue/site builds and frozen-lock installation.
- Production WASM: Rust release build with --no-opt succeeds (696,851 bytes on this run); Binaryen optimization was not part of this measurement.
- Browser: dedicated-worker repeated import/export/release, stateless centroid/contains/union/Voronoi, SharedWorker concurrent clients and independent clear/disconnect, zero budget; all passed in the in-app browser.
- Playground: area/centroid/buffer/simplify/length, unsupported-operation error, recovery, zero arena allocation after each run, and production-preview route leave/reentry verified.
- Screenshots: [development](evidence/playground-2026-10-09.png), [production](evidence/playground-production-2026-10-09.png).
- GitHub: the first two delivered commits passed CI. The final CI run must be checked after the remaining commits are pushed; local gates are evidence of local validation, not a substitute for hosted CI.

The browser smoke page is a repeatable manual suite at /tests/browser-smoke.html under the site dev server. It is typechecked and linted; CI currently runs real WASM in Node but does not automate browser UI interaction. No fresh native-vs-Turf or end-to-end benchmark suite was run in this audit, so historical site figures are not updated as current results.

## Continuing priorities

### Second audit implementation — 2026-10-09

- Import now rejects non-finite XY coordinates, short/open/zero-area polygon rings and empty MultiPolygon members before unsafe indexing or buffering. Direct Rust buffering also validates geometry and finite distance. Empty multi-geometries remain supported. These are structural/numeric checks, not a complete polygon topology validator.
- Real generated-WASM regression verifies that failed imports and NaN buffer distances leave the same Engine usable, with unchanged arena state; native text/MessagePack regressions and mixed-valid/NaN bbox coverage are included.

1. **Coordinate and unit contract.** Measurements and polygon buffering are planar; the current buffer unit argument and Vue units option do not perform geodesic conversion. Define explicit Cartesian versus WGS84 semantics and add metric-buffer reference cases before expanding the API.
2. **Input validation and resource bounds.** Audit malformed/empty polygon rings, non-finite coordinates, grid cell sizes and unbounded output growth; add failure tests before changing rejection rules. Arena budgets estimate geometry storage rather than all WASM allocations.
3. **Grid accuracy and performance.** Voronoi uses a sampled approximation; isolines use a simple triangulation and disconnected segments; hex spacing/side-length semantics need mathematical reference tests. Exact tessellation/interpolation should preserve a clear API contract.
4. **Long-running and reactive lifetimes.** Bound SDK freed-handle history; test overlapping Vue executions, rendering of Features/collections/holes, null updates and component resizing. SharedWorker clients should be explicitly destroyed; abrupt tab termination needs separate lifecycle coverage.
5. **Browser CI and performance measurements.** Automate the existing smoke suite against packaged release output, measure import/operation/export separately, and derive reproducible native/browser baselines before adding hard performance gates.
6. **Packaging.** Evaluate external WASM assets to avoid embedding the same binary in dedicated/shared worker bundles; verify packed npm consumers and Vue CSS exports before publishing. OffscreenCanvas, module splitting and framework examples remain separate feature work.

Continue with one independently reviewable improvement at a time. Recheck the live worktree and relevant CodeGraph context, add a regression for the real behavior, rebuild WASM when Rust changes, run affected gates and browser states, then commit and push. Preserve user changes. Update this audit with new evidence and remaining priorities.
