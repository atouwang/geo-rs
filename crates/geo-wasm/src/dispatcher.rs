use geo_core::types::*;
use crate::arena::MemoryArena;

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
    pub arena: MemoryArena,
}

impl WasmEngine {
    pub fn new() -> Self { Self { arena: MemoryArena::new() } }

    pub fn load_bytes(&mut self, data: &[u8]) -> Result<u64, String> {
        let geom = geo_core::convert::from_msgpack(data).map_err(|e| e.to_string())?;
        self.arena.store(geom).map_err(|e| e.to_string())
    }

    pub fn read_bytes(&self, handle: u64) -> Result<Vec<u8>, String> {
        let geom = self.arena.get(handle).map_err(|e| e.to_string())?;
        geo_core::convert::to_msgpack(geom).map_err(|e| e.to_string())
    }

    pub fn execute_unary(&mut self, op_code: u8, handle: u64, param: f64) -> Result<u64, String> {
        let geom = self.arena.get(handle).map_err(|e| e.to_string())?;
        let result = dispatch_unary(op_code, geom, param)?;
        self.arena.store(result).map_err(|e| e.to_string())
    }

    pub fn execute_binary(&mut self, op_code: u8, ha: u64, hb: u64) -> Result<u64, String> {
        let ga = self.arena.get(ha).map_err(|e| e.to_string())?;
        let gb = self.arena.get(hb).map_err(|e| e.to_string())?;
        let result = dispatch_binary(op_code, ga, gb)?;
        self.arena.store(result).map_err(|e| e.to_string())
    }

    pub fn execute_bool(&self, op_code: u8, ha: u64, hb: u64) -> Result<bool, String> {
        let ga = self.arena.get(ha).map_err(|e| e.to_string())?;
        let gb = self.arena.get(hb).map_err(|e| e.to_string())?;
        dispatch_predicate(op_code, ga, gb)
    }

    pub fn execute_measure(&self, op_code: u8, handle: u64) -> Result<f64, String> {
        let geom = self.arena.get(handle).map_err(|e| e.to_string())?;
        dispatch_measure(op_code, geom)
    }

    pub fn free(&mut self, handle: u64) -> Result<(), String> {
        self.arena.remove(handle).map_err(|e| e.to_string())
    }

    pub fn free_all(&mut self) { self.arena.clear(); }

    pub fn stats_json(&self) -> String {
        let s = self.arena.stats();
        format!(r#"{{"active":{},"refs":{},"allocated":{},"max":{}}}"#,
            s.active_geometries, s.total_references, s.total_allocated, s.max_memory)
    }

    pub fn points_within(&self, pts_handle: u64, poly_handle: u64) -> Result<Vec<usize>, String> {
        let pts_geom = self.arena.get(pts_handle).map_err(|e| e.to_string())?;
        let poly_geom = self.arena.get(poly_handle).map_err(|e| e.to_string())?;
        let pts = match pts_geom {
            Geometry::MultiPoint(mp) => &mp.points,
            _ => return Err("points_within requires MultiPoint".to_string()),
        };
        let mut result = Vec::new();
        for (i, pt) in pts.iter().enumerate() {
            if geo_bool::predicates::contains(poly_geom, &Geometry::Point(*pt)) {
                result.push(i);
            }
        }
        Ok(result)
    }

    pub fn transform_coords(&mut self, handle: u64, from_cs: u8, to_cs: u8) -> Result<u64, String> {
        let geom = self.arena.get(handle).map_err(|e| e.to_string())?;
        let from = cs_from_u8(from_cs)?;
        let to = cs_from_u8(to_cs)?;
        let transformed = transform_geom(geom, from, to);
        self.arena.store(transformed).map_err(|e| e.to_string())
    }

    pub fn voronoi_from_pts(&mut self, pts_handle: u64, bbox_json: &str) -> Result<u64, String> {
        let pts_geom = self.arena.get(pts_handle).map_err(|e| e.to_string())?;
        let pts = match pts_geom {
            Geometry::MultiPoint(mp) => mp.points.clone(),
            _ => return Err("voronoi requires MultiPoint".to_string()),
        };
        let bbox: geo_core::types::BBox = serde_json::from_str(bbox_json)
            .map_err(|e| format!("Invalid bbox JSON: {}", e))?;
        let polys = geo_grid::voronoi::voronoi(&pts, &bbox);
        let result = Geometry::GeometryCollection(
            polys.into_iter().map(Geometry::Polygon).collect()
        );
        self.arena.store(result).map_err(|e| e.to_string())
    }

    pub fn isolines_from_pts(&mut self, pts_handle: u64, values_json: &str, breaks_json: &str) -> Result<u64, String> {
        let pts_geom = self.arena.get(pts_handle).map_err(|e| e.to_string())?;
        let pts = match pts_geom {
            Geometry::MultiPoint(mp) => mp.points.clone(),
            _ => return Err("isolines requires MultiPoint".to_string()),
        };
        let values: Vec<f64> = serde_json::from_str(values_json)
            .map_err(|e| format!("Invalid values JSON: {}", e))?;
        let breaks: Vec<f64> = serde_json::from_str(breaks_json)
            .map_err(|e| format!("Invalid breaks JSON: {}", e))?;
        let line_strings = geo_grid::isolines::isolines(&pts, &values, &breaks);
        let result = Geometry::MultiLineString(MultiLineString { lines: line_strings });
        self.arena.store(result).map_err(|e| e.to_string())
    }
}

fn cs_from_u8(cs: u8) -> Result<geo_core::types::CoordSystem, String> {
    match cs {
        0 => Ok(geo_core::types::CoordSystem::Wgs84),
        1 => Ok(geo_core::types::CoordSystem::WebMercator),
        2 => Ok(geo_core::types::CoordSystem::Cartesian),
        _ => Err(format!("Unknown coordinate system: {}", cs)),
    }
}

fn transform_geom(geom: &Geometry, from: geo_core::types::CoordSystem, to: geo_core::types::CoordSystem) -> Geometry {
    use geo_core::coords::transform_coords;
    match geom {
        Geometry::Point(p) => Geometry::Point(transform_coords(p, from, to)),
        Geometry::LineString(ls) => Geometry::LineString(LineString {
            coords: ls.coords.iter().map(|p| transform_coords(p, from, to)).collect(),
        }),
        Geometry::Polygon(p) => Geometry::Polygon(Polygon {
            exterior: LineString { coords: p.exterior.coords.iter().map(|pt| transform_coords(pt, from, to)).collect() },
            interiors: p.interiors.iter().map(|ls| LineString { coords: ls.coords.iter().map(|pt| transform_coords(pt, from, to)).collect() }).collect(),
        }),
        Geometry::MultiLineString(mls) => Geometry::MultiLineString(MultiLineString {
            lines: mls.lines.iter().map(|ls| LineString { coords: ls.coords.iter().map(|pt| transform_coords(pt, from, to)).collect() }).collect(),
        }),
        Geometry::MultiPoint(mp) => Geometry::MultiPoint(MultiPoint {
            points: mp.points.iter().map(|p| transform_coords(p, from, to)).collect(),
        }),
        other => other.clone(),
    }
}

fn dispatch_unary(op: u8, geom: &Geometry, param: f64) -> Result<Geometry, String> {
    match op {
        OP_BUFFER => geo_algo::buffer::buffer(geom, param, Units::Meters).map_err(|e| e.to_string()),
        OP_SIMPLIFY => geo_algo::simplify::simplify(geom, param).map_err(|e| e.to_string()),
        OP_CENTROID => geo_core::measure::centroid(geom)
            .map(|pt| Geometry::Point(pt))
            .ok_or_else(|| "centroid undefined for this geometry".to_string()),
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

    fn pt_geom(x: f64, y: f64) -> Geometry { Geometry::Point(Point { x, y }) }
    fn square_geom() -> Geometry {
        Geometry::Polygon(Polygon {
            exterior: LineString { coords: vec![
                Point { x: 0.0, y: 0.0 }, Point { x: 1.0, y: 0.0 },
                Point { x: 1.0, y: 1.0 }, Point { x: 0.0, y: 1.0 },
                Point { x: 0.0, y: 0.0 },
            ]},
            interiors: vec![],
        })
    }

    fn encode(g: &Geometry) -> Vec<u8> { geo_core::convert::to_msgpack(g).unwrap() }

    #[test] fn test_load_read() {
        let mut e = WasmEngine::new();
        let h = e.load_bytes(&encode(&pt_geom(10.0, 20.0))).unwrap();
        let bytes = e.read_bytes(h).unwrap();
        let geom: Geometry = rmp_serde::from_slice(&bytes).unwrap();
        assert!(matches!(geom, Geometry::Point(_)));
    }
    #[test] fn test_area() {
        let mut e = WasmEngine::new();
        let h = e.load_bytes(&encode(&square_geom())).unwrap();
        assert!(e.execute_measure(OP_AREA, h).unwrap() > 0.0);
    }
    #[test] fn test_contains() {
        let mut e = WasmEngine::new();
        let hp = e.load_bytes(&encode(&square_geom())).unwrap();
        let ht = e.load_bytes(&encode(&pt_geom(0.5, 0.5))).unwrap();
        assert!(e.execute_bool(OP_CONTAINS, hp, ht).unwrap());
    }
    #[test] fn test_buffer() {
        let mut e = WasmEngine::new();
        let h = e.load_bytes(&encode(&square_geom())).unwrap();
        let hb = e.execute_unary(OP_BUFFER, h, 0.1).unwrap();
        let _bytes = e.read_bytes(hb).unwrap();
    }
    #[test] fn test_union() {
        let mut e = WasmEngine::new();
        let h1 = e.load_bytes(&encode(&square_geom())).unwrap();
        let h2 = e.load_bytes(&encode(&square_geom())).unwrap();
        let h3 = e.execute_binary(OP_UNION, h1, h2).unwrap();
        let _bytes = e.read_bytes(h3).unwrap();
    }
    #[test] fn test_free() {
        let mut e = WasmEngine::new();
        let h = e.load_bytes(&encode(&pt_geom(1.0, 2.0))).unwrap();
        assert!(e.free(h).is_ok());
        assert!(e.read_bytes(h).is_err());
    }
}
