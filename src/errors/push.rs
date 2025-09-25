use std::{fmt, io};

use tokio::task::JoinError;

use crate::{
    errors::{api_key::ApiKeyError, common::CommonError, set::SetError},
    impl_from,
};

#[derive(Debug)]
pub enum PushError {
    Common(CommonError),
    ApiKey(ApiKeyError),
    Set(SetError),
    Join(JoinError),
    InvalidShorthandFormat,
    InvalidShorthandId,
    InvalidOS,
    MissingId,
    MissingOS,
    MissingExecutableName,
    MissingExecutableFile,
    UnauthorizedToUpload,
    FileSizeLimitReach,
    FileindexMismatch,
    ServerError { code: u16, message: String },
    GameNotFound,
    S3Error { message: String },
}

impl fmt::Display for PushError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        use PushError::*;
        match self {
            Common(e) => write!(f, "{}", e),
            ApiKey(e) => write!(f, "{}", e),
            Set(e) => write!(f, "{}", e),
            Join(e) => write!(f, "{}", e),
            InvalidShorthandFormat => write!(
                f,
                "Shorthand format is not valid, it must be <id>:<os>/<executableName>:<version> id and version optional."
            ),
            InvalidShorthandId => write!(
                f,
                "Shorthand [id] format is not valid, it must be a number."
            ),
            InvalidOS => write!(
                f,
                "Operating System is not valid, it must be 'mac', 'windows', 'linux' or 'html'"
            ),
            MissingId => write!(
                f,
                "No game ID specified or target found, try using --id or clawdrop set <id>"
            ),
            MissingOS => write!(
                f,
                "Operating System is missing, it was not provided via --os or shorthand [windows, linux, mac, html]"
            ),
            MissingExecutableName => write!(
                f,
                "Executable name is missing, it was not provided via --exe or shorthand [example: My game.exe]"
            ),
            MissingExecutableFile => write!(
                f,
                "Executable file provided was not found inside build, please check you spelled the name correctly."
            ),
            UnauthorizedToUpload => write!(
                f,
                "You are unauthorized to upload files to this game. Check if you are a developer or the API Key is valid."
            ),
            FileSizeLimitReach => write!(
                f,
                "You have reached the build size limit for non Star users, please if you want large games subscribe to Raccreative Star"
            ),
            FileindexMismatch => write!(
                f,
                "Uploaded fileindex.json does not match with uploaded files, is fileindex.json correct or has been manipulated?"
            ),
            GameNotFound => write!(f, "The game you are trying to use was not found."),
            ServerError { code, message } => {
                write!(f, "Error in HTTP response: {}, {}", code, message)
            }
            S3Error { message } => write!(f, "Error in S3: {}", message),
        }
    }
}

impl std::error::Error for PushError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            PushError::Common(e) => Some(e),
            _ => None,
        }
    }
}

impl_from!(ApiKeyError => PushError::ApiKey);
impl_from!(reqwest::Error => PushError::Common : into);
impl_from!(io::Error => PushError::Common : into);
impl_from!(serde_json::Error => PushError::Common : into);
impl_from!(CommonError => PushError::Common);
impl_from!(SetError => PushError::Set);
impl_from!(JoinError => PushError::Join);
