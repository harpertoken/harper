#[derive(Debug, Clone, Copy)]
pub enum ApiProvider {
    OpenAI,
    Sambanova,
    Gemini,
}

#[derive(Debug)]
pub struct ApiConfig {
    pub provider: ApiProvider,
    pub api_key: String,
    pub base_url: String,
    pub model_name: String,
}

use serde::Serialize;

#[derive(Debug, Serialize)]
pub struct Message {
    pub role: String,
    pub content: String,
}
