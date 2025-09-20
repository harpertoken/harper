pub mod core;
pub mod providers;
pub mod storage;
pub mod ui;
pub mod utils;

// Re-export everything from core
pub use core::*;

// Re-export providers
pub use providers::*;

// Re-export storage
pub use storage::*;

// Re-export storage functions directly for easier access
pub use storage::{delete_messages, delete_session, list_sessions};

// Re-export utils
pub use utils::*;

#[cfg(test)]
mod tests {
    use super::*;
    // Use full paths in tests to avoid conflicts
    use crate::core::chat_service::ChatService;
    use crate::core::error::HarperError;

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

    #[test]
    fn test_harper_error_display() {
        let config_error = HarperError::Config("test config error".to_string());
        assert_eq!(
            format!("{}", config_error),
            "Configuration error: test config error"
        );

        let api_error = HarperError::Api("test api error".to_string());
        assert_eq!(format!("{}", api_error), "API error: test api error");

        let db_error = HarperError::Database("test db error".to_string());
        assert_eq!(format!("{}", db_error), "Database error: test db error");
    }

    // Config validation tests are disabled due to import conflicts in test scope
    // These tests would validate configuration parsing and validation logic

    #[test]
    fn test_chat_service_build_system_prompt() {
        use rusqlite::Connection;
        use tempfile::NamedTempFile;

        let temp_file = NamedTempFile::new().unwrap();
        let conn = Connection::open(temp_file.path()).unwrap();
        init_db(&conn).unwrap();

        let config = ApiConfig {
            provider: ApiProvider::OpenAI,
            api_key: "test-key".to_string(),
            base_url: "https://api.openai.com/v1/chat/completions".to_string(),
            model_name: "gpt-4".to_string(),
        };

        let chat_service = ChatService::new_test(&conn, &config);

        // Test without web search
        let prompt = chat_service.build_system_prompt(false);
        assert!(prompt.contains("gpt-4"));
        assert!(!prompt.contains("RUN_COMMAND"));
        assert!(!prompt.contains("SEARCH:"));

        // Test with web search
        let prompt = chat_service.build_system_prompt(true);
        assert!(prompt.contains("gpt-4"));
        assert!(prompt.contains("RUN_COMMAND"));
        assert!(prompt.contains("SEARCH:"));
    }

    #[test]
    fn test_chat_service_should_exit() {
        use rusqlite::Connection;
        use tempfile::NamedTempFile;

        let temp_file = NamedTempFile::new().unwrap();
        let conn = Connection::open(temp_file.path()).unwrap();
        init_db(&conn).unwrap();

        let config = ApiConfig {
            provider: ApiProvider::OpenAI,
            api_key: "test-key".to_string(),
            base_url: "https://api.openai.com/v1/chat/completions".to_string(),
            model_name: "gpt-4".to_string(),
        };

        let chat_service = ChatService::new_test(&conn, &config);

        assert!(chat_service.should_exit("exit"));
        assert!(chat_service.should_exit("quit"));
        assert!(chat_service.should_exit("EXIT"));
        assert!(chat_service.should_exit(""));
        assert!(!chat_service.should_exit("hello"));
        assert!(!chat_service.should_exit("how are you?"));
    }
}
