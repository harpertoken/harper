/// Model configurations for different AI providers
#[derive(Debug, Clone)]
pub struct ProviderModels {
    pub base_url: &'static str,
    pub default_model: &'static str,
}

impl ProviderModels {
    pub const OPENAI: ProviderModels = ProviderModels {
        base_url: "https://api.openai.com/v1/chat/completions",
        default_model: "gpt-4-turbo",
    };

    pub const SAMBANOVA: ProviderModels = ProviderModels {
        base_url: "https://api.sambanova.ai/v1/chat/completions",
        default_model: "Llama-4-Maverick-17B-128E-Instruct",
    };

    pub const GEMINI: ProviderModels = ProviderModels {
        base_url: "https://generativelanguage.googleapis.com/v1beta/models/gemini-2.5-flash:generateContent",
        default_model: "gemini-2.5-flash",
    };
}