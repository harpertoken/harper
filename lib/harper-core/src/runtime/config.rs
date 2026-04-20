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

use crate::core::error::{HarperError, HarperResult};
use crate::core::models::ProviderModels;
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
    pub ui: UiConfig,
    pub tools: ToolsConfig,
    pub exec_policy: ExecPolicyConfig,
    pub custom_commands: CustomCommandsConfig,
    pub firmware: FirmwareConfig,
    pub server: ServerConfig,
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

#[derive(Debug, Deserialize)]
pub struct UiConfig {
    pub theme: Option<String>,
    pub keys: Option<KeyConfig>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct KeyConfig {
    pub next: Option<String>,
    pub previous: Option<String>,
    pub enter: Option<String>,
    pub exit: Option<String>,
    pub tab: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct ToolsConfig {
    #[allow(dead_code)]
    pub enabled_tools: Option<Vec<String>>,
    #[allow(dead_code)]
    pub disabled_tools: Option<Vec<String>>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct ExecPolicyConfig {
    #[allow(dead_code)]
    pub allowed_commands: Option<Vec<String>>,
    #[allow(dead_code)]
    pub blocked_commands: Option<Vec<String>>,
    pub sandbox: Option<SandboxConfig>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct SandboxConfig {
    pub enabled: Option<bool>,
    pub allowed_dirs: Option<Vec<String>>,
    pub network_access: Option<bool>,
    pub readonly_home: Option<bool>,
    pub max_execution_time_secs: Option<u64>,
}

#[derive(Debug, Deserialize)]
pub struct CustomCommandsConfig {
    pub commands: Option<std::collections::HashMap<String, String>>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct FirmwareConfig {
    pub enabled: Option<bool>,
    pub devices: Option<Vec<FirmwareDeviceConfig>>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct FirmwareDeviceConfig {
    pub name: String,
    pub platform: String,
    pub port: Option<String>,
    pub address: Option<String>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct ServerConfig {
    pub enabled: Option<bool>,
    pub host: Option<String>,
    pub port: Option<u16>,
}

impl Default for ServerConfig {
    fn default() -> Self {
        Self {
            enabled: Some(false),
            host: Some("127.0.0.1".to_string()),
            port: Some(8080),
        }
    }
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
                temp_builder =
                    temp_builder.set_override("api.base_url", ProviderModels::OPENAI.base_url)?;
                temp_builder = temp_builder
                    .set_override("api.model_name", ProviderModels::OPENAI.default_model)?;
                *builder = temp_builder;
                return Ok(());
            }
        }

        // Map SAMBASTUDIO_API_KEY
        if let Ok(key) = env::var("SAMBASTUDIO_API_KEY") {
            if !key.trim().is_empty() {
                temp_builder = temp_builder.set_override("api.api_key", key)?;
                temp_builder = temp_builder.set_override("api.provider", "Sambanova")?;
                temp_builder = temp_builder
                    .set_override("api.base_url", ProviderModels::SAMBANOVA.base_url)?;
                temp_builder = temp_builder
                    .set_override("api.model_name", ProviderModels::SAMBANOVA.default_model)?;
                *builder = temp_builder;
                return Ok(());
            }
        }

        // Map GEMINI_API_KEY
        if let Ok(key) = env::var("GEMINI_API_KEY") {
            if !key.trim().is_empty() {
                temp_builder = temp_builder.set_override("api.api_key", key)?;
                temp_builder = temp_builder.set_override("api.provider", "Gemini")?;
                temp_builder =
                    temp_builder.set_override("api.base_url", ProviderModels::GEMINI.base_url)?;
                temp_builder = temp_builder
                    .set_override("api.model_name", ProviderModels::GEMINI.default_model)?;
                *builder = temp_builder;
                return Ok(());
            }
        }

        // Map OLLAMA_HOST/OLLAMA_BASE_URL
        let ollama_host = env::var("OLLAMA_HOST").or_else(|_| env::var("OLLAMA_BASE_URL"));
        if let Ok(host) = ollama_host {
            if !host.trim().is_empty() {
                let mut normalized = host.trim().trim_end_matches('/').to_string();
                if !normalized.starts_with("http://") && !normalized.starts_with("https://") {
                    normalized = format!("http://{}", normalized);
                }
                let base_url = if normalized.ends_with("/api/chat") {
                    normalized
                } else {
                    format!("{}/api/chat", normalized)
                };
                let model = env::var("OLLAMA_MODEL").unwrap_or_else(|_| "llama3".to_string());
                temp_builder = temp_builder.set_override("api.provider", "Ollama")?;
                temp_builder = temp_builder.set_override("api.base_url", base_url)?;
                temp_builder = temp_builder.set_override("api.model_name", model)?;
                temp_builder = temp_builder.set_override("api.api_key", "")?;
                *builder = temp_builder;
                return Ok(());
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
        self.ui.validate()?;
        self.tools.validate()?;
        self.exec_policy.validate()?;
        self.custom_commands.validate()?;
        Ok(())
    }
}

impl ApiConfig {
    /// Validate API configuration
    fn validate(&self) -> HarperResult<()> {
        // Validate provider
        let requires_api_key = match self.provider.as_str() {
            "OpenAI" | "Sambanova" | "Gemini" => true,
            "Ollama" => false,
            _ => {
                return Err(HarperError::Config(format!(
                "Invalid API provider: {}. Supported providers: OpenAI, Sambanova, Gemini, Ollama",
                self.provider
            )))
            }
        };

        // Validate API key
        if requires_api_key && self.api_key.trim().is_empty() {
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
            "Ollama" => Ok(ApiProvider::Ollama),
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

impl UiConfig {
    /// Validate UI configuration
    fn validate(&self) -> HarperResult<()> {
        if let Some(ref theme) = self.theme {
            match theme.as_str() {
                "default" | "dark" | "light" | "github" => {}
                _ => {
                    return Err(HarperError::Config(format!(
                        "Invalid theme: {}. Supported themes: default, dark, light, github",
                        theme
                    )))
                }
            }
        }
        Ok(())
    }
}

impl ToolsConfig {
    /// Validate tools configuration
    fn validate(&self) -> HarperResult<()> {
        // Basic validation - could add more specific tool name checks
        Ok(())
    }
}

impl ExecPolicyConfig {
    /// Validate exec policy configuration
    fn validate(&self) -> HarperResult<()> {
        // Basic validation
        Ok(())
    }
}

impl CustomCommandsConfig {
    /// Validate custom commands configuration
    fn validate(&self) -> HarperResult<()> {
        // Basic validation
        Ok(())
    }
}
