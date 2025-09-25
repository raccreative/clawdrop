use std::{fmt, io};

use crate::{errors::{api_key::ApiKeyError, common::CommonError}, impl_from};

#[derive(Debug)]
pub enum PostError {
    Common(CommonError),
    ApiKey(ApiKeyError),
    UnauthorizedToPost,
    NoIdSpecified,
    ServerError { code: u16, message: String }
}

impl fmt::Display for PostError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        use PostError::*;
        match self {
            Common(e) => write!(f, "{}", e),
            ApiKey(e) => write!(f, "{}", e),
            ServerError { code, message } => write!(f, "Error in HTTP response: {}, {}", code, message),
            UnauthorizedToPost => write!(f, "You don't have permissions to create a post for this game, are you the developer?"),
            NoIdSpecified => write!(f, "No game ID specified or target found, try using --id or clawdrop set <id>"),
        }
    }
}

impl std::error::Error for PostError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            PostError::Common(e) => Some(e),
            _ => None,
        }
    }
}

impl_from!(ApiKeyError => PostError::ApiKey);
impl_from!(reqwest::Error => PostError::Common : into);
impl_from!(io::Error => PostError::Common : into);