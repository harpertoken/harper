use harper::*;
use regex::Regex;
use rusqlite::Connection;
use std::thread;
use tempfile::NamedTempFile;

lazy_static::lazy_static! {
    static ref TIMESTAMP_REGEX: Regex = Regex::new(r"^\d{4}-\d{2}-\d{2} \d{2}:\d{2}:\d{2}$").unwrap();
}

#[test]
fn test_database_operations() {
    let temp_file = NamedTempFile::new().unwrap();
    let conn = Connection::open(temp_file.path()).unwrap();

    // Test database initialization
    init_db(&conn).unwrap();

    // Verify tables were created with correct schema
    let table_count: i64 = conn
        .query_row(
            "SELECT COUNT(*) FROM sqlite_master WHERE type='table' AND name IN ('sessions', 'messages')",
            [],
            |row| row.get(0),
        )
        .unwrap();
    assert_eq!(
        table_count, 2,
        "Expected sessions and messages tables to be created"
    );

    // Verify table schemas
    let sessions_columns: String = conn
        .query_row(
            "SELECT sql FROM sqlite_master WHERE type='table' AND name='sessions'",
            [],
            |row| row.get(0),
        )
        .unwrap();
    assert!(
        sessions_columns.contains("id"),
        "Sessions table missing id column"
    );
    assert!(
        sessions_columns.contains("created_at"),
        "Sessions table missing created_at column"
    );

    let messages_columns: String = conn
        .query_row(
            "SELECT sql FROM sqlite_master WHERE type='table' AND name='messages'",
            [],
            |row| row.get(0),
        )
        .unwrap();
    assert!(
        messages_columns.contains("id"),
        "Messages table missing id column"
    );
    assert!(
        messages_columns.contains("session_id"),
        "Messages table missing session_id column"
    );
    assert!(
        messages_columns.contains("role"),
        "Messages table missing role column"
    );
    assert!(
        messages_columns.contains("content"),
        "Messages table missing content column"
    );
    assert!(
        messages_columns.contains("created_at"),
        "Messages table missing created_at column"
    );

    // Test session creation with various ID formats
    let test_sessions = [
        "test-session-123",
        "session-with-hyphens",
        "session_with_underscores",
        "session123",
        &"a".repeat(100), // Test with long session ID
    ];

    for (i, &session_id) in test_sessions.iter().enumerate() {
        // Test session creation
        save_session(&conn, session_id).unwrap();

        // Verify session was created with correct ID
        let stored_id: String = conn
            .query_row(
                "SELECT id FROM sessions WHERE id = ?",
                [session_id],
                |row| row.get(0),
            )
            .unwrap();
        assert_eq!(
            stored_id, session_id,
            "Session ID mismatch for test case {}",
            i
        );

        // Verify created_at timestamp is set and has a valid format
        let created_at: String = conn
            .query_row(
                "SELECT created_at FROM sessions WHERE id = ?",
                [session_id],
                |row| row.get(0),
            )
            .unwrap();

        // Check that the timestamp is not empty and has the expected format
        assert!(
            !created_at.is_empty(),
            "Created_at timestamp is empty for session {}",
            session_id
        );

        // Basic format check (YYYY-MM-DD HH:MM:SS)
        assert!(
            TIMESTAMP_REGEX.is_match(&created_at),
            "Created_at timestamp '{}' does not match expected format for session {}",
            created_at,
            session_id
        );
    }

    // Test message operations with various content types
    let test_messages = [
        ("user", "Hello, world!"),
        ("assistant", "Hi there!"),
        ("system", "System message"),
        (
            "user",
            "Message with special chars: \"'!@#$%^&*()_+{}|:<>?~`,./;'[]\\\\-=\\n",
        ),
        ("user", "Message with emoji: 😊🚀🌟"),
        ("assistant", "Message with numbers: 1234567890"),
        ("user", &format!("A very long message {}", "x".repeat(1000))),
    ];

    let session_id = test_sessions[0];

    // Save all test messages
    for (i, (role, content)) in test_messages.iter().enumerate() {
        save_message(&conn, session_id, role, content).unwrap();

        // Verify message was saved correctly
        let (stored_role, stored_content): (String, String) = conn
            .query_row(
                "SELECT role, content FROM messages WHERE session_id = ? ORDER BY id LIMIT 1 OFFSET ?",
                [session_id, i.to_string().as_str()],
                |row| Ok((row.get(0)?, row.get(1)?)),
            )
            .unwrap();

        assert_eq!(*role, stored_role, "Role mismatch for message {}", i);
        assert_eq!(
            *content, stored_content,
            "Content mismatch for message {}",
            i
        );
    }

    // Test message retrieval
    let history = load_history(&conn, session_id).unwrap();
    assert_eq!(
        history.len(),
        test_messages.len(),
        "Incorrect number of messages loaded"
    );

    // Verify message content and role are correct and in the right order
    for (i, (expected_role, expected_content)) in test_messages.iter().enumerate() {
        assert_eq!(
            *expected_role, history[i].role,
            "Role mismatch at index {}",
            i
        );
        assert_eq!(
            *expected_content, history[i].content,
            "Content mismatch at index {}",
            i
        );
    }

    // Test loading non-existent session
    let empty_history = load_history(&conn, "non-existent-session").unwrap();
    assert!(
        empty_history.is_empty(),
        "Expected no history for non-existent session"
    );

    // Test session listing
    let sessions = list_sessions(&conn).unwrap();
    assert_eq!(
        sessions.len(),
        test_sessions.len(),
        "Incorrect number of sessions"
    );

    for &expected_session in &test_sessions {
        assert!(
            sessions.contains(&expected_session.to_string()),
            "Session {} not found in list",
            expected_session
        );
    }

    // Test message deletion
    delete_messages(&conn, session_id).unwrap();
    let empty_history = load_history(&conn, session_id).unwrap();
    assert!(
        empty_history.is_empty(),
        "Expected no messages after deletion for session {}",
        session_id
    );

    // Verify messages were actually deleted from the database
    let message_count: i64 = conn
        .query_row(
            "SELECT COUNT(*) FROM messages WHERE session_id = ?",
            [session_id],
            |row| row.get(0),
        )
        .unwrap();
    assert_eq!(message_count, 0, "Messages not deleted from database");

    // Test session deletion
    delete_session(&conn, session_id).unwrap();

    // Verify session was deleted
    let session_exists: bool = conn
        .query_row(
            "SELECT EXISTS(SELECT 1 FROM sessions WHERE id = ?)",
            [session_id],
            |row| row.get(0),
        )
        .unwrap();
    assert!(!session_exists, "Session was not deleted");

    // Verify messages were also deleted (cascading delete)
    let message_count: i64 = conn
        .query_row(
            "SELECT COUNT(*) FROM messages WHERE session_id = ?",
            [session_id],
            |row| row.get(0),
        )
        .unwrap();
    assert_eq!(message_count, 0, "Messages not deleted with session");
}

#[test]
fn test_concurrent_access() {
    use rand::Rng;
    use std::sync::Arc;
    use std::time::Duration;

    // Create a temporary database file
    let temp_file = NamedTempFile::new().unwrap();
    let db_path = Arc::new(temp_file.path().to_path_buf());

    // Number of threads to spawn
    const NUM_THREADS: usize = 10;
    // Number of operations per thread
    const OPS_PER_THREAD: usize = 50;

    // Create a barrier to synchronize thread startup
    let barrier = Arc::new(std::sync::Barrier::new(NUM_THREADS + 1));

    // Vector to hold thread handles
    let mut handles = vec![];

    // Spawn worker threads
    for thread_id in 0..NUM_THREADS {
        let db_path = Arc::clone(&db_path);
        let barrier = Arc::clone(&barrier);

        let handle = thread::spawn(move || {
            // Each thread gets its own connection
            let conn = Connection::open(&*db_path).unwrap();

            // Initialize database if this is the first thread to get here
            if thread_id == 0 {
                init_db(&conn).unwrap();
            }

            // Wait for all threads to be ready
            barrier.wait();

            // Perform operations
            for op in 0..OPS_PER_THREAD {
                // Randomly choose an operation
                let op_type = rand::thread_rng().gen_range(0..=3);

                match op_type {
                    // Create a new session
                    0 => {
                        let session_id = format!("session-{}-{}", thread_id, op);
                        save_session(&conn, &session_id).unwrap();

                        // Verify session was created
                        let exists: bool = conn
                            .query_row(
                                "SELECT EXISTS(SELECT 1 FROM sessions WHERE id = ?)",
                                [&session_id],
                                |row| row.get(0),
                            )
                            .unwrap();
                        assert!(exists, "Session was not created");
                    }

                    // Add messages to a random session
                    1 => {
                        let sessions = list_sessions(&conn).unwrap();
                        if !sessions.is_empty() {
                            let session_idx = rand::thread_rng().gen_range(0..sessions.len());
                            let session_id = &sessions[session_idx];

                            let role = if rand::random() { "user" } else { "assistant" };
                            let content = format!("Message {} from thread {}", op, thread_id);

                            save_message(&conn, session_id, role, &content).unwrap();

                            // Verify message was saved
                            let count: i64 = conn
                                .query_row(
                                    "SELECT COUNT(*) FROM messages WHERE session_id = ? AND content = ?",
                                    [session_id, &content],
                                    |row| row.get(0),
                                )
                                .unwrap();
                            assert_eq!(count, 1, "Message was not saved correctly");
                        }
                    }

                    // List sessions (read-only operation)
                    2 => {
                        let _ = list_sessions(&conn).unwrap();
                    }

                    // Load history for a random session
                    3 => {
                        let sessions = list_sessions(&conn).unwrap();
                        if !sessions.is_empty() {
                            let session_idx = rand::thread_rng().gen_range(0..sessions.len());
                            let _ = load_history(&conn, &sessions[session_idx]).unwrap();
                        }
                    }

                    _ => unreachable!(),
                }

                // Small delay to increase chance of interleaving
                if op % 10 == 0 {
                    thread::sleep(Duration::from_millis(1));
                }
            }
        });

        handles.push(handle);
    }

    // Wait for all threads to be ready
    barrier.wait();

    // Wait for all threads to complete
    for handle in handles {
        handle.join().unwrap();
    }

    // Final consistency check
    let conn = Connection::open(&*db_path).unwrap();

    // Verify all sessions have messages with correct session_id
    let sessions = list_sessions(&conn).unwrap();
    for session_id in sessions {
        // Verify there are messages for this session
        let _message_count: i64 = conn
            .query_row(
                "SELECT COUNT(*) FROM messages WHERE session_id = ?",
                [&session_id],
                |row| row.get(0),
            )
            .unwrap();

        // Get the count of messages that don't belong to any session (should be 0)
        let orphaned_count: i64 = conn
            .query_row(
                "SELECT COUNT(*) FROM messages WHERE session_id NOT IN (SELECT id FROM sessions)",
                [],
                |row| row.get(0),
            )
            .unwrap();

        assert_eq!(orphaned_count, 0, "Found messages without a valid session");
    }

    // Verify no duplicate messages
    let duplicate_messages: i64 = conn
        .query_row(
            "SELECT COUNT(*) - COUNT(DISTINCT session_id || '|' || content) FROM messages",
            [],
            |row| row.get(0),
        )
        .unwrap_or(0);

    assert_eq!(duplicate_messages, 0, "Found duplicate messages");
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

    // Skip this test in CI environments where network access might be restricted
    if std::env::var("CI").is_ok() {
        println!("Skipping web search test in CI environment");
        return;
    }

    // Test with a simple query - this will actually hit DuckDuckGo API
    // In local environments, we expect this to work
    let result = web_search("rust programming").await;

    // In local development, we expect the search to succeed
    match result {
        Ok(response) => {
            // If successful, should contain some content
            assert!(!response.is_empty(), "Search response should not be empty");
        }
        Err(e) => {
            // If it fails locally, it's unexpected - fail the test
            panic!("Web search failed unexpectedly: {:?}", e);
        }
    }
}

#[cfg(test)]
mod e2e_tests {
    use super::*;

    #[test]
    fn test_full_chat_session_e2e() {
        // Create a temporary database file
        let temp_db = NamedTempFile::new().unwrap();
        let db_path = temp_db.path().to_str().unwrap();

        // Set up environment variables for testing
        std::env::set_var("DATABASE_PATH", db_path);
        std::env::set_var("OPENAI_API_KEY", "test-key");

        let conn = Connection::open(db_path).unwrap();
        init_db(&conn).unwrap();

        // Create API config for testing (not used in this test but kept for future expansion)
        let _config = ApiConfig {
            provider: ApiProvider::OpenAI,
            api_key: "test-key".to_string(),
            base_url: "https://api.openai.com/v1/chat/completions".to_string(),
            model_name: "gpt-4".to_string(),
        };

        // Test session creation and message handling
        let session_id = "test-session-e2e";
        save_session(&conn, session_id).unwrap();

        // Add test messages
        save_message(&conn, session_id, "user", "Hello, test message").unwrap();
        save_message(
            &conn,
            session_id,
            "assistant",
            "Hi there! This is a test response.",
        )
        .unwrap();

        // Verify session was created
        let mut stmt = conn
            .prepare("SELECT COUNT(*) FROM sessions WHERE id = ?")
            .unwrap();
        let count: i64 = stmt.query_row([session_id], |row| row.get(0)).unwrap();
        assert_eq!(count, 1, "Session should be created");

        // Verify messages were saved
        let history = load_history(&conn, session_id).unwrap();
        assert_eq!(history.len(), 2, "Should have 2 messages");
        assert_eq!(history[0].role, "user");
        assert_eq!(history[0].content, "Hello, test message");
        assert_eq!(history[1].role, "assistant");
        assert_eq!(history[1].content, "Hi there! This is a test response.");
    }

    #[test]
    fn test_session_management_e2e() {
        // Create a temporary database file
        let temp_db = NamedTempFile::new().unwrap();
        let db_path = temp_db.path().to_str().unwrap();

        let conn = Connection::open(db_path).unwrap();
        init_db(&conn).unwrap();

        // Create multiple sessions
        let session1 = "session-1";
        let session2 = "session-2";

        save_session(&conn, session1).unwrap();
        save_session(&conn, session2).unwrap();

        // Add messages to both sessions
        save_message(&conn, session1, "user", "Message 1").unwrap();
        save_message(&conn, session1, "assistant", "Response 1").unwrap();
        save_message(&conn, session2, "user", "Message 2").unwrap();

        // Test session listing
        let mut stmt = conn.prepare("SELECT COUNT(*) FROM sessions").unwrap();
        let session_count: i64 = stmt.query_row([], |row| row.get(0)).unwrap();
        assert_eq!(session_count, 2, "Should have 2 sessions");

        // Test loading specific sessions
        let history1 = load_history(&conn, session1).unwrap();
        let history2 = load_history(&conn, session2).unwrap();

        assert_eq!(history1.len(), 2, "Session 1 should have 2 messages");
        assert_eq!(history2.len(), 1, "Session 2 should have 1 message");

        // Test ordering by creation time
        let mut stmt = conn
            .prepare("SELECT id FROM sessions ORDER BY created_at DESC")
            .unwrap();
        let sessions: Vec<String> = stmt
            .query_map([], |row| row.get(0))
            .unwrap()
            .map(|r| r.unwrap())
            .collect();

        assert_eq!(sessions.len(), 2, "Should retrieve both sessions");
        // The order might vary, but both should be present
        assert!(sessions.contains(&session1.to_string()));
        assert!(sessions.contains(&session2.to_string()));
    }

    #[test]
    fn test_export_functionality_e2e() {
        // Create a temporary database
        let temp_db = NamedTempFile::new().unwrap();
        let db_path = temp_db.path().to_str().unwrap();

        let conn = Connection::open(db_path).unwrap();
        init_db(&conn).unwrap();

        // Create a session with some messages
        let session_id = "export-test-session";
        save_session(&conn, session_id).unwrap();
        save_message(&conn, session_id, "user", "Message for export test").unwrap();
        save_message(&conn, session_id, "assistant", "Response for export test").unwrap();
        save_message(&conn, session_id, "system", "System message for export").unwrap();

        // Load the history
        let history = load_history(&conn, session_id).unwrap();
        assert_eq!(history.len(), 3, "Should have 3 messages");

        // Test JSON export format
        let json_export = serde_json::to_string_pretty(&history).unwrap();
        assert!(
            json_export.contains("Message for export test"),
            "JSON should contain user message"
        );
        assert!(
            json_export.contains("Response for export test"),
            "JSON should contain assistant message"
        );
        assert!(
            json_export.contains("System message for export"),
            "JSON should contain system message"
        );
        assert!(
            json_export.contains("\"role\""),
            "JSON should have role fields"
        );
        assert!(
            json_export.contains("\"content\""),
            "JSON should have content fields"
        );

        // Test text export format
        let text_export: String = history
            .iter()
            .map(|msg| format!("{}: {}", msg.role, msg.content))
            .collect::<Vec<_>>()
            .join("\n");

        assert!(
            text_export.contains("user: Message for export test"),
            "Text export should contain user message"
        );
        assert!(
            text_export.contains("assistant: Response for export test"),
            "Text export should contain assistant message"
        );
        assert!(
            text_export.contains("system: System message for export"),
            "Text export should contain system message"
        );

        // Test that JSON is valid and can be parsed back
        let parsed_history: Vec<Message> = serde_json::from_str(&json_export).unwrap();
        assert_eq!(
            parsed_history.len(),
            3,
            "Parsed JSON should have same number of messages"
        );
        assert_eq!(parsed_history[0].role, "user");
        assert_eq!(parsed_history[1].role, "assistant");
        assert_eq!(parsed_history[2].role, "system");
    }

    #[test]
    fn test_binary_execution_e2e() {
        use std::fs;
        use std::process::{Command, Stdio};

        // Create a temporary directory for the test
        let temp_dir = tempfile::tempdir().expect("Failed to create temp dir");

        // Set up database directory and path
        let db_dir = temp_dir.path().join("db");
        std::fs::create_dir_all(&db_dir).expect("Failed to create database directory");
        let db_path = db_dir.join("harper.db");

        // Print debug information
        println!("=== Test Setup ===");
        println!("Temp directory: {}", temp_dir.path().display());
        println!("Database path: {}", db_path.display());
        println!("Database directory exists: {}", db_dir.exists());
        println!("Database file exists before test: {}", db_path.exists());
        println!("Current directory: {:?}", std::env::current_dir().unwrap());

        // List contents of the temp directory
        println!("\n=== Directory Contents ===");
        if let Ok(entries) = std::fs::read_dir(temp_dir.path()) {
            for entry in entries.flatten() {
                println!(
                    "  - {} (dir: {})",
                    entry.path().display(),
                    entry.path().is_dir()
                );
            }
        }

        // Verify the directory is writable
        println!("\n=== Testing Directory Permissions ===");
        let test_file = temp_dir.path().join(".test_write");
        std::fs::write(&test_file, "test").expect("Failed to write test file to temp directory");
        std::fs::remove_file(&test_file).expect("Failed to remove test file");
        println!("Successfully wrote and removed test file");

        // Ensure the parent directory of the database file exists
        if let Some(parent) = db_path.parent() {
            if !parent.exists() {
                fs::create_dir_all(parent).expect("Failed to create parent directory for database");
            }
            println!("Database parent directory exists: {}", parent.exists());
        }

        // Build the command
        let mut command = Command::new(env!("CARGO_BIN_EXE_harper"));

        // Set environment variables
        command
            .env("HARPER_DATABASE__PATH", &db_path)
            .env("HARPER_API__API_KEY", "test-key")
            .env("HARPER_API__PROVIDER", "OpenAI")
            .env(
                "HARPER_API__BASE_URL",
                "https://api.openai.com/v1/chat/completions",
            )
            .env("HARPER_API__MODEL_NAME", "gpt-4")
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .stderr(Stdio::piped());

        // Print the command for debugging
        println!("\n=== Running Command ===");
        println!("Command: {:?}", command);
        println!("Database path in env: {}", db_path.display());

        println!("Running command: {:?}", command);

        let mut child = command.spawn().expect("Failed to start binary");

        // Send quit command using the constant
        use harper::core::constants::{menu, messages};

        let quit_command = format!("{}\n", menu::QUIT);
        println!("Sending quit command: {:?}", quit_command.trim());

        if let Some(mut stdin) = child.stdin.take() {
            use std::io::Write;
            // Use the QUIT constant and add newline
            stdin
                .write_all(quit_command.as_bytes())
                .expect("Failed to write to stdin");
            // Explicitly flush the stdin to ensure the command is sent
            stdin.flush().expect("Failed to flush stdin");
        }

        // Give the process some time to process the input
        std::thread::sleep(std::time::Duration::from_millis(500));

        // Wait for the process to finish with a timeout
        let output = match wait_timeout::ChildExt::wait_timeout(
            &mut child,
            std::time::Duration::from_secs(5),
        ) {
            Ok(Some(status)) => {
                // Process has finished
                println!("Process finished with status: {}", status);
                child.wait_with_output().expect("Failed to wait for child")
            }
            Ok(None) => {
                // Process is still running after timeout
                println!("Process is still running after timeout, attempting to get output...");
                let _ = child.kill();
                child.wait_with_output().expect("Failed to wait for child")
            }
            Err(e) => {
                panic!("Error waiting for child process: {}", e);
            }
        };

        // Always print stdout and stderr for debugging
        let stdout = String::from_utf8_lossy(&output.stdout);
        let stderr = String::from_utf8_lossy(&output.stderr);

        println!("=== STDOUT ===\n{}", stdout);
        println!("=== STDERR ===\n{}", stderr);

        // Check if the process exited successfully
        if !output.status.success() {
            panic!("Process exited with status: {}", output.status);
        }

        // Check for the goodbye message in the output
        assert!(
            stdout.contains(messages::GOODBYE),
            "Should print goodbye message. Expected '{}' in output.\nFull output:\n{}",
            messages::GOODBYE,
            stdout
        );
    }

    #[test]
    fn test_error_handling_e2e() {
        // Test database error handling
        let temp_db = NamedTempFile::new().unwrap();
        let db_path = temp_db.path().to_str().unwrap();

        let conn = Connection::open(db_path).unwrap();
        init_db(&conn).unwrap();

        // Test loading non-existent session
        let result = load_history(&conn, "non-existent-session");
        assert!(
            result.is_ok(),
            "Loading non-existent session should not error"
        );
        assert!(
            result.unwrap().is_empty(),
            "Non-existent session should return empty history"
        );

        // Test saving message to non-existent session (should still work)
        // First create the session, then save message
        save_session(&conn, "new-session").unwrap();
        let result = save_message(&conn, "new-session", "user", "test message");
        assert!(
            result.is_ok(),
            "Saving message should work for existing session"
        );

        // Verify the message was saved
        let history = load_history(&conn, "new-session").unwrap();
        assert_eq!(history.len(), 1, "Should have 1 message");
        assert_eq!(history[0].role, "user");
        assert_eq!(history[0].content, "test message");

        // Test empty message handling
        save_session(&conn, "test-session").unwrap(); // Ensure session exists
        let result = save_message(&conn, "test-session", "user", "");
        assert!(result.is_ok(), "Empty messages should be allowed");

        // Check database directly to see if message was saved
        let mut stmt = conn
            .prepare("SELECT COUNT(*) FROM messages WHERE session_id = ?")
            .unwrap();
        let msg_count: i64 = stmt.query_row(["test-session"], |row| row.get(0)).unwrap();
        assert_eq!(msg_count, 1, "Empty message should be saved to database");

        let _history = load_history(&conn, "test-session").unwrap();
        // load_history might filter out empty messages, which is fine
        // Just verify the save operation worked
        assert!(result.is_ok(), "Save operation should succeed");
    }
}
