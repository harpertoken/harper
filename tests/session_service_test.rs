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

use harper::core::error::HarperResult;
use harper::core::io_traits::{Input, Output};
use harper::memory::session_service::SessionService;
use rusqlite::Connection;
use tempfile::NamedTempFile;

// Mock implementations for testing
#[derive(Clone)]
struct MockInput {
    responses: std::sync::Arc<std::sync::Mutex<std::collections::VecDeque<String>>>,
}

impl MockInput {
    fn new(responses: Vec<&str>) -> Self {
        Self {
            responses: std::sync::Arc::new(std::sync::Mutex::new(
                responses.into_iter().map(String::from).collect(),
            )),
        }
    }
}

impl Input for MockInput {
    fn read_line(&self) -> std::io::Result<String> {
        let mut responses = self.responses.lock().unwrap();
        responses.pop_front().ok_or_else(|| {
            std::io::Error::new(std::io::ErrorKind::UnexpectedEof, "No more test responses")
        })
    }
}

#[derive(Clone)]
struct MockOutput {
    output: std::sync::Arc<std::sync::Mutex<Vec<String>>>,
}

impl MockOutput {
    fn new() -> Self {
        Self {
            output: std::sync::Arc::new(std::sync::Mutex::new(Vec::new())),
        }
    }

    fn get_output(&self) -> String {
        self.output.lock().unwrap().join("")
    }
}

impl Output for MockOutput {
    fn print(&self, text: &str) -> std::io::Result<()> {
        self.output.lock().unwrap().push(text.to_string());
        Ok(())
    }

    fn println(&self, text: &str) -> std::io::Result<()> {
        self.output.lock().unwrap().push(format!(
            "{}
",
            text
        ));
        Ok(())
    }

    fn flush(&self) -> std::io::Result<()> {
        Ok(())
    }
}

#[test]
fn test_list_sessions_empty() -> HarperResult<()> {
    // Setup test database
    let temp_file = NamedTempFile::new()?;
    let conn = Connection::open(temp_file.path())?;

    // Initialize database schema
    conn.execute(
        "CREATE TABLE IF NOT EXISTS sessions (
            id TEXT PRIMARY KEY,
            created_at DATETIME DEFAULT CURRENT_TIMESTAMP
        )",
        [],
    )?;

    // Setup mock I/O
    let input = MockInput::new(vec![]);
    let output = MockOutput::new();
    let output_clone = output.clone();

    // Create service with mock I/O
    let service = SessionService::with_io(&conn, input, output);

    // Test list_sessions with empty database
    let result = service.list_sessions();
    assert!(result.is_ok());

    // Verify output
    let output_str = output_clone.get_output();
    assert!(output_str.contains("Previous Sessions:"));

    Ok(())
}

#[test]
fn test_view_nonexistent_session() -> HarperResult<()> {
    let temp_file = NamedTempFile::new()?;
    let conn = Connection::open(temp_file.path())?;

    // Initialize database schema
    conn.execute(
        "CREATE TABLE IF NOT EXISTS sessions (
            id TEXT PRIMARY KEY,
            created_at DATETIME DEFAULT CURRENT_TIMESTAMP
        )",
        [],
    )?;

    // Setup mock I/O
    let input = MockInput::new(vec!["nonexistent-session-id"]);
    let output = MockOutput::new();
    let output_clone = output.clone();

    // Create service with mock I/O
    let service = SessionService::with_io(&conn, input, output);

    // Test viewing non-existent session
    let result = service.view_session();
    assert!(result.is_ok()); // Should not fail, just show no messages

    // Verify output
    let output_str = output_clone.get_output();
    assert!(output_str.contains("Session History:"));
    assert!(output_str.contains("showing last 0 of 0 messages"));

    Ok(())
}

#[test]
fn test_export_nonexistent_session() -> HarperResult<()> {
    let temp_file = NamedTempFile::new()?;
    let conn = Connection::open(temp_file.path())?;

    // Initialize database schema
    conn.execute(
        "CREATE TABLE IF NOT EXISTS sessions (
            id TEXT PRIMARY KEY,
            created_at DATETIME DEFAULT CURRENT_TIMESTAMP
        )",
        [],
    )?;

    // Setup mock I/O
    let input = MockInput::new(vec!["nonexistent-session-id", "txt", ""]);
    let output = MockOutput::new();
    let output_clone = output.clone();

    // Create service with mock I/O
    let service = SessionService::with_io(&conn, input, output);

    // Test exporting non-existent session
    let result = service.export_session();
    assert!(result.is_ok()); // Should not fail, just show message

    // Verify output
    let output_str = output_clone.get_output();
    assert!(output_str.contains("No history found for session"));

    Ok(())
}
