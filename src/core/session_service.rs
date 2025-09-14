//! Session management service
//!
//! This module provides functionality for managing chat sessions,
//! including listing, viewing, and exporting sessions.

use crate::core::error::HarperResult;
use crate::load_history;
use colored::*;
use rusqlite::Connection;
use serde_json;
use std::fs::File;
use std::io::{self, Write};

/// Service for managing chat sessions
pub struct SessionService<'a> {
    conn: &'a Connection,
}

impl<'a> SessionService<'a> {
    /// Create a new session service
    pub fn new(conn: &'a Connection) -> Self {
        Self { conn }
    }

    /// List all previous sessions
    pub fn list_sessions(&self) -> HarperResult<()> {
        let mut stmt = self
            .conn
            .prepare("SELECT id, created_at FROM sessions ORDER BY created_at DESC")?;
        let rows = stmt.query_map([], |row| {
            Ok((row.get::<_, String>(0)?, row.get::<_, String>(1)?))
        })?;

        println!("{}", "Previous Sessions:".bold().yellow());
        for (i, row) in rows.enumerate() {
            let (id, created_at) = row?;
            println!("{}: {} ({})", i + 1, id, created_at);
        }
        Ok(())
    }

    /// View a specific session's history
    pub fn view_session(&self) -> HarperResult<()> {
        print!("Enter session ID to view: ");
        io::stdout().flush()?;
        let mut session_id = String::new();
        io::stdin().read_line(&mut session_id)?;
        let session_id = session_id.trim();

        let history = load_history(self.conn, session_id).unwrap_or_default();
        let total_messages = history.len();

        // Show only the last 20 messages to prevent overwhelming output
        const MAX_DISPLAY: usize = 20;
        let display_start = total_messages.saturating_sub(MAX_DISPLAY);

        println!(
            "\n{} (showing last {} of {} messages)\n",
            "Session History:".bold().yellow(),
            total_messages.saturating_sub(display_start),
            total_messages
        );

        for msg in &history[display_start..] {
            match msg.role.as_str() {
                "user" => println!("{} {}", "You:".bold().blue(), msg.content.blue()),
                "assistant" => println!("{} {}", "Assistant:".bold().green(), msg.content.green()),
                "system" => println!("{} {}", "System:".bold().magenta(), msg.content.magenta()),
                _ => println!("{}: {}", msg.role, msg.content),
            }
        }

        if total_messages > MAX_DISPLAY {
            println!(
                "\n{} Use export to view the full transcript.",
                "Note:".bold().cyan()
            );
        }

        Ok(())
    }

    /// Export a session's history to a file
    pub fn export_session(&self) -> HarperResult<()> {
        print!("Enter session ID to export: ");
        io::stdout().flush()?;
        let mut session_id = String::new();
        io::stdin().read_line(&mut session_id)?;
        let session_id = session_id.trim();

        print!("Export format (txt/json) [txt]: ");
        io::stdout().flush()?;
        let mut format_choice = String::new();
        io::stdin().read_line(&mut format_choice)?;
        let format_choice = format_choice.trim().to_lowercase();
        let is_json = format_choice == "json";

        let history = load_history(self.conn, session_id).unwrap_or_default();

        if history.is_empty() {
            println!(
                "No messages found for session '{}' to export.",
                session_id.bold().red()
            );
            return Ok(());
        }

        let extension = if is_json { "json" } else { "txt" };
        let filename = format!("session_{}.{}", session_id, extension);
        let mut file = File::create(&filename)?;

        if is_json {
            let json = serde_json::to_string_pretty(&history)?;
            file.write_all(json.as_bytes())?;
        } else {
            for msg in &history {
                let line = format!("{}: {}\n", msg.role, msg.content);
                file.write_all(line.as_bytes())?;
            }
        }

        println!("Session exported to {}", filename.bold().yellow());
        Ok(())
    }
}
