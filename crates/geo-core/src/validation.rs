use crate::{error::GeoError, types::*};

fn point(p: &Point) -> Result<(), GeoError> {
    if p.x.is_finite() && p.y.is_finite() {
        Ok(())
    } else {
        Err(GeoError::InvalidGeometry("coordinates must be finite".into()))
    }
}

fn line(line: &LineString) -> Result<(), GeoError> {
    for p in &line.coords {
        point(p)?;
    }
    if line.coords.len() == 1 {
        return Err(GeoError::InvalidGeometry("a line needs at least two positions".into()));
    }
    Ok(())
}

fn ring(ring: &LineString) -> Result<(), GeoError> {
    line(ring)?;
    if ring.coords.len() < 4 || ring.coords.first() != ring.coords.last() {
        return Err(GeoError::InvalidGeometry(
            "polygon rings must be closed and contain at least four positions".into(),
        ));
    }
    let origin = ring.coords[0];
    let twice_area: f64 = ring
        .coords
        .windows(2)
        .map(|edge| (edge[0].x - origin.x) * (edge[1].y - origin.y) - (edge[1].x - origin.x) * (edge[0].y - origin.y))
        .sum();
    if !twice_area.is_finite() || twice_area == 0.0 {
        return Err(GeoError::InvalidGeometry("polygon rings must have finite nonzero area".into()));
    }
    Ok(())
}

/// Structural and numeric validation. This does not certify polygon topology.
pub fn validate_geometry(geom: &Geometry) -> Result<(), GeoError> {
    match geom {
        Geometry::Point(p) => point(p),
        Geometry::MultiPoint(mp) => mp.points.iter().try_for_each(point),
        Geometry::LineString(ls) => line(ls),
        Geometry::MultiLineString(mls) => mls.lines.iter().try_for_each(line),
        Geometry::Polygon(p) => {
            ring(&p.exterior)?;
            p.interiors.iter().try_for_each(ring)
        }
        Geometry::MultiPolygon(mp) => mp.polygons.iter().try_for_each(|p| {
            ring(&p.exterior)?;
            p.interiors.iter().try_for_each(ring)
        }),
        Geometry::GeometryCollection(gc) => gc.iter().try_for_each(validate_geometry),
    }
}
