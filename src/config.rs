use config::{ConfigBuilder, File};
use serde::Deserialize;

#[derive(Debug, Deserialize)]
pub struct HarperConfig {
    pub api: ApiConfig,
    pub database: DatabaseConfig,
    pub mcp: McpConfig,
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

impl HarperConfig {
    pub fn new() -> Result<Self, config::ConfigError> {
        let builder = ConfigBuilder::<config::builder::DefaultState>::default()
            .add_source(File::with_name("config/default"))
            .add_source(File::with_name("config/local").required(false))
            .add_source(config::Environment::with_prefix("HARPER"));

        let config = builder.build()?;
        config.try_deserialize()
    }
}
