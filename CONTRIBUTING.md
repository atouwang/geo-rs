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

pnpm test runs SDK tests, real WASM transport tests and benchmark-parser tests. It requires the generated WASM package. Lint checks both TypeScript and Vue SFCs; build includes the core library, Vue library and site.

With pnpm dev running, open http://127.0.0.1:5173/tests/browser-smoke.html and run the browser suite. It covers actual dedicated/shared workers, stateless APIs, reference cleanup and client memory-budget isolation. Also check the Playground success, failure, recovery and route-unmount states. Recheck production output using vite preview after pnpm build.

## Production and benchmarks

```sh
wasm-pack build --release --target web crates/geo-wasm
pnpm build
cargo bench --bench core_ops
```

Optional: add --no-opt to skip Binaryen optimization while retaining Rust release optimization.

Benchmark CI currently reports a warning against a stored, machine-specific native baseline. Missing output or a regression over 5% makes the comparison fail. Native timings do not measure Worker/RPC/import/export costs or prove a browser speedup over Turf.js.

## Structure and commits

See [ONBOARDING.md](docs/ONBOARDING.md) for the layer map and [OPTIMIZATION.md](docs/OPTIMIZATION.md) for audited priorities.

Use conventional commits (fix:, perf:, test:, ci:, docs:). Keep correctness changes separate from packaging/documentation where practical. Include tests that exercise the user-visible failure being repaired.

MIT license.
