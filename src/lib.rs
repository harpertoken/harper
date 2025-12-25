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

pub mod agent;
pub mod core;
pub mod interfaces;
pub mod memory;
pub mod plugins;
pub mod runtime;
pub mod tools;

// Re-export core types
#[allow(unused_imports)]
pub use core::*;

// Re-export agent types
pub use agent::chat::*;

// Re-export tools
#[allow(unused_imports)]
pub use tools::*;

// Re-export memory functions
pub use memory::storage::*;

// Re-export runtime
#[allow(unused_imports)]
pub use runtime::*;

// Re-export interfaces
#[allow(unused_imports)]
pub use interfaces::*;

#[cfg(test)]
mod tests {
    use super::*;
    // Use full paths in tests to avoid conflicts
    use crate::agent::chat::ChatService;
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

        let temp_file = NamedTempFile::new().expect("Failed to create temp file for test");
        let conn = Connection::open(temp_file.path()).expect("Failed to open test database");
        init_db(&conn).expect("Failed to initialize test database");

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
        assert!(prompt.contains("run_command")); // Tools are always available now
        assert!(!prompt.contains("SEARCH:"));

        // Test with web search
        let prompt = chat_service.build_system_prompt(true);
        assert!(prompt.contains("gpt-4"));
        assert!(prompt.contains("run_command"));
        assert!(prompt.contains("SEARCH:"));
    }

    #[test]
    fn test_preprocess_file_references() {
        use rusqlite::Connection;
        use tempfile::NamedTempFile;

        let temp_file = NamedTempFile::new().expect("Failed to create temp file for test");
        let conn = Connection::open(temp_file.path()).expect("Failed to open test database");
        init_db(&conn).expect("Failed to initialize test database");

        let config = ApiConfig {
            provider: ApiProvider::OpenAI,
            api_key: "test-key".to_string(),
            base_url: "https://api.openai.com/v1/chat/completions".to_string(),
            model_name: "gpt-4".to_string(),
        };

        let chat_service = ChatService::new_test(&conn, &config);

        // Test basic file reference
        let result = chat_service.preprocess_file_references("Check this file @src/main.rs");
        assert_eq!(result, "Check this file [READ_FILE src/main.rs]");

        // Test file reference at start
        let result = chat_service.preprocess_file_references("@Cargo.toml please");
        assert_eq!(result, "[READ_FILE Cargo.toml] please");

        // Test multiple @ symbols (should process all file references)
        let result = chat_service.preprocess_file_references("Look at @file1.txt and @file2.txt");
        assert_eq!(
            result,
            "Look at [READ_FILE file1.txt] and [READ_FILE file2.txt]"
        );

        // Test no @ symbol
        let result = chat_service.preprocess_file_references("Just a normal message");
        assert_eq!(result, "Just a normal message");

        // Test @ followed by nothing
        let result = chat_service.preprocess_file_references("Message with @");
        assert_eq!(result, "Message with @");

        // Test @ followed by command-like syntax (should skip)
        let result = chat_service.preprocess_file_references("Use @/help command");
        assert_eq!(result, "Use @/help command");

        // Test multiple file references with mixed valid/invalid
        let result = chat_service.preprocess_file_references("@file1.txt @/invalid @file2.txt");
        assert_eq!(
            result,
            "[READ_FILE file1.txt] @/invalid [READ_FILE file2.txt]"
        );

        // Test file references with spaces in names (should work with spaces)
        let result = chat_service.preprocess_file_references("Check @src/main.rs and @README.md");
        assert_eq!(
            result,
            "Check [READ_FILE src/main.rs] and [READ_FILE README.md]"
        );

        // Test empty file reference (should skip)
        let result = chat_service.preprocess_file_references("Check @ and continue");
        assert_eq!(result, "Check @ and continue");

        // Test @ followed by space (should skip)
        let result = chat_service.preprocess_file_references("Check @ file.txt");
        assert_eq!(result, "Check @ file.txt");

        // Test file path containing @ symbol (should treat as single path)
        let result = chat_service.preprocess_file_references("@file1.txt@file2.txt");
        assert_eq!(result, "[READ_FILE file1.txt@file2.txt]");

        // Test @ at end of string
        let result = chat_service.preprocess_file_references("Check this @");
        assert_eq!(result, "Check this @");

        // Test multiple @ with invalid ones mixed in
        let result = chat_service.preprocess_file_references("@valid.txt @ @invalid/ @another.txt");
        assert_eq!(
            result,
            "[READ_FILE valid.txt] @ [READ_FILE invalid/] [READ_FILE another.txt]"
        );
    }

    #[test]
    fn test_clear_todos_returns_count() {
        use rusqlite::Connection;
        use tempfile::NamedTempFile;

        let temp_file = NamedTempFile::new().expect("Failed to create temp file for test");
        let conn = Connection::open(temp_file.path()).expect("Failed to open test database");
        init_db(&conn).expect("Failed to initialize test database");

        // Add some todos
        crate::memory::storage::save_todo(&conn, "Test todo 1").unwrap();
        crate::memory::storage::save_todo(&conn, "Test todo 2").unwrap();
        crate::memory::storage::save_todo(&conn, "Test todo 3").unwrap();

        // Verify todos were added
        let todos = crate::memory::storage::load_todos(&conn).unwrap();
        assert_eq!(todos.len(), 3);

        // Clear todos and check return count
        let cleared_count = crate::memory::storage::clear_todos(&conn).unwrap();
        assert_eq!(cleared_count, 3);

        // Verify todos were cleared
        let todos_after = crate::memory::storage::load_todos(&conn).unwrap();
        assert_eq!(todos_after.len(), 0);
    }

    #[test]
    fn test_chat_service_should_exit() {
        use rusqlite::Connection;
        use tempfile::NamedTempFile;

        let temp_file = NamedTempFile::new().expect("Failed to create temp file for test");
        let conn = Connection::open(temp_file.path()).expect("Failed to open test database");
        init_db(&conn).expect("Failed to initialize test database");

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
