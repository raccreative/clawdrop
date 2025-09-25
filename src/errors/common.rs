use std::fmt;
use std::io;

use crate::errors::network::NetworkError;
use crate::impl_from;

#[derive(Debug)]
pub enum CommonError {
    Network(NetworkError),
    Io(io::Error),
    Json(serde_json::Error)
}

impl fmt::Display for CommonError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        use CommonError::*;
        match self {
            Io(e) => write!(f, "Input/output error: {}", e),
            Network(e) => write!(f, "{}", e),
            Json(e) => write!(f, "JSON deserialization error: {}", e),
        }
    }
}

impl std::error::Error for CommonError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        use CommonError::*;
        match self {
            Io(e) => Some(e),
            Network(e) => Some(e),
            Json(e) => Some(e),
        }
    }
}

impl_from!(reqwest::Error => CommonError::Network : into);
impl_from!(io::Error => CommonError::Io);
impl_from!(serde_json::Error => CommonError::Json);
impl_from!(NetworkError => CommonError::Network);
