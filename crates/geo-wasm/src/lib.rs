pub mod arena;
pub mod dispatcher;

use dispatcher::WasmEngine;
use wasm_bindgen::prelude::*;

#[global_allocator]
static ALLOC: wee_alloc::WeeAlloc = wee_alloc::WeeAlloc::INIT;

#[wasm_bindgen(start)]
pub fn init() {
    console_error_panic_hook::set_once();
}

#[wasm_bindgen]
pub struct Engine {
    inner: WasmEngine,
}

#[wasm_bindgen]
impl Engine {
    #[wasm_bindgen(constructor)]
    pub fn new() -> Self {
        Self { inner: WasmEngine::new() }
    }

    /// Load GeoJSON into WASM memory, returns a handle (u64 as JsValue).
    pub fn load(&mut self, geojson: &str) -> Result<u64, JsValue> {
        self.inner.load_geojson(geojson).map_err(|e| JsValue::from_str(&e))
    }

    /// Read the geometry at `handle` back as GeoJSON string.
    pub fn read(&self, handle: u64) -> Result<String, JsValue> {
        self.inner.read_geojson(handle).map_err(|e| JsValue::from_str(&e))
    }

    /// Execute a unary operation (buffer, simplify, convex_hull…).
    /// `param` is the main numeric parameter (distance, tolerance…).
    pub fn execute_unary(&mut self, op_code: u8, handle: u64, param: f64) -> Result<u64, JsValue> {
        self.inner.execute_unary(op_code, handle, param).map_err(|e| JsValue::from_str(&e))
    }

    /// Execute a binary set operation (union, intersect, difference, xor).
    pub fn execute_binary(&mut self, op_code: u8, handle_a: u64, handle_b: u64) -> Result<u64, JsValue> {
        self.inner.execute_binary(op_code, handle_a, handle_b).map_err(|e| JsValue::from_str(&e))
    }

    /// Execute a spatial predicate (contains, intersects, crosses…).
    pub fn execute_bool(&self, op_code: u8, handle_a: u64, handle_b: u64) -> Result<bool, JsValue> {
        self.inner.execute_bool(op_code, handle_a, handle_b).map_err(|e| JsValue::from_str(&e))
    }

    /// Execute a measurement operation (area, length…).
    pub fn execute_measure(&self, op_code: u8, handle: u64) -> Result<f64, JsValue> {
        self.inner.execute_measure(op_code, handle).map_err(|e| JsValue::from_str(&e))
    }

    /// Free a geometry from the arena.
    pub fn free(&mut self, handle: u64) -> Result<(), JsValue> {
        self.inner.free(handle).map_err(|e| JsValue::from_str(&e))
    }

    /// Free all geometries.
    pub fn free_all(&mut self) {
        self.inner.free_all();
    }

    /// Get arena statistics as JSON.
    pub fn stats(&self) -> String {
        self.inner.stats_json()
    }
}

// Export op code constants for JS
#[wasm_bindgen]
pub fn op_area() -> u8 { dispatcher::OP_AREA }
#[wasm_bindgen]
pub fn op_length() -> u8 { dispatcher::OP_LENGTH }
#[wasm_bindgen]
pub fn op_centroid() -> u8 { dispatcher::OP_CENTROID }
#[wasm_bindgen]
pub fn op_bbox() -> u8 { dispatcher::OP_BBOX }
#[wasm_bindgen]
pub fn op_buffer() -> u8 { dispatcher::OP_BUFFER }
#[wasm_bindgen]
pub fn op_simplify() -> u8 { dispatcher::OP_SIMPLIFY }
#[wasm_bindgen]
pub fn op_convex_hull() -> u8 { dispatcher::OP_CONVEX_HULL }
#[wasm_bindgen]
pub fn op_contains() -> u8 { dispatcher::OP_CONTAINS }
#[wasm_bindgen]
pub fn op_intersects() -> u8 { dispatcher::OP_INTERSECTS }
#[wasm_bindgen]
pub fn op_crosses() -> u8 { dispatcher::OP_CROSSES }
#[wasm_bindgen]
pub fn op_within() -> u8 { dispatcher::OP_WITHIN }
#[wasm_bindgen]
pub fn op_disjoint() -> u8 { dispatcher::OP_DISJOINT }
#[wasm_bindgen]
pub fn op_overlaps() -> u8 { dispatcher::OP_OVERLAPS }
#[wasm_bindgen]
pub fn op_touches() -> u8 { dispatcher::OP_TOUCHES }
#[wasm_bindgen]
pub fn op_equals() -> u8 { dispatcher::OP_EQUALS }
#[wasm_bindgen]
pub fn op_union() -> u8 { dispatcher::OP_UNION }
#[wasm_bindgen]
pub fn op_intersect() -> u8 { dispatcher::OP_INTERSECT }
#[wasm_bindgen]
pub fn op_difference() -> u8 { dispatcher::OP_DIFFERENCE }
#[wasm_bindgen]
pub fn op_xor() -> u8 { dispatcher::OP_XOR }
