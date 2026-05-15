use geo_core::error::GeoError;
use geo_core::types::Geometry;

pub fn simplify(_geom: &Geometry, _tolerance: f64) -> Result<Geometry, GeoError> {
    Err(GeoError::OperationNotSupported {
        op: "simplify".into(),
        reason: "not yet implemented".into(),
    })
}
