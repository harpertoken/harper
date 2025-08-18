pub mod core;
pub mod providers;
pub mod storage;
pub mod ui;
pub mod utils;

pub use core::*;
pub use providers::*;
pub use storage::*;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_api_provider_variants() {
        let providers = [
            ApiProvider::OpenAI,
            ApiProvider::Sambanova,
            ApiProvider::Gemini,
        ];

        for provider in providers {
            assert!(!format!("{:?}", provider).is_empty());
        }
    }

    #[test]
    fn test_api_config_creation() {
        let config = ApiConfig {
            provider: ApiProvider::OpenAI,
            api_key: "test-key".to_string(),
            base_url: "https://api.openai.com/v1/chat/completions".to_string(),
            model_name: "gpt-4".to_string(),
        };

        assert!(matches!(config.provider, ApiProvider::OpenAI));
        assert_eq!(config.api_key, "test-key");
        assert!(config.base_url.contains("openai.com"));
        assert_eq!(config.model_name, "gpt-4");
    }

    #[test]
    fn test_message_creation() {
        let message = Message {
            role: "user".to_string(),
            content: "Hello, world!".to_string(),
        };

        assert_eq!(message.role, "user");
        assert_eq!(message.content, "Hello, world!");
        assert!(!message.role.is_empty());
        assert!(!message.content.is_empty());
    }

    #[test]
    fn test_message_roles() {
        let roles = ["user", "assistant", "system"];

        for role in roles {
            let message = Message {
                role: role.to_string(),
                content: "Test content".to_string(),
            };
            assert_eq!(message.role, role);
        }
    }
}
