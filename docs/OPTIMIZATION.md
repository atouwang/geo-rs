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

## Verification — first audit

- Rust: 67 unit tests; strict all-target Clippy; rustfmt; workspace WASM-target check; private/public rustdoc build.
- SDK: 48 Vitest tests, including three tests against the actual generated WASM; four Node benchmark-tool tests.
- TypeScript/Vue: lint, typecheck, browser-fixture typecheck, core/Vue/site builds and frozen-lock installation.
- Production WASM: Rust release build with --no-opt succeeds (696,851 bytes on this run); Binaryen optimization was not part of this measurement.
- Browser: dedicated-worker repeated import/export/release, stateless centroid/contains/union/Voronoi, SharedWorker concurrent clients and independent clear/disconnect, zero budget; all passed in the in-app browser.
- Playground: area/centroid/buffer/simplify/length, unsupported-operation error, recovery, zero arena allocation after each run, and production-preview route leave/reentry verified.
- Screenshots: [development](evidence/playground-2026-10-09.png), [production](evidence/playground-production-2026-10-09.png).
- GitHub: the first audit's final commit c2bf34a passed all applicable hosted CI jobs.

The browser smoke page is a repeatable manual suite at /tests/browser-smoke.html under the site dev server. It is typechecked and linted; CI currently runs real WASM in Node but does not automate browser UI interaction. No fresh native-vs-Turf or end-to-end benchmark suite was run in this audit, so historical site figures are not updated as current results.

## Continuing priorities

### Second audit implementation — 2026-10-09

- Import now rejects non-finite XY coordinates, short/open/zero-area polygon rings and empty MultiPolygon members before unsafe indexing or buffering. Direct Rust buffering also validates geometry and finite distance. Empty multi-geometries remain supported. These are structural/numeric checks, not a complete polygon topology validator.
- Real generated-WASM regression verifies that failed imports and NaN buffer distances leave the same Engine usable, with unchanged arena state; native text/MessagePack regressions and mixed-valid/NaN bbox coverage are included.
- Hex grids now use the requested projected side length, standard pointy-top spacing and border rows/columns covering the full bbox. Miles convert to 1609.344 meters. The Rust API returns Result and rejects invalid size/bbox and requests exceeding 100,000 cells before allocation. Tests check every side, 441 coverage samples, units and failure bounds. Mercator distortion remains explicit; cells are not clipped at bbox edges.
- SDK active references are stored separately from a bounded 1,024-entry released-handle diagnostic history. Statistics now report retained history, not lifetime releases; evicted handles still reject as unknown. A 100,000-release regression verifies constant history size and live duplicate references. Statistics no longer scan the active/history maps.
- Buffer now declares Feature<MultiPolygon> throughout SDK/Vue. Vue explicitly rejects its reserved metric-units option instead of silently ignoring it. Composables keep only the latest result/error, track all pending requests for loading and ignore results after scope disposal. Seven concurrency/contract tests cover these rules.
- GeoCanvas renders Features/collections, all primitive geometry types and evenodd polygon holes. Post-update deep/dimension watching redraws after resize or nested changes; null/empty/invalid input clears safely. Three path-level tests and a real browser pixel suite verify holes, resizing, mutation, clearing, concurrent WASM buffer and scope disposal. Vue CSS is available through @geo-rs/vue/style.css. Evidence: [Vue browser regression](evidence/vue-regression-2026-10-09.png).
- Sampled Voronoi now restricts output to the supplied bbox, uses normalized RTree nearest-neighbor queries instead of scanning every point per sample and returns regions in stable input-index order. The Rust API returns Result and rejects invalid/degenerate bounds, unsupported numeric ranges and more than 10,000 points before sampling. Tests compare boundary ownership with brute-force distances, check repeatability/bounds and verify real-WASM recovery/refcounts. The algorithm remains approximate: small/duplicate/unsampled regions may be omitted and cells do not form an exact gap-free partition. No runtime speedup is claimed without comparative measurement.
- Vue Voronoi accepts optional caller bounds; automatic bounds pad degenerate axes by 5% of the larger input span (or one coordinate unit for coincident sites), keeping collinear/coincident inputs usable. Three unit tests and production-browser cases cover this behavior.
- Added a production build of both browser suites using packaged SDK/Vue output, and CI compiles these pages. Clean-output verification exposed a missing Vue declaration prerequisite in root typecheck; it now builds core and Vue before checking dependent packages. Generated local outputs were moved aside and the full TypeScript test/lint/typecheck/build sequence passed from that state.

Second-audit verification: 76 Rust tests; 52 SDK tests (five real-WASM cases), 13 Vue tests and four tooling tests (69 JS tests total); strict Clippy, rustfmt, WASM-target checking, rustdoc, release WASM build with --no-opt, frozen/offline installation, full library/site builds and packaged browser suite build. Both production-preview browser suites pass, including invalid-import recovery, Voronoi bbox, transparent holes/resize/null/deep updates, actual concurrent buffer, scope disposal and collinear/coincident Voronoi sites. Console error/warning logs are empty. [Production evidence](evidence/vue-production-2026-10-09.png). Browser execution remains manual; no comparative speedup or complete topology/geodesic correctness is claimed.

1. **Coordinate and unit contract.** Measurements and polygon buffering remain planar; Vue now rejects explicit metric units. Define Cartesian versus WGS84 semantics and metric-buffer reference cases before expanding the API; document Mercator distortion and antimeridian limits.
2. **Topology validation and resource bounds.** Structural/numeric input checks are covered, but complete self-intersection/hole validity and temporary operation allocations still need coverage. Arena budgets estimate geometry storage rather than all WASM allocations.
3. **Grid accuracy and performance.** Voronoi remains sampled; isolines still use simple triangulation and disconnected segments. Evaluate exact tessellation/interpolation and reproducible before/after measurements; preserve a clear API contract.
4. **Long-running lifetimes.** SharedWorker clients should be explicitly destroyed; abrupt tab termination and stress profiling of live references/worker memory still need separate coverage. SDK history and Vue overlap/disposal/canvas updates are covered in the second audit.
5. **Browser CI and performance measurements.** Native push/PR/manual gates now compare 16 workloads on the same runner with confirmed regression failures (see [PERFORMANCE.md](PERFORMANCE.md)). Automate the existing browser smoke suite against packaged release output, measure browser import/operation/export separately, and add long-term performance trend storage.
6. **Packaging.** Evaluate external WASM assets to avoid embedding the same binary in dedicated/shared worker bundles; verify packed npm consumers and Vue CSS exports before publishing. OffscreenCanvas, module splitting and framework examples remain separate feature work.

### Native performance gate — 2026-10-09

Benchmark CI previously skipped push runs and tolerated failed comparisons against a fixed baseline without machine provenance. It now runs on push/PR/manual dispatch, compiles both versions first and pairs 16 deterministic workloads on one runner with alternating order. True Criterion medians and 95% intervals replace console-mean parsing. Suspected >15% regressions rerun with reversed order and confirmed failures stop CI; missing data/build errors fail explicitly. Reports include commits, harness hash, machine/toolchain metadata, raw logs and GitHub job summaries, retained for 30 days. See [PERFORMANCE.md](PERFORMANCE.md).

Validation: ten gate tests, including injected slowdown/missing-result CLI failures; benchmark-package strict Clippy, rustfmt, script lint and workflow YAML checks. Local calibration passes all 16 cases; two suspects pass confirmation. [Saved measurement evidence](evidence/native-benchmarks-2026-10-09.json) describes the unchanged native implementation and dirty benchmark/tooling checkout. This is not a measured algorithm speedup; browser phase timings and long-term trends remain open.

Continue with one independently reviewable improvement at a time. Recheck the live worktree and relevant CodeGraph context, add a regression for the real behavior, rebuild WASM when Rust changes, run affected gates and browser states, then commit and push. Preserve user changes. Update this audit with new evidence and remaining priorities.
