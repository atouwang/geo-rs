use geo_core::types::Geometry;
use crate::arena::MemoryArena;
use geo_algo;
use geo_bool;
use geo_set;

// Operation codes
pub const OP_AREA: u8 = 0x01;
pub const OP_LENGTH: u8 = 0x02;
pub const OP_CENTROID: u8 = 0x03;
pub const OP_BBOX: u8 = 0x04;
pub const OP_BUFFER: u8 = 0x10;
pub const OP_SIMPLIFY: u8 = 0x11;
pub const OP_CONVEX_HULL: u8 = 0x12;
pub const OP_CONTAINS: u8 = 0x20;
pub const OP_INTERSECTS: u8 = 0x21;
pub const OP_CROSSES: u8 = 0x22;
pub const OP_WITHIN: u8 = 0x23;
pub const OP_DISJOINT: u8 = 0x24;
pub const OP_OVERLAPS: u8 = 0x25;
pub const OP_TOUCHES: u8 = 0x26;
pub const OP_EQUALS: u8 = 0x27;
pub const OP_UNION: u8 = 0x30;
pub const OP_INTERSECT: u8 = 0x31;
pub const OP_DIFFERENCE: u8 = 0x32;
pub const OP_XOR: u8 = 0x33;

pub struct WasmEngine {
    arena: MemoryArena,
}

impl WasmEngine {
    pub fn new() -> Self {
        Self { arena: MemoryArena::new() }
    }

    pub fn load_geojson(&mut self, json: &str) -> Result<u64, String> {
        let geom = geo_core::convert::from_geojson(json).map_err(|e| e.to_string())?;
        self.arena.store(geom).map_err(|e| e.to_string())
    }

    pub fn read_geojson(&self, handle: u64) -> Result<String, String> {
        let geom = self.arena.get(handle).map_err(|e| e.to_string())?;
        geo_core::convert::to_geojson(geom).map_err(|e| e.to_string())
    }

    pub fn execute_unary(
        &mut self,
        op_code: u8,
        handle: u64,
        param: f64,
    ) -> Result<u64, String> {
        let geom = self.arena.get(handle).map_err(|e| e.to_string())?;
        let result = dispatch_unary(op_code, geom, param)?;
        self.arena.store(result).map_err(|e| e.to_string())
    }

    pub fn execute_binary(
        &mut self,
        op_code: u8,
        handle_a: u64,
        handle_b: u64,
    ) -> Result<u64, String> {
        let geom_a = self.arena.get(handle_a).map_err(|e| e.to_string())?;
        let geom_b = self.arena.get(handle_b).map_err(|e| e.to_string())?;
        let result = dispatch_binary(op_code, geom_a, geom_b)?;
        self.arena.store(result).map_err(|e| e.to_string())
    }

    pub fn execute_bool(
        &self,
        op_code: u8,
        handle_a: u64,
        handle_b: u64,
    ) -> Result<bool, String> {
        let geom_a = self.arena.get(handle_a).map_err(|e| e.to_string())?;
        let geom_b = self.arena.get(handle_b).map_err(|e| e.to_string())?;
        dispatch_predicate(op_code, geom_a, geom_b)
    }

    pub fn execute_measure(
        &self,
        op_code: u8,
        handle: u64,
    ) -> Result<f64, String> {
        let geom = self.arena.get(handle).map_err(|e| e.to_string())?;
        dispatch_measure(op_code, geom)
    }

    pub fn free(&mut self, handle: u64) -> Result<(), String> {
        self.arena.remove(handle).map_err(|e| e.to_string())
    }

    pub fn free_all(&mut self) {
        self.arena.clear();
    }

    pub fn stats_json(&self) -> String {
        let s = self.arena.stats();
        format!(
            r#"{{"active":{},"allocated":{},"max":{}}}"#,
            s.active_geometries, s.total_allocated, s.max_memory
        )
    }
}

fn dispatch_unary(op: u8, geom: &Geometry, param: f64) -> Result<Geometry, String> {
    match op {
        OP_BUFFER => geo_algo::buffer::buffer(geom, param, geo_core::types::Units::Meters)
            .map_err(|e| e.to_string()),
        OP_SIMPLIFY => geo_algo::simplify::simplify(geom, param)
            .map_err(|e| e.to_string()),
        _ => Err(format!("Unknown unary op: {}", op)),
    }
}

fn dispatch_binary(op: u8, a: &Geometry, b: &Geometry) -> Result<Geometry, String> {
    match op {
        OP_UNION => geo_set::set_ops::union(a, b).map_err(|e| e.to_string()),
        OP_INTERSECT => geo_set::set_ops::intersect(a, b).map_err(|e| e.to_string()),
        OP_DIFFERENCE => geo_set::set_ops::difference(a, b).map_err(|e| e.to_string()),
        OP_XOR => geo_set::set_ops::xor(a, b).map_err(|e| e.to_string()),
        _ => Err(format!("Unknown binary op: {}", op)),
    }
}

fn dispatch_predicate(op: u8, a: &Geometry, b: &Geometry) -> Result<bool, String> {
    match op {
        OP_CONTAINS => Ok(geo_bool::predicates::contains(a, b)),
        OP_INTERSECTS => Ok(geo_bool::predicates::intersects(a, b)),
        OP_CROSSES => Ok(geo_bool::predicates::crosses(a, b)),
        OP_WITHIN => Ok(geo_bool::predicates::within(a, b)),
        OP_DISJOINT => Ok(geo_bool::predicates::disjoint(a, b)),
        OP_OVERLAPS => Ok(geo_bool::predicates::overlaps(a, b)),
        OP_TOUCHES => Ok(geo_bool::predicates::touches(a, b)),
        OP_EQUALS => Ok(geo_bool::predicates::equals(a, b)),
        _ => Err(format!("Unknown predicate op: {}", op)),
    }
}

fn dispatch_measure(op: u8, geom: &Geometry) -> Result<f64, String> {
    match op {
        OP_AREA => Ok(geo_core::measure::area(geom)),
        OP_LENGTH => Ok(geo_core::measure::length(geom)),
        _ => Err(format!("Unknown measure op: {}", op)),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn point_json(x: f64, y: f64) -> String {
        format!(r#"{{"type":"Point","coordinates":[{},{}]}}"#, x, y)
    }

    fn polygon_json() -> String {
        r#"{"type":"Polygon","coordinates":[[[0,0],[1,0],[1,1],[0,1],[0,0]]]}"#.to_string()
    }

    #[test]
    fn test_load_and_read() {
        let mut engine = WasmEngine::new();
        let h = engine.load_geojson(&point_json(10.0, 20.0)).unwrap();
        let json = engine.read_geojson(h).unwrap();
        assert!(json.contains("10.0"));
        assert!(json.contains("20.0"));
    }

    #[test]
    fn test_area() {
        let mut engine = WasmEngine::new();
        let h = engine.load_geojson(&polygon_json()).unwrap();
        let area = engine.execute_measure(OP_AREA, h).unwrap();
        assert!(area > 0.0);
    }

    #[test]
    fn test_contains() {
        let mut engine = WasmEngine::new();
        let h_poly = engine.load_geojson(&polygon_json()).unwrap();
        let h_pt = engine.load_geojson(&point_json(0.5, 0.5)).unwrap();
        assert!(engine.execute_bool(OP_CONTAINS, h_poly, h_pt).unwrap());
    }

    #[test]
    fn test_buffer() {
        let mut engine = WasmEngine::new();
        let h = engine.load_geojson(&polygon_json()).unwrap();
        let h_buf = engine.execute_unary(OP_BUFFER, h, 0.1).unwrap();
        let json = engine.read_geojson(h_buf).unwrap();
        assert!(json.contains("Polygon") || json.contains("MultiPolygon"));
    }

    #[test]
    fn test_union() {
        let mut engine = WasmEngine::new();
        let h1 = engine.load_geojson(&polygon_json()).unwrap();
        let h2 = engine.load_geojson(&polygon_json()).unwrap();
        let h3 = engine.execute_binary(OP_UNION, h1, h2).unwrap();
        let json = engine.read_geojson(h3).unwrap();
        assert!(json.contains("Polygon") || json.contains("MultiPolygon"));
    }

    #[test]
    fn test_free() {
        let mut engine = WasmEngine::new();
        let h = engine.load_geojson(&point_json(1.0, 2.0)).unwrap();
        assert!(engine.free(h).is_ok());
        assert!(engine.read_geojson(h).is_err());
    }
}
