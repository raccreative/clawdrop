use std::fmt;

use crate::{errors::{api_key::ApiKeyError, network::NetworkError}, impl_from};

#[derive(Debug)]
pub enum ListError {
    Network(NetworkError),
    ApiKey(ApiKeyError),
}

impl fmt::Display for ListError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        use ListError::*;
        match self {
            Network(e) => write!(f, "{}", e),
            ApiKey(e) => write!(f, "{}", e),
        }
    }
}

impl std::error::Error for ListError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            ListError::Network(e) => Some(e),
            _ => None,
        }
    }
}

impl_from!(NetworkError => ListError::Network);
impl_from!(ApiKeyError => ListError::ApiKey);
impl_from!(reqwest::Error => ListError::Network : into);