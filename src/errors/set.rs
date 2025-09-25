use std::{fmt, io};

use crate::{errors::{api_key::ApiKeyError, common::CommonError, network::NetworkError}, impl_from};

#[derive(Debug)]
pub enum SetError {
    Common(CommonError),
    ApiKey(ApiKeyError),
    UnauthorizedToSetGame,
    SerializationError,
    InvalidConfigPath,
}

impl fmt::Display for SetError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        use SetError::*;
        match self {
            Common(e) => write!(f, "{}", e),
            ApiKey(e) => write!(f, "{}", e),
            UnauthorizedToSetGame => write!(f, "You don't have permissions to set this game, are you the developer?"),
            SerializationError => write!(f, "Could not serializate"),
            InvalidConfigPath => write!(f, "Invalid route for target file."),
        }
    }
}

impl std::error::Error for SetError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        use SetError::*;
        match self {
            Common(e) => Some(e),
            _ => None,
        }
    }
}

impl_from!(NetworkError => SetError::Common : into);
impl_from!(ApiKeyError => SetError::ApiKey);
impl_from!(io::Error => SetError::Common : into);
impl_from!(reqwest::Error => SetError::Common : into);