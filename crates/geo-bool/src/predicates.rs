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
