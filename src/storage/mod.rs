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
            created_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
            FOREIGN KEY(session_id) REFERENCES sessions(id)
        )",
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
