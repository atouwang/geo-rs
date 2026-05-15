use geo_core::error::GeoError;
use geo_core::types::{Geometry, LineString, Point, Units};

pub fn along(_line: &LineString, _distance: f64) -> Result<Point, GeoError> {
    Err(GeoError::OperationNotSupported { op: "along".into(), reason: "not yet implemented".into() })
}

pub fn bearing(_p1: &Point, _p2: &Point) -> f64 { 0.0 }

pub fn destination(_origin: &Point, _distance: f64, _bearing: f64, _units: Units) -> Result<Point, GeoError> {
    Err(GeoError::OperationNotSupported { op: "destination".into(), reason: "not yet implemented".into() })
}

pub fn nearest_point_on_line(_line: &LineString, _point: &Point) -> Result<Point, GeoError> {
    Err(GeoError::OperationNotSupported { op: "nearest_point_on_line".into(), reason: "not yet implemented".into() })
}
