// Copyright 2025 harpertoken
//
// Licensed under the Apache License, Version 2.0 (the "License");
// you may not use this file except in compliance with the License.
// You may obtain a copy of the License at
//
//     http://www.apache.org/licenses/LICENSE-2.0
//
// Unless required by applicable law or agreed to in writing, software
// distributed under the License is distributed on an "AS IS" BASIS,
// WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
// See the License for the specific language governing permissions and
// limitations under the License.

use colored::Colorize;
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
    /// File operation errors
    File(String),
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
            HarperError::File(msg) => write!(f, "File operation error: {}", msg),
            HarperError::Command(msg) => write!(f, "Command execution error: {}", msg),
            HarperError::WebSearch(msg) => write!(f, "Web search error: {}", msg),
        }
    }
}

impl std::error::Error for HarperError {}

impl HarperError {
    /// Get formatted error message for display
    pub fn display_message(&self) -> String {
        self.to_string()
    }

    /// Get colored error message for CLI
    pub fn cli_message(&self) -> colored::ColoredString {
        self.display_message().red()
    }
}

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

impl From<rustyline::error::ReadlineError> for HarperError {
    fn from(err: rustyline::error::ReadlineError) -> Self {
        HarperError::Io(err.to_string())
    }
}

/// Result type alias for Harper operations
pub type HarperResult<T> = Result<T, HarperError>;
