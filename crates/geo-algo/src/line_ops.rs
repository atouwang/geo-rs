use geo::algorithm::closest_point::ClosestPoint as _;
use geo::algorithm::geodesic_bearing::GeodesicBearing as _;
use geo::algorithm::geodesic_destination::GeodesicDestination as _;
use geo::algorithm::line_interpolate_point::LineInterpolatePoint as _;
use geo::Closest;
use geo_core::types::*;

fn ls_to_geo(ls: &LineString) -> geo_types::LineString {
    ls.coords.iter().map(|p| geo_types::Coord { x: p.x, y: p.y }).collect::<Vec<_>>().into()
}

pub fn along(line: &LineString, distance: f64) -> Option<Point> {
    ls_to_geo(line).line_interpolate_point(distance).map(|p| Point { x: p.x(), y: p.y() })
}

pub fn bearing(p1: &Point, p2: &Point) -> f64 {
    geo_types::Point::new(p1.x, p1.y).geodesic_bearing(geo_types::Point::new(p2.x, p2.y))
}

pub fn destination(origin: &Point, distance: f64, bearing: f64) -> Point {
    let pt = geo_types::Point::new(origin.x, origin.y);
    let result = pt.geodesic_destination(bearing, distance);
    Point { x: result.x(), y: result.y() }
}

pub fn nearest_point_on_line(line: &LineString, point: &Point) -> Option<Point> {
    let ls = ls_to_geo(line);
    let gt_pt = geo_types::Point::new(point.x, point.y);
    match ls.closest_point(&gt_pt) {
        Closest::Intersection(pt) | Closest::SinglePoint(pt) => Some(Point { x: pt.x(), y: pt.y() }),
        Closest::Indeterminate => None,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_along() {
        let line = LineString { coords: vec![Point { x: 0.0, y: 0.0 }, Point { x: 1.0, y: 1.0 }] };
        let pt = along(&line, 0.0).unwrap();
        assert!((pt.x - 0.0).abs() < 0.01);
        assert!((pt.y - 0.0).abs() < 0.01);
    }

    #[test]
    fn test_bearing() {
        let beijing = Point { x: 116.397, y: 39.908 };
        let shanghai = Point { x: 121.473, y: 31.230 };
        let b = bearing(&beijing, &shanghai);
        assert!(b > 100.0 && b < 180.0); // roughly southeast
    }

    #[test]
    fn test_destination() {
        let origin = Point { x: 0.0, y: 0.0 };
        let dest = destination(&origin, 1000.0, 90.0);
        assert!(dest.x > 0.0);
        assert!(dest.y.abs() < 1.0); // near equator
    }

    #[test]
    fn test_nearest_point_on_line() {
        let line = LineString { coords: vec![Point { x: 0.0, y: 0.0 }, Point { x: 10.0, y: 0.0 }] };
        let pt = Point { x: 5.0, y: 1.0 };
        let nearest = nearest_point_on_line(&line, &pt).unwrap();
        assert!((nearest.x - 5.0).abs() < 0.01);
        assert!((nearest.y - 0.0).abs() < 0.01);
    }
}
