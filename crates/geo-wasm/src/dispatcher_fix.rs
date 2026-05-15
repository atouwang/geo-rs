// Replacement for the problematic sections in dispatcher.rs

fn transform_geometry_fixed(
    geom: &Geometry,
    from: geo_core::types::CoordSystem,
    to: geo_core::types::CoordSystem,
) -> Geometry {
    use geo_core::coords::transform_coords;
    use geo_core::types::*;
    match geom {
        Geometry::Point(p) => Geometry::Point(transform_coords(p, from, to)),
        Geometry::LineString(ls) => Geometry::LineString(LineString {
            coords: ls.coords.iter().map(|p| transform_coords(p, from, to)).collect(),
        }),
        Geometry::Polygon(p) => Geometry::Polygon(Polygon {
            exterior: LineString {
                coords: p.exterior.coords.iter().map(|pt| transform_coords(pt, from, to)).collect(),
            },
            interiors: p.interiors.iter().map(|ls| LineString {
                coords: ls.coords.iter().map(|pt| transform_coords(pt, from, to)).collect(),
            }).collect(),
        }),
        Geometry::MultiLineString(mls) => Geometry::MultiLineString(MultiLineString {
            lines: mls.lines.iter().map(|ls| LineString {
                coords: ls.coords.iter().map(|pt| transform_coords(pt, from, to)).collect(),
            }).collect(),
        }),
        Geometry::MultiPoint(mp) => Geometry::MultiPoint(MultiPoint {
            points: mp.points.iter().map(|p| transform_coords(p, from, to)).collect(),
        }),
        other => other.clone(),
    }
}

fn voronoi_from_handle_fixed(
    arena: &mut crate::arena::MemoryArena,
    points_handle: u64,
    bbox_json: &str,
) -> Result<u64, String> {
    use geo_core::types::*;
    let points_geom = arena.get(points_handle).map_err(|e| e.to_string())?;
    let pts = match points_geom {
        Geometry::MultiPoint(mp) => mp.points.clone(),
        _ => return Err("voronoi requires MultiPoint input".to_string()),
    };
    let bbox: geo_core::types::BBox = serde_json::from_str(bbox_json)
        .map_err(|e| format!("Invalid bbox JSON: {}", e))?;
    let polygons = geo_grid::voronoi::voronoi(&pts, &bbox);
    let result = Geometry::GeometryCollection(
        polygons.into_iter().map(Geometry::Polygon).collect()
    );
    arena.store(result).map_err(|e| e.to_string())
}

fn isolines_from_handle_fixed(
    arena: &mut crate::arena::MemoryArena,
    points_handle: u64,
    values_json: &str,
    breaks_json: &str,
) -> Result<u64, String> {
    use geo_core::types::*;
    let points_geom = arena.get(points_handle).map_err(|e| e.to_string())?;
    let pts = match points_geom {
        Geometry::MultiPoint(mp) => mp.points.clone(),
        _ => return Err("isolines requires MultiPoint input".to_string()),
    };
    let values: Vec<f64> = serde_json::from_str(values_json)
        .map_err(|e| format!("Invalid values JSON: {}", e))?;
    let breaks: Vec<f64> = serde_json::from_str(breaks_json)
        .map_err(|e| format!("Invalid breaks JSON: {}", e))?;
    let line_strings = geo_grid::isolines::isolines(&pts, &values, &breaks);
    let result = Geometry::MultiLineString(MultiLineString { lines: line_strings });
    arena.store(result).map_err(|e| e.to_string())
}
