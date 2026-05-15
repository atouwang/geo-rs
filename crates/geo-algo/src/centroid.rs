use geo_core::types::{Geometry, Point};
use geo::algorithm::centroid::Centroid as _;

pub fn centroid(geom: &Geometry) -> Option<Point> {
    let g: geo_types::Geometry = geom.into();
    g.centroid().map(|c| Point { x: c.x(), y: c.y() })
}
