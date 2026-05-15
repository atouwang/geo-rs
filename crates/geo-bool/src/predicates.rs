use geo::algorithm::contains::Contains as _;
use geo::algorithm::intersects::Intersects as _;
use geo_core::types::Geometry;

pub fn contains(a: &Geometry, b: &Geometry) -> bool {
    let ga: geo_types::Geometry = a.into();
    let gb: geo_types::Geometry = b.into();
    ga.contains(&gb)
}

pub fn intersects(a: &Geometry, b: &Geometry) -> bool {
    let ga: geo_types::Geometry = a.into();
    let gb: geo_types::Geometry = b.into();
    ga.intersects(&gb)
}
