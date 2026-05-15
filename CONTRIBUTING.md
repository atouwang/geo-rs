# Contributing to geo-rs

## Setup

```bash
# Rust
cargo build
cargo test --workspace --exclude geo-wasm

# WASM target
rustup target add wasm32-unknown-unknown
cargo check --target wasm32-unknown-unknown

# TypeScript
pnpm install
pnpm typecheck
```

## Project Structure

- `crates/` — Rust workspace (7 crates)
- `packages/` — TypeScript/Vue (core, vue, site)
- `benches/` — Criterion + JS benchmarks
- `docs/` — Architecture and implementation docs

## Testing

```bash
cargo test --workspace --exclude geo-wasm && cargo test -p geo-wasm
pnpm typecheck
```

## Commit Style

Conventional commits: `feat:`, `fix:`, `docs:`, `test:`, `ci:`.

## License

MIT
