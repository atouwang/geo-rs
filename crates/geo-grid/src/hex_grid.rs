use geo_core::{error::GeoError, types::*};
use std::f64::consts::PI;

/// Full pointy-top hexagons covering a WGS84 bbox. Side lengths are measured
/// in Web Mercator, not on the earth's surface; border cells extend past bbox.
/// Rejects invalid inputs and grids exceeding 100,000 cells before allocation.
pub fn hex_grid(bbox: &BBox, cell_side: f64, units: Units) -> Result<Vec<Polygon>, GeoError> {
    if !cell_side.is_finite()
        || cell_side <= 0.0
        || ![bbox.min_x, bbox.min_y, bbox.max_x, bbox.max_y].iter().all(|v| v.is_finite())
        || bbox.min_x >= bbox.max_x
        || bbox.min_y >= bbox.max_y
        || bbox.min_x < -180.0
        || bbox.max_x > 180.0
        || bbox.min_y < -85.05112878
        || bbox.max_y > 85.05112878
    {
        return Err(GeoError::InvalidGeometry(
            "hex grid requires positive finite size and an ordered Web Mercator-compatible WGS84 bbox".into(),
        ));
    }
    let side = match units {
        Units::Meters => cell_side,
        Units::Kilometers => cell_side * 1000.0,
        Units::Miles => cell_side * 1609.344,
        Units::Degrees => cell_side * 111_320.0,
    };
    let width = 3.0_f64.sqrt() * side;
    let row_step = 1.5 * side;
    if !width.is_finite() || !row_step.is_finite() || width == 0.0 || row_step == 0.0 {
        return Err(GeoError::InvalidGeometry("hex grid size is outside the supported numeric range".into()));
    }
    let min = geo_core::coords::wgs84_to_web_mercator(bbox.min_x, bbox.min_y);
    let max = geo_core::coords::wgs84_to_web_mercator(bbox.max_x, bbox.max_y);
    let cols = ((max.x - min.x) / width).ceil() + 3.0;
    let rows = ((max.y - min.y) / row_step).ceil() + 3.0;
    if !cols.is_finite() || !rows.is_finite() || cols * rows > 100_000.0 {
        return Err(GeoError::InvalidGeometry("hex grid exceeds the 100000-cell limit".into()));
    }
    let cols = cols as usize;
    let rows = rows as usize;
    let mut result = Vec::with_capacity(cols * rows);
    for row in 0..rows {
        for col in 0..cols {
            let offset = if row % 2 == 0 { 0.0 } else { width * 0.5 };
            let cx = min.x + (col as f64 - 1.0) * width + offset;
            let cy = min.y + (row as f64 - 1.0) * row_step;
            let mut coords = Vec::with_capacity(7);
            for i in 0..6 {
                let angle = PI / 180.0 * (60.0 * i as f64 - 30.0);
                coords.push(geo_core::coords::web_mercator_to_wgs84(cx + side * angle.cos(), cy + side * angle.sin()));
            }
            coords.push(coords[0]);
            result.push(Polygon { exterior: LineString { coords }, interiors: vec![] });
        }
    }
    Ok(result)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn rejects_invalid_sizes_bboxes_and_excessive_output() {
        let bbox = BBox { min_x: 0.0, min_y: 0.0, max_x: 0.01, max_y: 0.01 };
        for side in [0.0, -1.0, f64::NAN, f64::INFINITY, 1e-9, f64::MAX] {
            assert!(hex_grid(&bbox, side, Units::Meters).is_err());
        }
        for invalid in [BBox { max_x: -1.0, ..bbox }, BBox { min_y: -90.0, ..bbox }, BBox { min_x: f64::NAN, ..bbox }] {
            assert!(hex_grid(&invalid, 100.0, Units::Meters).is_err());
        }
    }

    #[test]
    fn miles_and_kilometers_match_meter_sizes() {
        let bbox = BBox { min_x: 0.0, min_y: 0.0, max_x: 0.01, max_y: 0.01 };
        assert_eq!(hex_grid(&bbox, 1.0, Units::Miles).unwrap(), hex_grid(&bbox, 1609.344, Units::Meters).unwrap());
        assert_eq!(hex_grid(&bbox, 0.1, Units::Kilometers).unwrap(), hex_grid(&bbox, 100.0, Units::Meters).unwrap());
    }

    #[test]
    fn test_hex_grid_beijing() {
        let bbox = BBox { min_x: 116.0, min_y: 39.5, max_x: 117.0, max_y: 40.5 };
        let grid = hex_grid(&bbox, 5000.0, Units::Meters).unwrap();
        assert!(!grid.is_empty());
        // All hexes should be valid polygons
        for hex in &grid {
            assert!(hex.exterior.coords.len() >= 4);
            assert_eq!(hex.exterior.coords.first(), hex.exterior.coords.last());
        }
    }

    #[test]
    fn test_hex_grid_returns_valid_polygons() {
        let bbox = BBox { min_x: 0.0, min_y: 0.0, max_x: 0.01, max_y: 0.01 };
        let grid = hex_grid(&bbox, 100.0, Units::Meters).unwrap();
        for hex in &grid {
            // Each hex has 6 sides + closing point = 7 coords
            assert!(hex.exterior.coords.len() == 7);
            assert!(hex.interiors.is_empty());
        }
    }

    #[test]
    fn projected_side_length_and_full_coverage() {
        let bbox = BBox { min_x: 0.0, min_y: 0.0, max_x: 0.01, max_y: 0.01 };
        let grid = hex_grid(&bbox, 100.0, Units::Meters).unwrap();
        for hex in &grid {
            for edge in hex.exterior.coords.windows(2) {
                let a = geo_core::coords::wgs84_to_web_mercator(edge[0].x, edge[0].y);
                let b = geo_core::coords::wgs84_to_web_mercator(edge[1].x, edge[1].y);
                assert!(((a.x - b.x).hypot(a.y - b.y) - 100.0).abs() < 1e-6);
            }
        }
        use geo::Contains;
        let polygons: Vec<geo_types::Geometry> = grid.iter().map(|p| (&Geometry::Polygon(p.clone())).into()).collect();
        for i in 0..=20 {
            for j in 0..=20 {
                let p = geo_types::Point::new(0.000013 + i as f64 * 0.000499, 0.000017 + j as f64 * 0.000499);
                assert!(polygons.iter().any(|poly| poly.contains(&geo_types::Geometry::Point(p))), "uncovered {p:?}");
            }
        }
    }
}
