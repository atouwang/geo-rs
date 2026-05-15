use geo::algorithm::line_interpolate_point::LineInterpolatePoint as _;
use geo::algorithm::closest_point::ClosestPoint as _;
use geo::algorithm::geodesic_bearing::GeodesicBearing as _;
use geo::algorithm::geodesic_destination::GeodesicDestination as _;
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
