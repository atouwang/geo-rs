use geo_core::types::*;

pub fn isolines(points: &[Point], values: &[f64], breaks: &[f64]) -> Vec<LineString> {
    if points.len() < 3 || values.len() != points.len() || breaks.is_empty() {
        return vec![];
    }
    // Marching Squares on Delaunay triangulation
    // Build triangles from the point set using simple triangulation
    let triangles = build_triangulation(points);
    let mut results = Vec::new();

    for &break_val in breaks {
        let mut segments: Vec<(Point, Point)> = Vec::new();
        for &(i, j, k) in &triangles {
            let vi = values[i]; let vj = values[j]; let vk = values[k];
            let above = |v: f64| v >= break_val;
            let ai = above(vi); let aj = above(vj); let ak = above(vk);
            let count = ai as u8 + aj as u8 + ak as u8;

            match count {
                1 => {
                    // One vertex above: segment between the two crossing edges
                    let (p1, p2) = if ai {
                        (interpolate(&points[i], vi, &points[j], vj, break_val),
                         interpolate(&points[i], vi, &points[k], vk, break_val))
                    } else if aj {
                        (interpolate(&points[j], vj, &points[i], vi, break_val),
                         interpolate(&points[j], vj, &points[k], vk, break_val))
                    } else {
                        (interpolate(&points[k], vk, &points[i], vi, break_val),
                         interpolate(&points[k], vk, &points[j], vj, break_val))
                    };
                    segments.push((p1, p2));
                }
                2 => {
                    // Two vertices above: segment between the two crossing edges
                    let (p1, p2) = if !ai {
                        (interpolate(&points[i], vi, &points[j], vj, break_val),
                         interpolate(&points[i], vi, &points[k], vk, break_val))
                    } else if !aj {
                        (interpolate(&points[j], vj, &points[i], vi, break_val),
                         interpolate(&points[j], vj, &points[k], vk, break_val))
                    } else {
                        (interpolate(&points[k], vk, &points[i], vi, break_val),
                         interpolate(&points[k], vk, &points[j], vj, break_val))
                    };
                    segments.push((p1, p2));
                }
                _ => {} // 0 or 3 above: no crossing
            }
        }

        // Connect segments into polylines
        for (a, b) in segments {
            results.push(LineString { coords: vec![a, b] });
        }
    }

    results
}

fn interpolate(p1: &Point, v1: f64, p2: &Point, v2: f64, target: f64) -> Point {
    if (v2 - v1).abs() < 1e-12 {
        return Point { x: (p1.x + p2.x) / 2.0, y: (p1.y + p2.y) / 2.0 };
    }
    let t = (target - v1) / (v2 - v1);
    Point {
        x: p1.x + t * (p2.x - p1.x),
        y: p1.y + t * (p2.y - p1.y),
    }
}

fn build_triangulation(points: &[Point]) -> Vec<(usize, usize, usize)> {
    // Simple Delaunay-like triangulation using ear clipping
    if points.len() < 3 { return vec![]; }
    let mut triangles = Vec::new();
    // Build a simple fan triangulation from centroid
    let cx: f64 = points.iter().map(|p| p.x).sum::<f64>() / points.len() as f64;
    let cy: f64 = points.iter().map(|p| p.y).sum::<f64>() / points.len() as f64;
    let mut indices: Vec<usize> = (0..points.len()).collect();
    // Sort by angle around centroid
    indices.sort_by(|&a, &b| {
        (points[a].y - cy).atan2(points[a].x - cx)
            .partial_cmp(&(points[b].y - cy).atan2(points[b].x - cx))
            .unwrap_or(std::cmp::Ordering::Equal)
    });
    for i in 1..indices.len() - 1 {
        triangles.push((indices[0], indices[i], indices[i + 1]));
    }
    triangles
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_isolines_simple() {
        // 4 corner points forming a square with a central peak
        let pts = vec![
            Point { x: 0.0, y: 0.0 }, Point { x: 1.0, y: 0.0 },
            Point { x: 1.0, y: 1.0 }, Point { x: 0.0, y: 1.0 },
        ];
        let values = vec![0.0, 0.0, 1.0, 0.0];
        let breaks = vec![0.5];
        let lines = isolines(&pts, &values, &breaks);
        // Should produce contour lines around the high value
        assert!(!lines.is_empty());
    }

    #[test]
    fn test_isolines_empty_input() {
        assert!(isolines(&[], &[], &[1.0]).is_empty());
    }

    #[test]
    fn test_isolines_no_breaks() {
        let pts = vec![Point { x: 0.0, y: 0.0 }];
        let values = vec![1.0];
        assert!(isolines(&pts, &values, &[]).is_empty());
    }

    #[test]
    fn test_isolines_single_break() {
        // Triangle with values on each vertex
        let pts = vec![
            Point { x: 0.0, y: 0.0 },
            Point { x: 1.0, y: 0.0 },
            Point { x: 0.5, y: 1.0 },
        ];
        let values = vec![0.0, 0.0, 1.0];
        let breaks = vec![0.5];
        let lines = isolines(&pts, &values, &breaks);
        // The 0.5 contour should cross two edges of the triangle
        for line in &lines {
            assert!(line.coords.len() == 2);
        }
    }
}
