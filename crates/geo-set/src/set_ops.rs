use geo_core::error::GeoError;
use geo_core::types::Geometry;

pub fn union(_a: &Geometry, _b: &Geometry) -> Result<Geometry, GeoError> {
    Err(GeoError::OperationNotSupported { op: "union".into(), reason: "not yet implemented".into() })
}

pub fn intersect(_a: &Geometry, _b: &Geometry) -> Result<Geometry, GeoError> {
    Err(GeoError::OperationNotSupported { op: "intersect".into(), reason: "not yet implemented".into() })
}

pub fn difference(_a: &Geometry, _b: &Geometry) -> Result<Geometry, GeoError> {
    Err(GeoError::OperationNotSupported { op: "difference".into(), reason: "not yet implemented".into() })
}
