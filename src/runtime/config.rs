use crate::core::error::{HarperError, HarperResult};
use crate::core::ApiProvider;
use config::{ConfigBuilder, File};
use serde::Deserialize;
use std::env;
use std::path::Path;

#[derive(Debug, Deserialize)]
pub struct HarperConfig {
    pub api: ApiConfig,
    pub database: DatabaseConfig,
    pub mcp: McpConfig,
    pub prompts: PromptConfig,
}

#[derive(Debug, Deserialize)]
pub struct ApiConfig {
    pub provider: String,
    pub api_key: String,
    pub base_url: String,
    pub model_name: String,
}

#[derive(Debug, Deserialize)]
pub struct DatabaseConfig {
    pub path: String,
}

#[derive(Debug, Deserialize)]
pub struct McpConfig {
    pub enabled: bool,
    pub server_url: String,
}

#[derive(Debug, Deserialize)]
pub struct PromptConfig {
    pub system_prompt_id: Option<String>,
}

impl HarperConfig {
    /// Load and validate configuration
    pub fn new() -> HarperResult<Self> {
        let mut builder = ConfigBuilder::<config::builder::DefaultState>::default()
            .add_source(File::with_name("config/default"))
            .add_source(File::with_name("config/local").required(false))
            .add_source(config::Environment::with_prefix("HARPER"));

        Self::apply_env_overrides(&mut builder)?;

        let config = builder.build()?;
        let harper_config: Self = config.try_deserialize()?;

        // Validate the configuration
        harper_config.validate()?;

        Ok(harper_config)
    }

    /// Apply environment variable overrides to the config builder
    fn apply_env_overrides(
        builder: &mut ConfigBuilder<config::builder::DefaultState>,
    ) -> HarperResult<()> {
        let mut temp_builder = std::mem::take(builder);

        // Map OPENAI_API_KEY to api settings
        if let Ok(key) = env::var("OPENAI_API_KEY") {
            if !key.trim().is_empty() {
                temp_builder = temp_builder.set_override("api.api_key", key)?;
                temp_builder = temp_builder.set_override("api.provider", "OpenAI")?;
                temp_builder = temp_builder
                    .set_override("api.base_url", "https://api.openai.com/v1/chat/completions")?;
                temp_builder = temp_builder.set_override("api.model_name", "gpt-4-turbo")?;
            }
        }

        // Map SAMBASTUDIO_API_KEY
        if let Ok(key) = env::var("SAMBASTUDIO_API_KEY") {
            if !key.trim().is_empty() {
                temp_builder = temp_builder.set_override("api.api_key", key)?;
                temp_builder = temp_builder.set_override("api.provider", "Sambanova")?;
                temp_builder = temp_builder.set_override(
                    "api.base_url",
                    "https://api.sambanova.ai/v1/chat/completions",
                )?;
                temp_builder = temp_builder
                    .set_override("api.model_name", "Llama-4-Maverick-17B-128E-Instruct")?;
            }
        }

        // Map GEMINI_API_KEY
        if let Ok(key) = env::var("GEMINI_API_KEY") {
            if !key.trim().is_empty() {
                temp_builder = temp_builder.set_override("api.api_key", key)?;
                temp_builder = temp_builder.set_override("api.provider", "Gemini")?;
                temp_builder = temp_builder.set_override("api.base_url", "https://generativelanguage.googleapis.com/v1beta/models/gemini-2.5-flash:generateContent")?;
                temp_builder = temp_builder.set_override("api.model_name", "gemini-2.5-flash")?;
            }
        }

        // Map DATABASE_PATH
        if let Ok(path) = env::var("DATABASE_PATH") {
            if !path.trim().is_empty() {
                temp_builder = temp_builder.set_override("database.path", path)?;
            }
        }

        *builder = temp_builder;
        Ok(())
    }

    /// Validate configuration values
    fn validate(&self) -> HarperResult<()> {
        self.api.validate()?;
        self.database.validate()?;
        self.mcp.validate()?;
        Ok(())
    }
}

impl ApiConfig {
    /// Validate API configuration
    fn validate(&self) -> HarperResult<()> {
        // Validate provider
        match self.provider.as_str() {
            "OpenAI" | "Sambanova" | "Gemini" => {}
            _ => {
                return Err(HarperError::Config(format!(
                    "Invalid API provider: {}. Supported providers: OpenAI, Sambanova, Gemini",
                    self.provider
                )))
            }
        }

        // Validate API key
        if self.api_key.trim().is_empty() {
            return Err(HarperError::Config("API key cannot be empty".to_string()));
        }

        // Validate base URL
        if self.base_url.trim().is_empty() {
            return Err(HarperError::Config("Base URL cannot be empty".to_string()));
        }

        if !self.base_url.starts_with("http://") && !self.base_url.starts_with("https://") {
            return Err(HarperError::Config(
                "Base URL must start with http:// or https://".to_string(),
            ));
        }

        // Validate model name
        if self.model_name.trim().is_empty() {
            return Err(HarperError::Config(
                "Model name cannot be empty".to_string(),
            ));
        }

        Ok(())
    }

    /// Convert string provider to ApiProvider enum
    pub fn get_provider(&self) -> HarperResult<ApiProvider> {
        match self.provider.as_str() {
            "OpenAI" => Ok(ApiProvider::OpenAI),
            "Sambanova" => Ok(ApiProvider::Sambanova),
            "Gemini" => Ok(ApiProvider::Gemini),
            _ => Err(HarperError::Config(format!(
                "Unsupported provider: {}",
                self.provider
            ))),
        }
    }
}

impl DatabaseConfig {
    /// Validate database configuration
    fn validate(&self) -> HarperResult<()> {
        if self.path.trim().is_empty() {
            return Err(HarperError::Config(
                "Database path cannot be empty".to_string(),
            ));
        }

        // Check if the parent directory exists or can be created
        if let Some(parent) = Path::new(&self.path).parent() {
            if !parent.exists() {
                // Try to create the directory if it doesn't exist
                if let Err(e) = std::fs::create_dir_all(parent) {
                    return Err(HarperError::Config(format!(
                        "Failed to create database directory {}: {}",
                        parent.display(),
                        e
                    )));
                }
            }
        }

        Ok(())
    }
}

impl McpConfig {
    /// Validate MCP configuration
    fn validate(&self) -> HarperResult<()> {
        if self.enabled {
            if self.server_url.trim().is_empty() {
                return Err(HarperError::Config(
                    "MCP server URL cannot be empty when MCP is enabled".to_string(),
                ));
            }

            if !self.server_url.starts_with("http://") && !self.server_url.starts_with("https://") {
                return Err(HarperError::Config(
                    "MCP server URL must start with http:// or https://".to_string(),
                ));
            }
        }

        Ok(())
    }
}
