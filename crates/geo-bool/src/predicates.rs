use geo::algorithm::relate::Relate as _;
use geo_core::types::Geometry;

fn relate(a: &Geometry, b: &Geometry) -> geo::algorithm::relate::IntersectionMatrix {
    let ga: geo_types::Geometry = a.into();
    let gb: geo_types::Geometry = b.into();
    ga.relate(&gb)
}

pub fn contains(a: &Geometry, b: &Geometry) -> bool { relate(a, b).is_contains() }
pub fn intersects(a: &Geometry, b: &Geometry) -> bool { relate(a, b).is_intersects() }
pub fn within(a: &Geometry, b: &Geometry) -> bool { relate(a, b).is_within() }
pub fn crosses(a: &Geometry, b: &Geometry) -> bool { relate(a, b).is_crosses() }
pub fn disjoint(a: &Geometry, b: &Geometry) -> bool { relate(a, b).is_disjoint() }
pub fn overlaps(a: &Geometry, b: &Geometry) -> bool { relate(a, b).is_overlaps() }
pub fn touches(a: &Geometry, b: &Geometry) -> bool { relate(a, b).is_touches() }
pub fn equals(a: &Geometry, b: &Geometry) -> bool { relate(a, b).is_equal_topo() }

#[cfg(test)]
mod tests {
    use super::*;
    use geo_core::types::*;

    fn square() -> Geometry {
        Geometry::Polygon(Polygon {
            exterior: LineString {
                coords: vec![
                    Point { x: 0.0, y: 0.0 }, Point { x: 1.0, y: 0.0 },
                    Point { x: 1.0, y: 1.0 }, Point { x: 0.0, y: 1.0 },
                    Point { x: 0.0, y: 0.0 },
                ],
            },
            interiors: vec![],
        })
    }

    fn point_inside() -> Geometry { Geometry::Point(Point { x: 0.5, y: 0.5 }) }
    fn point_outside() -> Geometry { Geometry::Point(Point { x: 2.0, y: 2.0 }) }

    #[test] fn test_contains_inside() { assert!(contains(&square(), &point_inside())); }
    #[test] fn test_contains_outside() { assert!(!contains(&square(), &point_outside())); }
    #[test] fn test_intersects() { assert!(intersects(&square(), &point_inside())); }
    #[test] fn test_disjoint() { assert!(disjoint(&square(), &point_outside())); }
    #[test] fn test_within() { assert!(within(&point_inside(), &square())); }
    #[test] fn test_equals() { assert!(equals(&square(), &square())); }

    #[test] fn test_touches() {
        let sq1 = Geometry::Polygon(Polygon {
            exterior: LineString {
                coords: vec![
                    Point { x: 0.0, y: 0.0 }, Point { x: 1.0, y: 0.0 },
                    Point { x: 1.0, y: 1.0 }, Point { x: 0.0, y: 1.0 },
                    Point { x: 0.0, y: 0.0 },
                ],
            },
            interiors: vec![],
        });
        let sq2 = Geometry::Polygon(Polygon {
            exterior: LineString {
                coords: vec![
                    Point { x: 1.0, y: 0.0 }, Point { x: 2.0, y: 0.0 },
                    Point { x: 2.0, y: 1.0 }, Point { x: 1.0, y: 1.0 },
                    Point { x: 1.0, y: 0.0 },
                ],
            },
            interiors: vec![],
        });
        assert!(touches(&sq1, &sq2));
    }

    #[test] fn test_crosses() {
        let line = Geometry::LineString(LineString {
            coords: vec![Point { x: -1.0, y: 0.5 }, Point { x: 2.0, y: 0.5 }],
        });
        let sq = Geometry::Polygon(Polygon {
            exterior: LineString {
                coords: vec![
                    Point { x: 0.0, y: 0.0 }, Point { x: 1.0, y: 0.0 },
                    Point { x: 1.0, y: 1.0 }, Point { x: 0.0, y: 1.0 },
                    Point { x: 0.0, y: 0.0 },
                ],
            },
            interiors: vec![],
        });
        assert!(crosses(&line, &sq));
    }
}
