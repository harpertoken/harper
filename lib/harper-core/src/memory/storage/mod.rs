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

//! Database storage operations for conversation persistence
//!
//! This module provides functions for storing and retrieving chat sessions
//! and messages using SQLite as the backend.

use crate::core::error::HarperResult;
use crate::core::Message;
use rusqlite::{params, Connection};

/// Initialize the database schema
///
/// Creates the necessary tables for storing sessions and messages if they don't exist.
///
/// # Arguments
/// * `conn` - SQLite database connection
///
/// # Errors
/// Returns `HarperError::Database` if table creation fails
pub fn init_db(conn: &Connection) -> HarperResult<()> {
    // Enable WAL mode for better concurrent access
    conn.execute_batch("PRAGMA journal_mode=WAL;")?;
    // Enable foreign key constraints
    conn.execute_batch("PRAGMA foreign_keys = ON;")?;

    // Migration: Drop tables with old foreign key constraints
    for table_name in &["messages", "command_logs"] {
        let has_table: bool = conn
            .query_row(
                "SELECT COUNT(*) FROM sqlite_master WHERE type='table' AND name=?1",
                [table_name],
                |row| row.get::<_, i32>(0),
            )
            .unwrap_or(0)
            > 0;

        if has_table {
            let has_fk = conn
                .query_row(
                    "SELECT COUNT(*) FROM pragma_foreign_key_list(?1)",
                    [table_name],
                    |row| row.get::<_, i32>(0),
                )
                .unwrap_or(0)
                > 0;

            if has_fk {
                if *table_name == "messages" {
                    conn.execute("DROP TABLE IF EXISTS messages", [])?;
                } else if *table_name == "command_logs" {
                    conn.execute("DROP TABLE IF EXISTS command_logs", [])?;
                }
            }
        }
    }

    conn.execute(
        "CREATE TABLE IF NOT EXISTS sessions (
            id TEXT PRIMARY KEY,
            created_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP
        )",
        [],
    )?;
    conn.execute(
        "CREATE TABLE IF NOT EXISTS messages (
             id INTEGER PRIMARY KEY AUTOINCREMENT,
             session_id TEXT,
             role TEXT,
             content TEXT,
             created_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP
         )",
        [],
    )?;
    conn.execute(
        "CREATE TABLE IF NOT EXISTS todos (
             id INTEGER PRIMARY KEY AUTOINCREMENT,
             description TEXT NOT NULL,
             created_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP
         )",
        [],
    )?;
    conn.execute(
        "CREATE TABLE IF NOT EXISTS command_logs (
             id INTEGER PRIMARY KEY AUTOINCREMENT,
             session_id TEXT,
             command TEXT NOT NULL,
             source TEXT NOT NULL,
             requires_approval INTEGER NOT NULL,
             approved INTEGER NOT NULL,
             status TEXT NOT NULL,
             exit_code INTEGER,
             duration_ms INTEGER,
             stdout_preview TEXT,
             stderr_preview TEXT,
             error_message TEXT,
             created_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP
         )",
        [],
    )?;
    conn.execute(
        "CREATE INDEX IF NOT EXISTS idx_command_logs_session_id ON command_logs(session_id)",
        [],
    )?;
    Ok(())
}

/// Save a message to the database
///
/// Stores a message in the database associated with a specific session.
///
/// # Arguments
/// * `conn` - SQLite database connection
/// * `session_id` - Unique identifier for the chat session
/// * `role` - Role of the message sender (user, assistant, system)
/// * `content` - The message content
///
/// # Errors
/// Returns `HarperError::Database` if the insert operation fails
pub fn save_message(
    conn: &Connection,
    session_id: &str,
    role: &str,
    content: &str,
) -> HarperResult<()> {
    conn.execute(
        "INSERT INTO messages (session_id, role, content) VALUES (?1, ?2, ?3)",
        params![session_id, role, content],
    )?;
    Ok(())
}

/// Save a session to the database
///
/// Creates a new session record in the database.
///
/// # Arguments
/// * `conn` - SQLite database connection
/// * `session_id` - Unique identifier for the chat session
///
/// # Errors
/// Returns `HarperError::Database` if the insert operation fails
pub fn save_session(conn: &Connection, session_id: &str) -> HarperResult<()> {
    conn.execute(
        "INSERT OR IGNORE INTO sessions (id) VALUES (?1)",
        params![session_id],
    )?;
    Ok(())
}

/// Load conversation history for a session
///
/// Retrieves all messages for a given session from the database,
/// ordered by creation time.
///
/// # Arguments
/// * `conn` - SQLite database connection
/// * `session_id` - Unique identifier for the chat session
///
/// # Returns
/// A vector of `Message` structs representing the conversation history
///
/// # Errors
/// Returns `HarperError::Database` if the query fails
pub fn load_history(conn: &Connection, session_id: &str) -> HarperResult<Vec<Message>> {
    let mut stmt =
        conn.prepare("SELECT role, content FROM messages WHERE session_id = ?1 ORDER BY id ASC")?;
    let rows = stmt.query_map(params![session_id], |row| {
        Ok(Message {
            role: row.get(0)?,
            content: row.get(1)?,
        })
    })?;

    let mut messages = Vec::new();
    for message in rows {
        messages.push(message?);
    }
    Ok(messages)
}

/// List all session IDs in the database
///
/// Retrieves all session IDs from the sessions table.
///
/// # Arguments
/// * `conn` - SQLite database connection
///
/// # Returns
/// A vector of session IDs as strings
///
/// # Errors
/// Returns `HarperError::Database` if the query fails
#[allow(dead_code)]
pub fn list_sessions(conn: &Connection) -> HarperResult<Vec<String>> {
    let mut stmt = conn.prepare("SELECT id FROM sessions ORDER BY created_at DESC")?;
    let rows = stmt.query_map([], |row| row.get(0))?;

    let mut sessions = Vec::new();
    for session in rows {
        sessions.push(session?);
    }
    Ok(sessions)
}

/// Delete all messages for a specific session
///
/// # Arguments
/// * `conn` - SQLite database connection
/// * `session_id` - ID of the session to delete messages for
///
/// # Errors
/// Returns `HarperError::Database` if the delete operation fails
#[allow(dead_code)]
pub fn delete_messages(conn: &Connection, session_id: &str) -> HarperResult<()> {
    conn.execute("DELETE FROM messages WHERE session_id = ?", [session_id])?;
    Ok(())
}

/// Delete a session and all its messages
///
/// # Arguments
/// * `conn` - SQLite database connection
/// * `session_id` - ID of the session to delete
///
/// # Errors
/// Returns `HarperError::Database` if the delete operation fails
#[allow(dead_code)]
pub fn delete_session(conn: &Connection, session_id: &str) -> HarperResult<()> {
    // First delete all messages for this session
    delete_messages(conn, session_id)?;

    // Then delete the session itself
    conn.execute("DELETE FROM sessions WHERE id = ?", [session_id])?;

    Ok(())
}

/// Save a todo to the database
///
/// # Arguments
/// * `conn` - SQLite database connection
/// * `description` - The todo description
///
/// # Errors
/// Returns `HarperError::Database` if the insert operation fails
pub fn save_todo(conn: &Connection, description: &str) -> HarperResult<()> {
    conn.execute(
        "INSERT INTO todos (description) VALUES (?1)",
        params![description],
    )?;
    Ok(())
}

/// Load all todos from the database
///
/// # Arguments
/// * `conn` - SQLite database connection
///
/// # Returns
/// A vector of (id, description) tuples
///
/// # Errors
/// Returns `HarperError::Database` if the query fails
pub fn load_todos(conn: &Connection) -> HarperResult<Vec<(i64, String)>> {
    let mut stmt = conn.prepare("SELECT id, description FROM todos ORDER BY id ASC")?;
    let rows = stmt.query_map([], |row| Ok((row.get(0)?, row.get(1)?)))?;

    let todos = rows.collect::<Result<Vec<_>, _>>()?;
    Ok(todos)
}

/// Delete a todo by ID
///
/// # Arguments
/// * `conn` - SQLite database connection
/// * `id` - The todo ID to delete
///
/// # Errors
/// Returns `HarperError::Database` if the delete operation fails
pub fn delete_todo(conn: &Connection, id: i64) -> HarperResult<()> {
    conn.execute("DELETE FROM todos WHERE id = ?", [id])?;
    Ok(())
}

/// Clear all todos
///
/// # Arguments
/// * `conn` - SQLite database connection
///
/// # Returns
/// Returns the number of todos that were cleared
///
/// # Errors
/// Returns `HarperError::Database` if the delete operation fails
pub fn clear_todos(conn: &Connection) -> HarperResult<usize> {
    let count = conn.execute("DELETE FROM todos", [])?;
    Ok(count)
}

/// Persisted record of a command execution attempt
#[derive(Debug, Clone)]
pub struct CommandLogRecord {
    pub session_id: Option<String>,
    pub command: String,
    pub source: String,
    pub requires_approval: bool,
    pub approved: bool,
    pub status: String,
    pub exit_code: Option<i32>,
    pub duration_ms: Option<i64>,
    pub stdout_preview: Option<String>,
    pub stderr_preview: Option<String>,
    pub error_message: Option<String>,
}

impl CommandLogRecord {
    #[allow(clippy::too_many_arguments)]
    /// Helper constructor for building log entries from borrowed data
    pub fn new(
        session_id: Option<&str>,
        command: &str,
        source: &str,
        requires_approval: bool,
        approved: bool,
        status: &str,
        exit_code: Option<i32>,
        duration_ms: Option<i64>,
        stdout_preview: Option<String>,
        stderr_preview: Option<String>,
        error_message: Option<String>,
    ) -> Self {
        Self {
            session_id: session_id.map(|s| s.to_string()),
            command: command.to_string(),
            source: source.to_string(),
            requires_approval,
            approved,
            status: status.to_string(),
            exit_code,
            duration_ms,
            stdout_preview,
            stderr_preview,
            error_message,
        }
    }
}

/// Insert a command log entry into the audit table
pub fn insert_command_log(conn: &Connection, record: &CommandLogRecord) -> HarperResult<()> {
    conn.execute(
        "INSERT INTO command_logs (
             session_id,
             command,
             source,
             requires_approval,
             approved,
             status,
             exit_code,
             duration_ms,
             stdout_preview,
             stderr_preview,
             error_message
         ) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11)",
        params![
            record.session_id,
            record.command,
            record.source,
            record.requires_approval as i32,
            record.approved as i32,
            record.status,
            record.exit_code,
            record.duration_ms,
            record.stdout_preview,
            record.stderr_preview,
            record.error_message,
        ],
    )?;
    Ok(())
}

/// Simplified view of an audit record for presentation
#[derive(Debug, Clone)]
pub struct CommandLogEntry {
    pub command: String,
    pub source: String,
    pub requires_approval: bool,
    pub approved: bool,
    pub status: String,
    pub exit_code: Option<i32>,
    pub duration_ms: Option<i64>,
    pub created_at: String,
}

/// Load recent command logs for a given session
pub fn load_command_logs_for_session(
    conn: &Connection,
    session_id: &str,
    limit: usize,
) -> HarperResult<Vec<CommandLogEntry>> {
    let mut stmt = conn.prepare(
        "SELECT command,
                source,
                requires_approval,
                approved,
                status,
                exit_code,
                duration_ms,
                created_at
         FROM command_logs
         WHERE session_id = ?1
         ORDER BY id DESC
         LIMIT ?2",
    )?;

    let rows = stmt.query_map(params![session_id, limit as i64], |row| {
        Ok(CommandLogEntry {
            command: row.get(0)?,
            source: row.get(1)?,
            requires_approval: row.get::<_, i32>(2)? != 0,
            approved: row.get::<_, i32>(3)? != 0,
            status: row.get(4)?,
            exit_code: row.get(5)?,
            duration_ms: row.get(6)?,
            created_at: row.get(7)?,
        })
    })?;

    let mut entries = Vec::new();
    for row in rows {
        entries.push(row?);
    }
    Ok(entries)
}

/// Fetch the most recent command log entry for a session
pub fn load_latest_command_log(
    conn: &Connection,
    session_id: &str,
) -> HarperResult<Option<CommandLogEntry>> {
    let mut stmt = conn.prepare(
        "SELECT command,
                source,
                requires_approval,
                approved,
                status,
                exit_code,
                duration_ms,
                created_at
         FROM command_logs
         WHERE session_id = ?1
         ORDER BY id DESC
         LIMIT 1",
    )?;

    let result = stmt.query_row(params![session_id], |row| {
        Ok(CommandLogEntry {
            command: row.get(0)?,
            source: row.get(1)?,
            requires_approval: row.get::<_, i32>(2)? != 0,
            approved: row.get::<_, i32>(3)? != 0,
            status: row.get(4)?,
            exit_code: row.get(5)?,
            duration_ms: row.get(6)?,
            created_at: row.get(7)?,
        })
    });

    match result {
        Ok(entry) => Ok(Some(entry)),
        Err(rusqlite::Error::QueryReturnedNoRows) => Ok(None),
        Err(e) => Err(e.into()),
    }
}
