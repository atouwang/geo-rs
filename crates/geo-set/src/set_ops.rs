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

pub fn dissolve(geoms: &[Geometry]) -> Result<Geometry, GeoError> {
    if geoms.is_empty() {
        return Err(GeoError::InvalidGeometry("empty input".into()));
    }
    let mut result = to_geo_mp(&geoms[0])?;
    for geom in &geoms[1..] {
        result = result.union(&to_geo_mp(geom)?);
    }
    Ok(mp_to_ours(&result))
}

#[cfg(test)]
mod tests {
    use super::*;

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

    #[test]
    fn test_union_basic() {
        let result = union(&square(), &square()).unwrap();
        assert!(matches!(result, Geometry::MultiPolygon(_)));
    }

    #[test]
    fn test_dissolve_basic() {
        let result = dissolve(&[square(), square()]).unwrap();
        assert!(matches!(result, Geometry::MultiPolygon(_)));
    }
}
