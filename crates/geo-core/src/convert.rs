use crate::error::GeoError;
use crate::types::*;
use geojson::{GeometryValue as Gv, Position};

// --- MessagePack binary serialization ---

pub fn from_msgpack(bytes: &[u8]) -> Result<Geometry, GeoError> {
    rmp_serde::from_slice(bytes)
        .map_err(|e| GeoError::SerializationError(format!("MsgPack: {}", e)))
}

pub fn to_msgpack(geom: &Geometry) -> Result<Vec<u8>, GeoError> {
    rmp_serde::to_vec(geom)
        .map_err(|e| GeoError::SerializationError(format!("MsgPack: {}", e)))
}

// --- GeoJSON text serialization ---

pub fn from_geojson(json: &str) -> Result<Geometry, GeoError> {
    let gj: geojson::GeoJson =
        serde_json::from_str(json).map_err(|e| GeoError::SerializationError(format!("{}", e)))?;
    geojson_to_geometry(&gj)
}

pub fn to_geojson(geom: &Geometry) -> Result<String, GeoError> {
    let gj = geometry_to_geojson(geom);
    serde_json::to_string(&gj).map_err(|e| GeoError::SerializationError(format!("{}", e)))
}

fn geojson_to_geometry(gj: &geojson::GeoJson) -> Result<Geometry, GeoError> {
    match gj {
        geojson::GeoJson::Geometry(g) => geojson_geom_to_geometry(g),
        geojson::GeoJson::Feature(f) => f.geometry.as_ref()
            .map(|g| geojson_geom_to_geometry(g))
            .unwrap_or_else(|| Err(GeoError::InvalidGeometry("Feature has no geometry".into()))),
        geojson::GeoJson::FeatureCollection(fc) => {
            let geoms: Result<Vec<_>, _> = fc.features.iter()
                .filter_map(|f| f.geometry.as_ref())
                .map(|g| geojson_geom_to_geometry(g))
                .collect();
            Ok(Geometry::GeometryCollection(geoms?))
        }
    }
}

fn pos_to_point(p: &Position) -> Point { Point { x: p[0], y: p[1] } }
fn pos_slice_to_points(pts: &[Position]) -> Vec<Point> {
    pts.iter().map(|p| pos_to_point(p)).collect()
}

fn geojson_geom_to_geometry(g: &geojson::Geometry) -> Result<Geometry, GeoError> {
    match &g.value {
        Gv::Point { coordinates: p } => Ok(Geometry::Point(pos_to_point(p))),
        Gv::MultiPoint { coordinates: pts } => Ok(Geometry::MultiPoint(MultiPoint {
            points: pts.iter().map(|p| pos_to_point(p)).collect(),
        })),
        Gv::LineString { coordinates: coords } => Ok(Geometry::LineString(LineString {
            coords: pos_slice_to_points(coords),
        })),
        Gv::MultiLineString { coordinates: lines } => Ok(Geometry::MultiLineString(MultiLineString {
            lines: lines.iter().map(|l| LineString { coords: pos_slice_to_points(l) }).collect(),
        })),
        Gv::Polygon { coordinates: rings } => {
            if rings.is_empty() {
                return Err(GeoError::InvalidGeometry("empty polygon".into()));
            }
            Ok(Geometry::Polygon(Polygon {
                exterior: LineString { coords: pos_slice_to_points(&rings[0]) },
                interiors: rings[1..].iter()
                    .map(|r| LineString { coords: pos_slice_to_points(r) }).collect(),
            }))
        }
        Gv::MultiPolygon { coordinates: polys } => Ok(Geometry::MultiPolygon(MultiPolygon {
            polygons: polys.iter().map(|rings| Polygon {
                exterior: LineString { coords: pos_slice_to_points(&rings[0]) },
                interiors: rings[1..].iter()
                    .map(|r| LineString { coords: pos_slice_to_points(r) }).collect(),
            }).collect(),
        })),
        Gv::GeometryCollection { geometries: gc } => {
            let geoms: Result<Vec<_>, _> = gc.iter()
                .map(|g| geojson_geom_to_geometry(g)).collect();
            Ok(Geometry::GeometryCollection(geoms?))
        }
    }
}

fn point_to_pos(p: &Point) -> Position { [p.x, p.y].into() }
fn pts_to_positions(pts: &[Point]) -> Vec<Position> {
    pts.iter().map(|p| point_to_pos(p)).collect()
}

fn geometry_to_geojson(geom: &Geometry) -> geojson::GeoJson {
    let gj_geom = geometry_to_geojson_geom(geom);
    geojson::GeoJson::Feature(geojson::Feature {
        bbox: None, geometry: Some(gj_geom), id: None, properties: None, foreign_members: None,
    })
}

fn geometry_to_geojson_geom(geom: &Geometry) -> geojson::Geometry {
    match geom {
        Geometry::Point(p) => geojson::Geometry::new(Gv::Point {
            coordinates: point_to_pos(p),
        }),
        Geometry::MultiPoint(mp) => geojson::Geometry::new(Gv::MultiPoint {
            coordinates: mp.points.iter().map(|p| point_to_pos(p)).collect(),
        }),
        Geometry::LineString(ls) => geojson::Geometry::new(Gv::LineString {
            coordinates: pts_to_positions(&ls.coords),
        }),
        Geometry::MultiLineString(mls) => geojson::Geometry::new(Gv::MultiLineString {
            coordinates: mls.lines.iter().map(|l| pts_to_positions(&l.coords)).collect(),
        }),
        Geometry::Polygon(p) => {
            let mut rings = vec![pts_to_positions(&p.exterior.coords)];
            rings.extend(p.interiors.iter().map(|r| pts_to_positions(&r.coords)));
            geojson::Geometry::new(Gv::Polygon { coordinates: rings })
        }
        Geometry::MultiPolygon(mp) => geojson::Geometry::new(Gv::MultiPolygon {
            coordinates: mp.polygons.iter().map(|p| {
                let mut rings = vec![pts_to_positions(&p.exterior.coords)];
                rings.extend(p.interiors.iter().map(|r| pts_to_positions(&r.coords)));
                rings
            }).collect(),
        }),
        Geometry::GeometryCollection(gc) => geojson::Geometry::new(Gv::GeometryCollection {
            geometries: gc.iter().map(|g| geometry_to_geojson_geom(g)).collect(),
        }),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_roundtrip_point() {
        let gj = r#"{"type":"Point","coordinates":[116.397,39.908]}"#;
        let geom = from_geojson(gj).unwrap();
        let back = to_geojson(&geom).unwrap();
        assert!(back.contains("116.397"));
        assert!(back.contains("39.908"));
    }

    #[test]
    fn test_roundtrip_polygon() {
        let gj = r#"{"type":"Polygon","coordinates":[[[0,0],[1,0],[1,1],[0,1],[0,0]]]}"#;
        let geom = from_geojson(gj).unwrap();
        let back = to_geojson(&geom).unwrap();
        assert!(back.contains("Polygon"));
    }

    #[test]
    fn test_msgpack_roundtrip() {
        let original = r#"{"type":"Point","coordinates":[116.397,39.908]}"#;
        let geom = from_geojson(original).unwrap();
        let bytes = to_msgpack(&geom).unwrap();
        let back = from_msgpack(&bytes).unwrap();
        assert_eq!(geom, back);
    }

    #[test]
    fn test_msgpack_polygon() {
        let gj = r#"{"type":"Polygon","coordinates":[[[0,0],[1,0],[1,1],[0,1],[0,0]]]}"#;
        let geom = from_geojson(gj).unwrap();
        let bytes = to_msgpack(&geom).unwrap();
        assert!(bytes.len() > 0);
        let back = from_msgpack(&bytes).unwrap();
        let json = to_geojson(&back).unwrap();
        assert!(json.contains("Polygon"));
    }
}
