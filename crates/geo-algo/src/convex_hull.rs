use geo::algorithm::convex_hull::ConvexHull as _;
use geo_core::error::GeoError;
use geo_core::types::{Geometry, Polygon};

pub fn convex_hull(geom: &Geometry) -> Result<Geometry, GeoError> {
    let g: geo_types::Geometry = geom.into();
    let hull: geo_types::Polygon = g.convex_hull();
    Ok(Geometry::Polygon(Polygon {
        exterior: geo_core::types::LineString {
            coords: hull.exterior().coords().map(|c| geo_core::types::Point { x: c.x, y: c.y }).collect(),
        },
        interiors: hull
            .interiors()
            .iter()
            .map(|ls| geo_core::types::LineString {
                coords: ls.coords().map(|c| geo_core::types::Point { x: c.x, y: c.y }).collect(),
            })
            .collect(),
    }))
}

#[cfg(test)]
mod tests {
    use super::*;
    use geo_core::types::*;

    #[test]
    fn test_convex_hull_triangle() {
        let tri = Geometry::Polygon(Polygon {
            exterior: LineString {
                coords: vec![
                    Point { x: 0.0, y: 0.0 },
                    Point { x: 2.0, y: 0.0 },
                    Point { x: 1.0, y: 2.0 },
                    Point { x: 0.0, y: 0.0 },
                ],
            },
            interiors: vec![],
        });
        let hull = convex_hull(&tri).unwrap();
        assert!(matches!(hull, Geometry::Polygon(_)));
    }

    #[test]
    fn test_convex_hull_multipoint() {
        let pts = Geometry::MultiPoint(MultiPoint {
            points: vec![
                Point { x: 0.0, y: 0.0 },
                Point { x: 1.0, y: 0.0 },
                Point { x: 2.0, y: 0.0 },
                Point { x: 0.5, y: 0.5 },
            ],
        });
        let hull = convex_hull(&pts).unwrap();
        assert!(matches!(hull, Geometry::Polygon(_)));
    }
}
