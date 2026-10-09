use geo_core::error::GeoError;
use geo_core::types::*;
use geo_index::rtree::RTree;
use std::collections::BTreeMap;

/// Sampled Voronoi approximation restricted to bbox. Output is ordered by
/// owning input index; unsampled/small regions may be omitted. Not exact cells.
pub fn voronoi(points: &[Point], bbox: &BBox) -> Result<Vec<Polygon>, GeoError> {
    let invalid = |message: &str| GeoError::InvalidGeometry(message.into());
    if ![bbox.min_x, bbox.min_y, bbox.max_x, bbox.max_y].iter().all(|v| v.is_finite())
        || bbox.min_x >= bbox.max_x
        || bbox.min_y >= bbox.max_y
    {
        return Err(invalid("voronoi requires finite ordered bbox bounds"));
    }
    if points.len() > 10_000 {
        return Err(invalid("voronoi exceeds the 10000-point limit"));
    }
    if points.iter().any(|p| !p.x.is_finite() || !p.y.is_finite()) {
        return Err(invalid("voronoi coordinates must be finite"));
    }
    if points.len() < 3 {
        return Ok(vec![]);
    }

    let bx = bbox.min_x;
    let by = bbox.min_y;
    let bw = bbox.max_x - bbox.min_x;
    let bh = bbox.max_y - bbox.min_y;
    let scale = bw.max(bh);
    if !scale.is_finite() {
        return Err(invalid("voronoi bbox range overflows"));
    }

    // Simple grid-based Voronoi approximation:
    // For each cell in a fine grid, assign to nearest point
    let resolution = (points.len() as f64 * 20.0).sqrt().max(20.0) as usize;
    let cell_w = bw / resolution as f64;
    let cell_h = bh / resolution as f64;
    if bx + cell_w == bx || by + cell_h == by {
        return Err(invalid("voronoi bbox has insufficient numeric precision for sampling"));
    }
    let mut tree = RTree::new();
    for (id, p) in points.iter().enumerate() {
        let normalized = Point { x: (p.x - bx) / scale, y: (p.y - by) / scale };
        if !normalized.x.is_finite()
            || !normalized.y.is_finite()
            || normalized.x.abs() > 1e100
            || normalized.y.abs() > 1e100
        {
            return Err(invalid("voronoi point range exceeds supported sampling precision"));
        }
        tree.insert_point(&normalized, id as u64);
    }

    // Map: point_index -> list of cells
    let mut cells: BTreeMap<usize, Vec<(usize, usize)>> = BTreeMap::new();
    for i in 0..resolution {
        for j in 0..resolution {
            let sample = Point {
                x: (i as f64 + 0.5) * (bw / scale) / resolution as f64,
                y: (j as f64 + 0.5) * (bh / scale) / resolution as f64,
            };
            let (min_idx, _) = tree.nearest(&sample).ok_or_else(|| invalid("voronoi has no indexed points"))?;
            let min_idx = min_idx as usize;
            cells.entry(min_idx).or_default().push((i, j));
        }
    }

    // Convert cell regions to approximate polygons
    // Order boundary sample centers angularly; this is not exact tessellation.
    let mut results = Vec::new();
    for (_, cell_list) in cells {
        if cell_list.len() < 3 {
            continue;
        }

        // Collect boundary cells (cells that neighbor another region)
        let cell_set: std::collections::HashSet<(usize, usize)> = cell_list.iter().copied().collect();
        let mut boundary: Vec<(usize, usize)> = Vec::new();

        for &(ci, cj) in &cell_list {
            let neighbors = [(ci.wrapping_sub(1), cj), (ci + 1, cj), (ci, cj.wrapping_sub(1)), (ci, cj + 1)];
            let is_boundary = neighbors.iter().any(|n| !cell_set.contains(n));
            if is_boundary {
                boundary.push((ci, cj));
            }
        }

        if boundary.len() < 3 {
            continue;
        }

        // Sort boundary points by angle around centroid
        let cx: f64 = boundary.iter().map(|p| p.0 as f64).sum::<f64>() / boundary.len() as f64;
        let cy: f64 = boundary.iter().map(|p| p.1 as f64).sum::<f64>() / boundary.len() as f64;
        let angle = |p: &(usize, usize)| ((p.1 as f64 - cy) * cell_h).atan2((p.0 as f64 - cx) * cell_w);
        boundary.sort_by(|a, b| angle(a).total_cmp(&angle(b)));
        let mut boundary: Vec<Point> = boundary
            .into_iter()
            .map(|(i, j)| Point {
                x: (bx + (i as f64 + 0.5) * cell_w).clamp(bbox.min_x, bbox.max_x),
                y: (by + (j as f64 + 0.5) * cell_h).clamp(bbox.min_y, bbox.max_y),
            })
            .collect();

        boundary.push(boundary[0]); // close
        results.push(Polygon { exterior: LineString { coords: boundary }, interiors: vec![] });
    }

    Ok(results)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn rejects_invalid_bounds_points_and_excessive_samples() {
        let bbox = BBox { min_x: 0.0, min_y: 0.0, max_x: 10.0, max_y: 10.0 };
        let points = [Point { x: 1.0, y: 1.0 }; 3];
        for invalid in [BBox { max_x: 0.0, ..bbox }, BBox { max_y: f64::INFINITY, ..bbox }] {
            assert!(voronoi(&points, &invalid).is_err());
        }
        assert!(voronoi(&[Point { x: f64::NAN, y: 0.0 }; 3], &bbox).is_err());
        assert!(voronoi(&vec![Point { x: 1.0, y: 1.0 }; 10001], &bbox).is_err());
        assert!(voronoi(&[Point { x: f64::MAX, y: 0.0 }; 3], &bbox).is_err());
    }

    #[test]
    fn boundary_samples_belong_to_the_brute_force_nearest_owner() {
        let points = [Point { x: -1.0, y: -1.0 }, Point { x: 4.0, y: -1.0 }, Point { x: 2.0, y: 4.0 }];
        let bbox = BBox { min_x: -2.0, min_y: -2.0, max_x: 5.0, max_y: 5.0 };
        let cells = voronoi(&points, &bbox).unwrap();
        assert_eq!(cells.len(), 3);
        for (index, cell) in cells.iter().enumerate() {
            for sample in &cell.exterior.coords {
                let distance = |p: &Point| (sample.x - p.x).powi(2) + (sample.y - p.y).powi(2);
                let expected = points.iter().map(distance).fold(f64::INFINITY, f64::min);
                assert!((distance(&points[index]) - expected).abs() < 1e-10);
            }
        }
    }

    #[test]
    fn output_respects_bbox_and_is_repeatable() {
        let pts = vec![Point { x: 0.0, y: 0.0 }, Point { x: 5.0, y: 0.0 }, Point { x: 2.5, y: 5.0 }];
        let bbox = BBox { min_x: -1.0, min_y: -1.0, max_x: 6.0, max_y: 6.0 };
        let first = voronoi(&pts, &bbox).unwrap();
        for poly in &first {
            for p in &poly.exterior.coords {
                assert!(p.x >= bbox.min_x && p.x <= bbox.max_x && p.y >= bbox.min_y && p.y <= bbox.max_y);
            }
        }
        assert_eq!(first, voronoi(&pts, &bbox).unwrap());
    }

    #[test]
    fn test_voronoi_too_few_points() {
        let bbox = BBox { min_x: 0.0, min_y: 0.0, max_x: 10.0, max_y: 10.0 };
        assert!(voronoi(&[], &bbox).unwrap().is_empty());
        assert!(voronoi(&[Point { x: 1.0, y: 1.0 }], &bbox).unwrap().is_empty());
    }

    #[test]
    fn test_voronoi_three_points() {
        let pts = vec![Point { x: 0.0, y: 0.0 }, Point { x: 5.0, y: 0.0 }, Point { x: 2.5, y: 5.0 }];
        let bbox = BBox { min_x: -1.0, min_y: -1.0, max_x: 6.0, max_y: 6.0 };
        let cells = voronoi(&pts, &bbox).unwrap();
        // 3 input points should produce up to 3 Voronoi cells
        assert!(!cells.is_empty() && cells.len() <= 3);
        for cell in &cells {
            assert!(cell.exterior.coords.len() >= 4);
        }
    }
}
