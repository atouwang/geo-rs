use geo_core::types::*;
use std::collections::HashMap;

pub fn voronoi(points: &[Point], bbox: &BBox) -> Vec<Polygon> {
    if points.len() < 3 { return vec![]; }

    let margin = (bbox.max_x - bbox.min_x).max(bbox.max_y - bbox.min_y) * 0.5;
    let bx = bbox.min_x - margin;
    let by = bbox.min_y - margin;
    let bw = bbox.max_x - bbox.min_x + 2.0 * margin;
    let bh = bbox.max_y - bbox.min_y + 2.0 * margin;

    // Simple grid-based Voronoi approximation:
    // For each cell in a fine grid, assign to nearest point
    let resolution = (points.len() as f64 * 20.0).sqrt().max(20.0) as usize;
    let cell_w = bw / resolution as f64;
    let cell_h = bh / resolution as f64;

    // Map: point_index -> list of cells
    let mut cells: HashMap<usize, Vec<(usize, usize)>> = HashMap::new();
    for i in 0..resolution {
        for j in 0..resolution {
            let cx = bx + (i as f64 + 0.5) * cell_w;
            let cy = by + (j as f64 + 0.5) * cell_h;
            // Find nearest point
            let mut min_dist = f64::MAX;
            let mut min_idx = 0;
            for (idx, pt) in points.iter().enumerate() {
                let dx = cx - pt.x;
                let dy = cy - pt.y;
                let d = dx * dx + dy * dy;
                if d < min_dist { min_dist = d; min_idx = idx; }
            }
            cells.entry(min_idx).or_default().push((i, j));
        }
    }

    // Convert cell regions to approximate polygons
    // For each point, compute convex hull of its cell corners
    let mut results = Vec::new();
    for (_, cell_list) in cells {
        if cell_list.len() < 3 { continue; }
        
        // Collect boundary cells (cells that neighbor another region)
        let cell_set: std::collections::HashSet<(usize, usize)> = cell_list.iter().copied().collect();
        let mut boundary: Vec<Point> = Vec::new();
        
        for &(ci, cj) in &cell_list {
            let neighbors = [
                (ci.wrapping_sub(1), cj), (ci + 1, cj),
                (ci, cj.wrapping_sub(1)), (ci, cj + 1),
            ];
            let is_boundary = neighbors.iter().any(|n| !cell_set.contains(n));
            if is_boundary {
                boundary.push(Point {
                    x: bx + (ci as f64 + 0.5) * cell_w,
                    y: by + (cj as f64 + 0.5) * cell_h,
                });
            }
        }
        
        if boundary.len() < 3 { continue; }
        
        // Sort boundary points by angle around centroid
        let cx: f64 = boundary.iter().map(|p| p.x).sum::<f64>() / boundary.len() as f64;
        let cy: f64 = boundary.iter().map(|p| p.y).sum::<f64>() / boundary.len() as f64;
        boundary.sort_by(|a, b| {
            (a.y - cy).atan2(a.x - cx).partial_cmp(&(b.y - cy).atan2(b.x - cx)).unwrap()
        });
        
        boundary.push(boundary[0]); // close
        results.push(Polygon {
            exterior: LineString { coords: boundary },
            interiors: vec![],
        });
    }

    results
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_voronoi_too_few_points() {
        let bbox = BBox { min_x: 0.0, min_y: 0.0, max_x: 10.0, max_y: 10.0 };
        assert!(voronoi(&[], &bbox).is_empty());
        assert!(voronoi(&[Point { x: 1.0, y: 1.0 }], &bbox).is_empty());
    }

    #[test]
    fn test_voronoi_three_points() {
        let pts = vec![
            Point { x: 0.0, y: 0.0 },
            Point { x: 5.0, y: 0.0 },
            Point { x: 2.5, y: 5.0 },
        ];
        let bbox = BBox { min_x: -1.0, min_y: -1.0, max_x: 6.0, max_y: 6.0 };
        let cells = voronoi(&pts, &bbox);
        // 3 input points should produce up to 3 Voronoi cells
        assert!(cells.len() > 0 && cells.len() <= 3);
        for cell in &cells {
            assert!(cell.exterior.coords.len() >= 4);
        }
    }
}
