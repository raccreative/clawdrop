use std::fmt;

#[derive(Debug)]
pub enum ApiKeyError {
    MissingEnv,
}

impl fmt::Display for ApiKeyError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ApiKeyError::MissingEnv => write!(f, "CLAWDROP_API_KEY is not configured. Please run 'clawdrop auth' first."),
        }
    }
}