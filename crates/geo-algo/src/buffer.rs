use geo_core::error::GeoError;
use geo_core::types::{Geometry, Units};

pub fn buffer(_geom: &Geometry, _distance: f64, _units: Units) -> Result<Geometry, GeoError> {
    Err(GeoError::OperationNotSupported {
        op: "buffer".into(),
        reason: "buffer algorithm not in geo 0.28 core; pending separate implementation".into(),
    })
}
