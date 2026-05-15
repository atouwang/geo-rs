use geo_core::types::{Geometry, Point};

pub fn centroid(geom: &Geometry) -> Option<Point> {
    geo_core::measure::centroid(geom)
}
