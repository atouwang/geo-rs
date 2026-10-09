use geo_core::error::GeoError;
use geo_core::types::*;

const MAX_POINTS: usize = 10_000;
const MAX_TRIANGLE_LEVELS: usize = 100_000;

/// Approximate marching-triangle segments over an angularly sorted fan.
/// Segments are disconnected; this is not a Delaunay contour reconstruction.
/// Rejects non-finite/mismatched input, over 10,000 points, or over 100,000
/// triangle-level evaluations before triangulation/output allocation.
pub fn isolines(points: &[Point], values: &[f64], breaks: &[f64]) -> Result<Vec<LineString>, GeoError> {
    let invalid = |message: &str| GeoError::InvalidGeometry(message.into());
    if values.len() != points.len() {
        return Err(invalid("isolines values must match the point count"));
    }
    if points.len() > MAX_POINTS {
        return Err(invalid("isolines exceeds the 10000-point limit"));
    }
    if points.iter().any(|p| !p.x.is_finite() || !p.y.is_finite())
        || values.iter().chain(breaks).any(|v| !v.is_finite())
    {
        return Err(invalid("isolines coordinates, values and breaks must be finite"));
    }
    if points.len().saturating_sub(2).checked_mul(breaks.len()).is_none_or(|work| work > MAX_TRIANGLE_LEVELS) {
        return Err(invalid("isolines exceeds the 100000 triangle-level limit"));
    }
    if points.len() < 3 || breaks.is_empty() {
        return Ok(vec![]);
    }
    // Keep the existing approximate fan; validate before building it.
    let triangles = build_triangulation(points);
    let mut results = Vec::new();

    for &break_val in breaks {
        for &(i, j, k) in &triangles {
            let vi = values[i];
            let vj = values[j];
            let vk = values[k];
            let above = |v: f64| v >= break_val;
            let ai = above(vi);
            let aj = above(vj);
            let ak = above(vk);
            let count = ai as u8 + aj as u8 + ak as u8;

            match count {
                1 => {
                    // One vertex above: segment between the two crossing edges
                    let (p1, p2) = if ai {
                        (
                            interpolate(&points[i], vi, &points[j], vj, break_val),
                            interpolate(&points[i], vi, &points[k], vk, break_val),
                        )
                    } else if aj {
                        (
                            interpolate(&points[j], vj, &points[i], vi, break_val),
                            interpolate(&points[j], vj, &points[k], vk, break_val),
                        )
                    } else {
                        (
                            interpolate(&points[k], vk, &points[i], vi, break_val),
                            interpolate(&points[k], vk, &points[j], vj, break_val),
                        )
                    };
                    results.push(LineString { coords: vec![p1, p2] });
                }
                2 => {
                    // Two vertices above: segment between the two crossing edges
                    let (p1, p2) = if !ai {
                        (
                            interpolate(&points[i], vi, &points[j], vj, break_val),
                            interpolate(&points[i], vi, &points[k], vk, break_val),
                        )
                    } else if !aj {
                        (
                            interpolate(&points[j], vj, &points[i], vi, break_val),
                            interpolate(&points[j], vj, &points[k], vk, break_val),
                        )
                    } else {
                        (
                            interpolate(&points[k], vk, &points[i], vi, break_val),
                            interpolate(&points[k], vk, &points[j], vj, break_val),
                        )
                    };
                    results.push(LineString { coords: vec![p1, p2] });
                }
                _ => {} // 0 or 3 above: no crossing
            }
        }
    }

    Ok(results)
}

fn interpolate(p1: &Point, v1: f64, p2: &Point, v2: f64, target: f64) -> Point {
    // Crossing edges have distinct finite values. Preserve tiny ranges, and
    // halve first when subtraction of opposite extreme values overflows.
    let range = v2 - v1;
    let t = if range.is_finite() { (target - v1) / range } else { (target / 2.0 - v1 / 2.0) / (v2 / 2.0 - v1 / 2.0) }
        .clamp(0.0, 1.0);
    let lerp = |a: f64, b: f64| {
        // Convex weights avoid overflowing b-a; clamp rounding at endpoints.
        ((1.0 - t) * a + t * b).clamp(a.min(b), a.max(b))
    };
    Point { x: lerp(p1.x, p2.x), y: lerp(p1.y, p2.y) }
}

fn build_triangulation(points: &[Point]) -> Vec<(usize, usize, usize)> {
    // Angularly sorted fan triangulation; not Delaunay or ear clipping.
    if points.len() < 3 {
        return vec![];
    }
    let mut triangles = Vec::new();
    // Uniformly scale before summing/angle differences to avoid finite input
    // overflowing the centroid or subtraction. Scaling preserves angles.
    let scale = points.iter().fold(0.0_f64, |s, p| s.max(p.x.abs()).max(p.y.abs()));
    let scale = if scale == 0.0 { 1.0 } else { scale };
    let cx = points.iter().map(|p| p.x / scale).sum::<f64>() / points.len() as f64;
    let cy = points.iter().map(|p| p.y / scale).sum::<f64>() / points.len() as f64;
    let mut indices: Vec<usize> = (0..points.len()).collect();
    // Sort by angle around centroid
    indices.sort_by(|&a, &b| {
        (points[a].y / scale - cy)
            .atan2(points[a].x / scale - cx)
            .partial_cmp(&(points[b].y / scale - cy).atan2(points[b].x / scale - cx))
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
            Point { x: 0.0, y: 0.0 },
            Point { x: 1.0, y: 0.0 },
            Point { x: 1.0, y: 1.0 },
            Point { x: 0.0, y: 1.0 },
        ];
        let values = vec![0.0, 0.0, 1.0, 0.0];
        let breaks = vec![0.5];
        let lines = isolines(&pts, &values, &breaks).unwrap();
        // Should produce contour lines around the high value
        assert!(!lines.is_empty());
    }

    #[test]
    fn test_isolines_empty_input() {
        assert!(isolines(&[], &[], &[1.0]).unwrap().is_empty());
    }

    #[test]
    fn test_isolines_no_breaks() {
        let pts = vec![Point { x: 0.0, y: 0.0 }];
        let values = vec![1.0];
        assert!(isolines(&pts, &values, &[]).unwrap().is_empty());
    }

    #[test]
    fn test_isolines_single_break() {
        // Triangle with values on each vertex
        let pts = vec![Point { x: 0.0, y: 0.0 }, Point { x: 1.0, y: 0.0 }, Point { x: 0.5, y: 1.0 }];
        let values = vec![0.0, 0.0, 1.0];
        let breaks = vec![0.5];
        let lines = isolines(&pts, &values, &breaks).unwrap();
        // The 0.5 contour should cross two edges of the triangle
        for line in &lines {
            assert!(line.coords.len() == 2);
        }
    }
    #[test]
    fn small_value_range_interpolates_at_the_requested_level() {
        let points = [Point { x: 0.0, y: 0.0 }, Point { x: 1.0, y: 0.0 }, Point { x: 0.0, y: 1.0 }];
        let lines = isolines(&points, &[0.0, 1e-13, 1e-13], &[2.5e-14]).unwrap();
        assert_eq!(lines.len(), 1);
        for point in &lines[0].coords {
            assert!((point.x + point.y - 0.25).abs() < 1e-12);
        }
    }

    #[test]
    fn extreme_finite_values_produce_finite_crossings() {
        let points = [Point { x: -f64::MAX, y: 0.0 }, Point { x: f64::MAX, y: 0.0 }, Point { x: -f64::MAX, y: 1.0 }];
        let lines = isolines(&points, &[-f64::MAX, f64::MAX, -f64::MAX], &[0.0]).unwrap();
        assert_eq!(lines.len(), 1);
        for point in &lines[0].coords {
            assert!(point.x.is_finite() && point.y.is_finite());
            assert_eq!(point.x, 0.0);
        }
    }

    #[test]
    fn rejects_invalid_lengths_nonfinite_input_and_work_limits() {
        let points = [Point { x: 0.0, y: 0.0 }, Point { x: 1.0, y: 0.0 }, Point { x: 0.0, y: 1.0 }];
        assert!(isolines(&points, &[0.0, 1.0], &[0.5]).is_err());
        for invalid in [f64::NAN, f64::INFINITY, f64::NEG_INFINITY] {
            assert!(isolines(&points, &[0.0, invalid, 1.0], &[0.5]).is_err());
            assert!(isolines(&points, &[0.0, 1.0, 1.0], &[invalid]).is_err());
            let mut invalid_points = points;
            invalid_points[0].x = invalid;
            assert!(isolines(&invalid_points, &[0.0, 1.0, 1.0], &[0.5]).is_err());
        }
        assert!(isolines(&vec![points[0]; MAX_POINTS + 1], &vec![0.0; MAX_POINTS + 1], &[]).is_err());
        assert!(isolines(&points, &[0.0; 3], &vec![1.0; MAX_TRIANGLE_LEVELS + 1]).is_err());
        // Two triangles exceed the limit even with fewer than 100,000 breaks.
        assert!(isolines(&[points[0]; 4], &[0.0; 4], &vec![1.0; MAX_TRIANGLE_LEVELS / 2 + 1]).is_err());
        assert!(isolines(&points, &[0.0; 3], &vec![1.0; MAX_TRIANGLE_LEVELS]).unwrap().is_empty());
        assert!(isolines(&vec![points[0]; MAX_POINTS], &vec![0.0; MAX_POINTS], &[]).unwrap().is_empty());
    }
}
