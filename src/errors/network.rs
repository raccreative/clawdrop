use std::fmt;
use std::error::Error;

#[derive(Debug)]
pub enum NetworkError {
    Http {
        message: String,
        source: reqwest::Error,
    },
    Json {
        message: String,
        source: reqwest::Error,
    },
    InvalidApiKey
}

impl fmt::Display for NetworkError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        use NetworkError::*;
        match self {
            Http { message, source } => {
                write!(f, "{}: {}", message, source)
            }
            Json { message, source } => {
                write!(f, "{}: {}", message, source)
            }
            InvalidApiKey => write!(f, "HTTP ERROR: API key might be invalid or expired. Run 'clawdrop auth' for a new one."),
        }
    }
}

impl Error for NetworkError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        match self {
            NetworkError::Http { source, .. } => Some(source),
            NetworkError::Json { source, .. } => Some(source),
            _ => None
        }
    }
}

impl From<reqwest::Error> for NetworkError {
    fn from(e: reqwest::Error) -> Self {
        if e.is_decode() {
            NetworkError::Json {
                message: "Failed to parse JSON response".to_string(),
                source: e,
            }
        } else {
            NetworkError::Http {
                message: "HTTP request failed".to_string(),
                source: e,
            }
        }
    }
}
