use geo::algorithm::simplify::Simplify as _;
use geo_core::error::GeoError;
use geo_core::types::*;

pub fn simplify(geom: &Geometry, tolerance: f64) -> Result<Geometry, GeoError> {
    let result = match geom {
        Geometry::LineString(ls) => {
            let gt_ls: geo_types::LineString = ls.coords.iter()
                .map(|p| geo_types::Coord { x: p.x, y: p.y }).collect::<Vec<_>>().into();
            Geometry::LineString(LineString {
                coords: gt_ls.simplify(&tolerance).coords().map(|c| Point { x: c.x, y: c.y }).collect(),
            })
        }
        Geometry::Polygon(p) => {
            let gt_poly = poly_to_geo(p);
            let simplified = gt_poly.simplify(&tolerance);
            Geometry::Polygon(Polygon {
                exterior: LineString { coords: simplified.exterior().coords().map(|c| Point { x: c.x, y: c.y }).collect() },
                interiors: simplified.interiors().iter().map(|ls| LineString { coords: ls.coords().map(|c| Point { x: c.x, y: c.y }).collect() }).collect(),
            })
        }
        Geometry::MultiPolygon(mp) => {
            Geometry::MultiPolygon(MultiPolygon {
                polygons: mp.polygons.iter().map(|p| {
                    let gt_poly = poly_to_geo(p);
                    let simplified = gt_poly.simplify(&tolerance);
                    Polygon {
                        exterior: LineString { coords: simplified.exterior().coords().map(|c| Point { x: c.x, y: c.y }).collect() },
                        interiors: simplified.interiors().iter().map(|ls| LineString { coords: ls.coords().map(|c| Point { x: c.x, y: c.y }).collect() }).collect(),
                    }
                }).collect(),
            })
        }
        _ => return Err(GeoError::OperationNotSupported {
            op: "simplify".into(),
            reason: "simplify only supports LineString, Polygon, and MultiPolygon".into(),
        }),
    };
    Ok(result)
}

fn poly_to_geo(p: &Polygon) -> geo_types::Polygon {
    geo_types::Polygon::new(
        p.exterior.coords.iter().map(|pt| geo_types::Coord { x: pt.x, y: pt.y }).collect::<Vec<_>>().into(),
        p.interiors.iter().map(|i| i.coords.iter().map(|pt| geo_types::Coord { x: pt.x, y: pt.y }).collect::<Vec<_>>().into()).collect(),
    )
}
