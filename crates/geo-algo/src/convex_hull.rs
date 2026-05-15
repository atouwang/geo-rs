use geo_core::error::GeoError;
use geo_core::types::{Geometry, Polygon};
use geo::algorithm::convex_hull::ConvexHull as _;

pub fn convex_hull(geom: &Geometry) -> Result<Geometry, GeoError> {
    let g: geo_types::Geometry = geom.into();
    let hull: geo_types::Polygon = g.convex_hull();
    Ok(Geometry::Polygon(Polygon {
        exterior: geo_core::types::LineString {
            coords: hull.exterior().coords().map(|c| geo_core::types::Point { x: c.x, y: c.y }).collect(),
        },
        interiors: hull.interiors().iter().map(|ls| geo_core::types::LineString {
            coords: ls.coords().map(|c| geo_core::types::Point { x: c.x, y: c.y }).collect(),
        }).collect(),
    }))
}
