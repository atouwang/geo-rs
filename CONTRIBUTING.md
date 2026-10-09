# Contributing to geo-rs

## Clean setup

Use Rust stable, Node.js 22+ and pnpm 10.32.1 (pinned in package.json).

```sh
rustup target add wasm32-unknown-unknown
cargo install wasm-pack --locked
wasm-pack build --dev --target web crates/geo-wasm
pnpm install --frozen-lockfile
pnpm build
pnpm dev
```

The generated WASM package must exist before workspace installation. Rebuild it after Rust changes. Generated pkg/, dist/ and target/ directories are ignored.

## Verification

```sh
cargo fmt --all -- --check
cargo clippy --workspace --all-targets -- -D warnings
cargo test --workspace
cargo check --workspace --target wasm32-unknown-unknown
cargo doc --workspace --no-deps --document-private-items
pnpm test
pnpm lint
pnpm typecheck
pnpm build
```

pnpm test runs SDK tests, real WASM transport tests, Vue concurrency/rendering tests and benchmark-gate tests. It requires the generated WASM package. Lint checks both TypeScript and Vue SFCs; build includes the core library, Vue library and site.

With pnpm dev running, open http://127.0.0.1:5173/tests/browser-smoke.html and run the browser suite. It covers actual dedicated/shared workers, stateless APIs, reference cleanup and client memory-budget isolation. Also check the Playground success, failure, recovery and route-unmount states. Recheck production output using vite preview after pnpm build.

The Vue suite at /tests/browser-vue.html checks actual canvas pixels for transparent holes, resize/nested updates/null clearing, concurrent buffer calls and scope disposal. To run both suites against packaged production output after pnpm build:

```sh
pnpm --filter @geo-rs/site build:browser
pnpm --filter @geo-rs/site exec vite preview --outDir dist/browser --host 127.0.0.1
```

Open /tests/browser-smoke.html and /tests/browser-vue.html at the preview URL and click their run buttons. The suites are built/linted/typechecked, but browser execution remains manual in CI.

## Production and benchmarks

```sh
wasm-pack build --release --target web crates/geo-wasm
pnpm build
cargo bench --bench core_ops
pnpm bench -- HEAD^
```

Optional: add --no-opt to skip Binaryen optimization while retaining Rust release optimization.

Benchmark CI runs on push, pull requests and manual dispatch. It compiles both versions first, then pairs each case on the same runner using the candidate's 16-case harness with alternating order. Push compares with the pre-push commit, PR compares with its base SHA, and manual dispatch defaults to HEAD^. Missing/invalid output and build errors fail the job. A median slowdown over 15% with separated 95% confidence intervals is rerun on both commits with reversed order; a confirmed regression fails CI. Reports, metadata and logs are uploaded for 30 days. See [PERFORMANCE.md](docs/PERFORMANCE.md) for reproduction and limits. The old .github/benchmarks/baseline.json is historical and is no longer used as a gate.

## Structure and commits

See [ONBOARDING.md](docs/ONBOARDING.md) for the layer map and [OPTIMIZATION.md](docs/OPTIMIZATION.md) for audited priorities.

Use conventional commits (fix:, perf:, test:, ci:, docs:). Keep correctness changes separate from packaging/documentation where practical. Include tests that exercise the user-visible failure being repaired.

MIT license.
