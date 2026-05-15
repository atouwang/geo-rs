use geo::algorithm::bool_ops::BooleanOps as _;
use geo_core::error::GeoError;
use geo_core::types::*;

fn to_geo_mp(geom: &Geometry) -> Result<geo_types::MultiPolygon, GeoError> {
    match geom {
        Geometry::Polygon(p) => Ok(geo_types::MultiPolygon::new(vec![poly_to_geo(p)])),
        Geometry::MultiPolygon(mp) => Ok(mp_to_geo(mp)),
        _ => Err(GeoError::OperationNotSupported {
            op: "set operation".into(),
            reason: "only Polygon and MultiPolygon supported".into(),
        }),
    }
}

fn poly_to_geo(p: &Polygon) -> geo_types::Polygon {
    geo_types::Polygon::new(
        p.exterior.coords.iter().map(|pt| geo_types::Coord { x: pt.x, y: pt.y }).collect::<Vec<_>>().into(),
        p.interiors.iter().map(|i| i.coords.iter().map(|pt| geo_types::Coord { x: pt.x, y: pt.y }).collect::<Vec<_>>().into()).collect(),
    )
}

fn mp_to_geo(mp: &MultiPolygon) -> geo_types::MultiPolygon {
    geo_types::MultiPolygon::new(mp.polygons.iter().map(|p| poly_to_geo(p)).collect())
}

fn mp_to_ours(mp: &geo_types::MultiPolygon) -> Geometry {
    Geometry::MultiPolygon(MultiPolygon {
        polygons: mp.iter().map(|p| Polygon {
            exterior: LineString { coords: p.exterior().coords().map(|c| Point { x: c.x, y: c.y }).collect() },
            interiors: p.interiors().iter().map(|ls| LineString { coords: ls.coords().map(|c| Point { x: c.x, y: c.y }).collect() }).collect(),
        }).collect(),
    })
}

pub fn union(a: &Geometry, b: &Geometry) -> Result<Geometry, GeoError> {
    Ok(mp_to_ours(&to_geo_mp(a)?.union(&to_geo_mp(b)?)))
}

pub fn intersect(a: &Geometry, b: &Geometry) -> Result<Geometry, GeoError> {
    Ok(mp_to_ours(&to_geo_mp(a)?.intersection(&to_geo_mp(b)?)))
}

pub fn difference(a: &Geometry, b: &Geometry) -> Result<Geometry, GeoError> {
    Ok(mp_to_ours(&to_geo_mp(a)?.difference(&to_geo_mp(b)?)))
}

pub fn xor(a: &Geometry, b: &Geometry) -> Result<Geometry, GeoError> {
    Ok(mp_to_ours(&to_geo_mp(a)?.xor(&to_geo_mp(b)?)))
}
