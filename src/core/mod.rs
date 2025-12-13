//! Core functionality for Harper AI Agent
//!
//! This module contains the fundamental types and services used throughout the application.

pub mod cache;
pub mod constants;
pub mod error;
pub mod io_traits;
pub mod llm_client;

/// Supported AI API providers
#[derive(Debug, Clone, Copy)]
pub enum ApiProvider {
    OpenAI,
    Sambanova,
    Gemini,
}

impl std::fmt::Display for ApiProvider {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ApiProvider::OpenAI => write!(f, "OpenAI"),
            ApiProvider::Sambanova => write!(f, "Sambanova"),
            ApiProvider::Gemini => write!(f, "Gemini"),
        }
    }
}

/// Configuration for AI API connections
#[derive(Debug)]
pub struct ApiConfig {
    /// The AI provider to use
    pub provider: ApiProvider,
    /// API key for authentication
    pub api_key: String,
    /// Base URL for the API endpoint
    pub base_url: String,
    /// Name of the model to use
    pub model_name: String,
}

use serde::{Deserialize, Serialize};

/// A message in a conversation with an AI model
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Message {
    /// The role of the message sender (user, assistant, system)
    pub role: String,
    /// The content of the message
    pub content: String,
}
