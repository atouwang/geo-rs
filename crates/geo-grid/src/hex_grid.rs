use geo_core::types::*;
use std::f64::consts::PI;

pub fn hex_grid(bbox: &BBox, cell_side: f64, units: Units) -> Vec<Polygon> {
    let cell_m = match units {
        Units::Meters => cell_side,
        Units::Kilometers => cell_side * 1000.0,
        _ => cell_side * 111_320.0,
    };
    let hex_w = 3.0_f64.sqrt() * cell_m;
    let hex_h = 2.0 * cell_m;

    let (min_x, max_x) = mercator_x_range(bbox);
    let (min_y, max_y) = mercator_y_range(bbox);

    let cols = ((max_x - min_x) / (hex_w * 0.75)).ceil() as i32 + 1;
    let rows = ((max_y - min_y) / hex_h).ceil() as i32 + 1;

    let mut result = Vec::new();
    for row in 0..rows {
        for col in 0..cols {
            let offset_x = if row % 2 == 0 { 0.0 } else { hex_w * 0.375 };
            let cx = min_x + col as f64 * hex_w * 0.75 + offset_x;
            let cy = min_y + row as f64 * hex_h * 0.5;
            result.push(make_hex_mercator(cx, cy, cell_m));
        }
    }
    result
}

fn mercator_x_range(bbox: &BBox) -> (f64, f64) {
    let p1 = geo_core::coords::wgs84_to_web_mercator(bbox.min_x, bbox.min_y);
    let p2 = geo_core::coords::wgs84_to_web_mercator(bbox.max_x, bbox.max_y);
    (p1.x.min(p2.x), p1.x.max(p2.x))
}

fn mercator_y_range(bbox: &BBox) -> (f64, f64) {
    let p1 = geo_core::coords::wgs84_to_web_mercator(bbox.min_x, bbox.min_y);
    let p2 = geo_core::coords::wgs84_to_web_mercator(bbox.max_x, bbox.max_y);
    (p1.y.min(p2.y), p1.y.max(p2.y))
}

fn make_hex_mercator(cx: f64, cy: f64, size_m: f64) -> Polygon {
    let mut coords = Vec::new();
    for i in 0..6 {
        let angle = PI / 180.0 * (60.0 * i as f64 - 30.0);
        let x = cx + size_m / 3.0_f64.sqrt() * angle.cos();
        let y = cy + size_m / 3.0_f64.sqrt() * angle.sin();
        let wgs = geo_core::coords::web_mercator_to_wgs84(x, y);
        coords.push(Point { x: wgs.x, y: wgs.y });
    }
    coords.push(coords[0]); // close ring
    Polygon { exterior: LineString { coords }, interiors: vec![] }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_hex_grid_beijing() {
        let bbox = BBox { min_x: 116.0, min_y: 39.5, max_x: 117.0, max_y: 40.5 };
        let grid = hex_grid(&bbox, 5000.0, Units::Meters);
        assert!(grid.len() > 0);
        // All hexes should be valid polygons
        for hex in &grid {
            assert!(hex.exterior.coords.len() >= 4);
            assert_eq!(hex.exterior.coords.first(), hex.exterior.coords.last());
        }
    }

    #[test]
    fn test_hex_grid_returns_valid_polygons() {
        let bbox = BBox { min_x: 0.0, min_y: 0.0, max_x: 0.01, max_y: 0.01 };
        let grid = hex_grid(&bbox, 100.0, Units::Meters);
        for hex in &grid {
            // Each hex has 6 sides + closing point = 7 coords
            assert!(hex.exterior.coords.len() == 7);
            assert!(hex.interiors.is_empty());
        }
    }
}
