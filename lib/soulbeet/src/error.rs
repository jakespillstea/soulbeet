use thiserror::Error;

#[derive(Error, Debug)]
pub enum SoulseekError {
    #[error("Client is not configured. Base URL is missing.")]
    NotConfigured,

    #[error("Request error: {0}")]
    Request(#[from] reqwest::Error),

    #[error("URL parsing error: {0}")]
    UrlParse(#[from] url::ParseError),

    #[error("API error: {status} - {message}")]
    Api { status: u16, message: String },

    #[error("Failed to acquire lock for rate limiting")]
    LockError,

    #[error("Search timed out")]
    SearchTimeout,

    #[error("Could not find a username for the given download ID")]
    UsernameNotFound,
}

pub type Result<T> = std::result::Result<T, SoulseekError>;
