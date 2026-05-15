use geo_core::types::*;

pub fn voronoi(points: &[Point], _bbox: &BBox) -> Vec<Polygon> {
    if points.len() < 3 { return vec![]; }
    vec![]
}
