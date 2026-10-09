pub fn dummy() {}
// Benchmark the arena implementation without linking WASM exports or its allocator.
#[path = "../../crates/geo-wasm/src/arena.rs"]
pub mod arena;
