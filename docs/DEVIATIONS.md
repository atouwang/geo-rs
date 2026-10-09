# Architecture deviations

The current audit is maintained in [OPTIMIZATION.md](OPTIMIZATION.md). The original design and roadmap are retained separately as proposals.

- MessagePack replaces FlatBuffers. Import/export preserve the GeoJSON wire shape; operations keep geometry resident in WASM. ArrayBuffers transfer ownership between threads.
- WASM is one module. Module splitting and SharedArrayBuffer are deferred until measurements justify their complexity.
- RPC is implemented directly. Dedicated workers and SharedWorker clients use the same runtime, with independent arenas and budgets per shared port.
- A worker crash rejects requests and requires reinitialization; restarting an empty arena cannot restore lost geometry handles.
- No Turf.js fallback is supplied.
- OffscreenCanvas worker rendering is not implemented; requesting it fails explicitly before transferring the canvas.
- Only the existing planar operations are claimed. Buffer unit conversion, exact Voronoi, geographic area/perimeter and advanced grid semantics need separate algorithm work.
- Static site timing figures are historical native measurements/estimates; they do not prove current browser speedups.

See the audit for the verified fixes, tests and prioritized follow-up work.
