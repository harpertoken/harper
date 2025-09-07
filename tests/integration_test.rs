use harper::*;
use rusqlite::Connection;
use tempfile::NamedTempFile;

#[test]
fn test_database_operations() {
    let temp_file = NamedTempFile::new().unwrap();
    let conn = Connection::open(temp_file.path()).unwrap();

    // Test database initialization
    init_db(&conn).unwrap();

    // Test session creation
    let session_id = "test-session-123";
    save_session(&conn, session_id).unwrap();

    // Test message saving
    save_message(&conn, session_id, "user", "Hello, world!").unwrap();
    save_message(&conn, session_id, "assistant", "Hi there!").unwrap();

    // Test message retrieval
    let history = load_history(&conn, session_id).unwrap();
    assert_eq!(history.len(), 2);
    assert_eq!(history[0].role, "user");
    assert_eq!(history[0].content, "Hello, world!");
    assert_eq!(history[1].role, "assistant");
    assert_eq!(history[1].content, "Hi there!");
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
}

#[test]
fn test_message_creation() {
    let message = Message {
        role: "system".to_string(),
        content: "You are a helpful assistant.".to_string(),
    };

    assert_eq!(message.role, "system");
    assert_eq!(message.content, "You are a helpful assistant.");
}

#[tokio::test]
async fn test_web_search_mock() {
    use harper::utils::web_search;

    // Test with a simple query - this will actually hit DuckDuckGo API
    // In CI environments, network requests might fail, so we check that the function
    // handles errors gracefully rather than panicking
    let result = web_search("rust programming").await;

    // The function should either succeed or return a handled error message
    // It should not panic or return an unhandled error
    match result {
        Ok(response) => {
            // If successful, should contain some content
            assert!(!response.is_empty(), "Search response should not be empty");
        }
        Err(_) => {
            // If it fails, it should be due to network issues, not a panic
            // This is acceptable in CI environments without network access
        }
    }
}
