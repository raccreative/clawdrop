use std::fmt;
use std::io;

use crate::errors::common::CommonError;
use crate::impl_from;

#[derive(Debug)]
pub enum AuthError {
    Common(CommonError),
    InvalidConfigPath,
    MissingApiKeyInResponse,
    RequestExpired,
    InvalidTokenResponse,
    RequestTimeout,
    InvalidProvidedKey,
    UnknownStatus(String),
}

impl fmt::Display for AuthError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        use AuthError::*;
        match self {
            InvalidConfigPath => write!(f, "Invalid route for config file."),
            MissingApiKeyInResponse => write!(f, "No API Key found in response."),
            RequestExpired => write!(f, "Request expired. Try again."),
            InvalidTokenResponse => write!(f, "Request not found or invalid token."),
            RequestTimeout => write!(f, "Auth request timeout. Try again."),
            InvalidProvidedKey => write!(f, "Invalid provided API Key: Expired or not found."),
            UnknownStatus(status) => write!(f, "Unknown response status: {}", status),
            Common(e) => write!(f, "{}", e),
        }
    }
}

impl std::error::Error for AuthError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        use AuthError::*;
        match self {
            Common(e) => Some(e),
            _ => None,
        }
    }
}

impl_from!(io::Error => AuthError::Common : into);
impl_from!(reqwest::Error => AuthError::Common : into);
