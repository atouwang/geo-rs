use geo_buffer::buffer_polygon;
use geo_core::error::GeoError;
use geo_core::types::*;

pub fn buffer(geom: &Geometry, distance: f64, _units: Units) -> Result<Geometry, GeoError> {
    let result_mp = match geom {
        Geometry::Polygon(p) => buffer_polygon(&polygon_to_geo(p), distance),
        Geometry::MultiPolygon(mp) => {
            let mut all: Vec<geo_types::Polygon> = vec![];
            for p in &mp.polygons {
                let merged = buffer_polygon(&polygon_to_geo(p), distance);
                for poly in merged {
                    all.push(poly);
                }
            }
            geo_types::MultiPolygon::new(all)
        }
        _ => {
            return Err(GeoError::OperationNotSupported {
                op: "buffer".into(),
                reason: "buffer only supports Polygon and MultiPolygon".into(),
            })
        }
    };
    Ok(geo_mp_to_ours(&result_mp))
}

fn polygon_to_geo(p: &Polygon) -> geo_types::Polygon {
    geo_types::Polygon::new(
        p.exterior.coords.iter().map(|pt| geo_types::Coord { x: pt.x, y: pt.y }).collect::<Vec<_>>().into(),
        p.interiors
            .iter()
            .map(|i| i.coords.iter().map(|pt| geo_types::Coord { x: pt.x, y: pt.y }).collect::<Vec<_>>().into())
            .collect(),
    )
}

fn geo_mp_to_ours(mp: &geo_types::MultiPolygon) -> Geometry {
    Geometry::MultiPolygon(MultiPolygon {
        polygons: mp
            .iter()
            .map(|p| Polygon {
                exterior: LineString { coords: p.exterior().coords().map(|c| Point { x: c.x, y: c.y }).collect() },
                interiors: p
                    .interiors()
                    .iter()
                    .map(|ls| LineString { coords: ls.coords().map(|c| Point { x: c.x, y: c.y }).collect() })
                    .collect(),
            })
            .collect(),
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    fn square() -> Geometry {
        Geometry::Polygon(Polygon {
            exterior: LineString {
                coords: vec![
                    Point { x: 0.0, y: 0.0 },
                    Point { x: 1.0, y: 0.0 },
                    Point { x: 1.0, y: 1.0 },
                    Point { x: 0.0, y: 1.0 },
                    Point { x: 0.0, y: 0.0 },
                ],
            },
            interiors: vec![],
        })
    }

    #[test]
    fn test_buffer_square() {
        let result = buffer(&square(), 0.1, Units::Meters).unwrap();
        assert!(matches!(result, Geometry::MultiPolygon(_)));
    }
}
