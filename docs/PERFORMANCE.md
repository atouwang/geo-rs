# Performance regression checks

## Native CI gate

The benchmark job runs on every push to main/master, every PR targeting those branches, and workflow_dispatch. It is a required failure of the workflow when measurements/builds fail; whether GitHub blocks a merge also depends on repository branch protection.

- Push baseline: github.event.before (the pre-push commit, including multi-commit pushes).
- PR baseline: the PR base SHA; the candidate is GitHub's checked-out merge commit.
- Manual/local default: HEAD^. An explicit local SHA can be supplied with pnpm bench -- <SHA>.
- Both implementations are compiled before measurement, then each case is paired sequentially on one runner with alternating order. Both use the candidate's benchmark package and resolved Cargo.lock, allowing new harness dependencies with identical resolution in both builds. The baseline implementation and its crate manifests remain unchanged. This reduces compilation/thermal/time-window bias between full suites. A baseline API/manifest/lockfile incompatibility fails explicitly and needs review; it is never silently treated as a pass. Dependency upgrade comparisons therefore hold the candidate's resolved versions constant rather than reproducing the old deployed dependency set.
- Each case uses 30 Criterion samples, one second of warmup and a two-second target measurement time (Criterion may extend this for slower cases). Setup fixtures are outside timing; allocation/output construction and destruction are included where part of the operation. Fallible operations unwrap so a fast error path cannot masquerade as a speedup.
- The checker reads median point estimates and their bootstrap 95% confidence intervals from Criterion JSON. The console time interval reports a mean and is not parsed for the gate.
- A >15% median slowdown and a candidate lower confidence bound greater than the baseline upper bound is a suspected regression. Both commits rerun the suspected cases once with reversed pair order. A repeated failure blocks the job. An overlap above the threshold is reported as NOISE. Small regressions below the threshold can pass; hosted runner noise and gradual changes still need trend review.
- Exact expected cases are listed in .github/benchmarks/suite.json. Missing/malformed estimates, nonzero benchmark exits and failed baseline builds fail closed. Cached estimates for measured cases are removed before each run.

## Workloads

Eighteen cases cover area, centroid, buffer, simplify (1,000 points), contains, union, RTree empty search and 400-result search on 10,000 points, RTree construction, small/10,000-point GeoJSON import, 10,000-point MessagePack import/export, sampled Voronoi with 100/1,000 sites, a Beijing hex grid, and arena store/release plus deduplication on 10,000 points. Arena cases compile the actual arena source through the benchmark library, without linking WASM exports or its allocator. Input cloning happens in Criterion's untimed batch setup; storage, hashing, equality checks and release/destruction are measured. Deterministic fixtures make comparisons reproducible without timing fixture generation. Voronoi remains an approximation; the benchmark does not certify its geometry accuracy.

## Reproduction and artifacts

```sh
pnpm test:tooling
pnpm bench -- HEAD^
# An individual native run without comparison:
cargo bench --locked -p geo-bench --bench core_ops -- --noplot --sample-size 30 --warm-up-time 1 --measurement-time 2
# Recheck saved JSON with the same gate:
node .github/scripts/bench-compare.mjs target/performance/run-XXXX/baseline.json target/performance/run-XXXX/current.json
```

The runner creates and removes its own detached temporary baseline worktree. It copies only the candidate benchmark package and Cargo.lock into that worktree. The current checkout is preserved. Build caches are separate at target/bench-baseline and target/bench-current. Avoid concurrent benchmark runners sharing those directories and avoid unrelated CPU-intensive work during local measurements.

Reports are written under target/performance/run-* and uploaded by CI for 30 days, including when a benchmark command fails. They include both commit IDs, dirty-checkout status, Rust toolchain, CPU/OS, configuration, baseline/current median estimates and confidence bounds, first/final comparisons, a Markdown table and raw logs. Potential regressions retain their confirmation logs. GitHub's job summary shows the comparison table. The original .github/benchmarks/baseline.json has unknown machine provenance and is retained only as historical data.

## Measurement boundaries

Initial local calibration on 2026-10-09 passed all 16 cases against the unchanged native implementation at 76c174c on Windows, Intel Core i7-14650HX, Rust 1.95.0. Two first-pass suspects passed the reversed-order confirmation. This validates complete output and the confirmation path, not a performance improvement. [Saved medians, bounds and metadata](evidence/native-benchmarks-2026-10-09.json) include the dirty checkout flag and harness hash; CI independently measures its own runner and commits.

These timings exercise optimized native Rust. They exclude WASM startup, Worker/RPC, JavaScript encoding/decoding, browser rendering, and total memory peaks. Real-WASM and browser regression tests cover correctness but do not constitute performance measurements. Browser phase timings and stable long-term benchmark trend storage remain follow-up work. This gate detects sizeable regressions in covered workloads; it cannot guarantee all product performance or substantiate a Turf.js speedup.
