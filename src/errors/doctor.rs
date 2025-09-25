use std::{fmt, io};

use crate::{errors::{api_key::ApiKeyError, common::CommonError}, impl_from};

#[derive(Debug)]
pub enum DoctorError {
    Common(CommonError),
    ApiKey(ApiKeyError),
    InvalidApiKey,
    InternetUnavailable {
        message: String,
        source: Option<Box<dyn std::error::Error + Send + Sync>>,
    },
}

impl fmt::Display for DoctorError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        use DoctorError::*;
        match self {
            InvalidApiKey => write!(f, "API key is invalid or expired."),
            InternetUnavailable { message, source } => {
                write!(f, "Internet check failed: {}", message)?;
                if let Some(err) = source.as_ref() {
                    write!(f, " (caused by: {})", err)?;
                }
                Ok(())
            },
            Common(e) => write!(f, "{}", e),
            ApiKey(e) => write!(f, "{}", e),
        }
    }
}

impl std::error::Error for DoctorError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        use DoctorError::*;
        match self {
            Common(e) => Some(e),
            InternetUnavailable { source: Some(e), .. } => Some(&**e),
            _ => None,
        }
    }
}

impl_from!(io::Error => DoctorError::Common : into);
impl_from!(reqwest::Error => DoctorError::Common : into);
impl_from!(ApiKeyError => DoctorError::ApiKey);