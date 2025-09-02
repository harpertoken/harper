use std::fmt;

/// Custom error type for Harper application
#[derive(Debug)]
pub enum HarperError {
    /// Configuration related errors
    Config(String),
    /// Database related errors
    Database(String),
    /// API related errors
    Api(String),
    /// MCP related errors
    #[allow(dead_code)]
    Mcp(String),
    /// Cryptography related errors
    Crypto(String),
    /// I/O related errors
    Io(String),
    /// Command execution errors
    Command(String),
    /// Web search errors
    #[allow(dead_code)]
    WebSearch(String),
}

impl fmt::Display for HarperError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            HarperError::Config(msg) => write!(f, "Configuration error: {}", msg),
            HarperError::Database(msg) => write!(f, "Database error: {}", msg),
            HarperError::Api(msg) => write!(f, "API error: {}", msg),
            HarperError::Mcp(msg) => write!(f, "MCP error: {}", msg),
            HarperError::Crypto(msg) => write!(f, "Cryptography error: {}", msg),
            HarperError::Io(msg) => write!(f, "I/O error: {}", msg),
            HarperError::Command(msg) => write!(f, "Command execution error: {}", msg),
            HarperError::WebSearch(msg) => write!(f, "Web search error: {}", msg),
        }
    }
}

impl std::error::Error for HarperError {}

impl From<rusqlite::Error> for HarperError {
    fn from(err: rusqlite::Error) -> Self {
        HarperError::Database(err.to_string())
    }
}

impl From<reqwest::Error> for HarperError {
    fn from(err: reqwest::Error) -> Self {
        HarperError::Api(err.to_string())
    }
}

impl From<config::ConfigError> for HarperError {
    fn from(err: config::ConfigError) -> Self {
        HarperError::Config(err.to_string())
    }
}

impl From<std::io::Error> for HarperError {
    fn from(err: std::io::Error) -> Self {
        HarperError::Io(err.to_string())
    }
}

impl From<uuid::Error> for HarperError {
    fn from(err: uuid::Error) -> Self {
        HarperError::Io(format!("UUID generation error: {}", err))
    }
}

impl From<serde_json::Error> for HarperError {
    fn from(err: serde_json::Error) -> Self {
        HarperError::Api(format!("JSON serialization error: {}", err))
    }
}

/// Result type alias for Harper operations
pub type HarperResult<T> = Result<T, HarperError>;