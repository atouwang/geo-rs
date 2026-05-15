use std::fmt;

#[derive(Debug)]
pub enum GeoError {
    InvalidGeometry(String),
    TopologyError(String),
    MemoryLimitExceeded { requested: u64, available: u64 },
    OperationNotSupported { op: String, reason: String },
    HandleNotFound(u64),
    SerializationError(String),
}

impl fmt::Display for GeoError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            GeoError::InvalidGeometry(msg) => write!(f, "Invalid geometry: {}", msg),
            GeoError::TopologyError(msg) => write!(f, "Topology error: {}", msg),
            GeoError::MemoryLimitExceeded { requested, available } => {
                write!(f, "Memory limit exceeded: requested {} bytes, {} bytes available", requested, available)
            }
            GeoError::OperationNotSupported { op, reason } => {
                write!(f, "Operation '{}' not supported: {}", op, reason)
            }
            GeoError::HandleNotFound(h) => write!(f, "Handle not found: {}", h),
            GeoError::SerializationError(msg) => write!(f, "Serialization error: {}", msg),
        }
    }
}

impl std::error::Error for GeoError {}
