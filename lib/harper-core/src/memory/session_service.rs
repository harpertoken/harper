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

//! Session management service
//!
//! This module provides functionality for managing chat sessions,
//! including listing, viewing, and exporting sessions.

use crate::core::error::{HarperError, HarperResult};
use crate::core::io_traits::{Input, Output};
use crate::memory::storage::load_history;
use chrono::Local;
use colored::*;
use rusqlite::Connection;
use serde_json;
use std::fs::File;
use std::io::Write;
use std::sync::Arc;

#[derive(Debug, Clone)]
pub struct Session {
    pub id: String,
    pub created_at: String,
}

/// Service for managing chat sessions
pub struct SessionService<'a> {
    conn: &'a Connection,
    input: Arc<dyn Input>,
    output: Arc<dyn Output>,
}

impl<'a> SessionService<'a> {
    /// Create a new session service
    pub fn new(conn: &'a Connection) -> Self {
        Self::with_io(
            conn,
            crate::core::io_traits::StdInput,
            crate::core::io_traits::StdOutput,
        )
    }

    /// Create a new session service with custom I/O (for testing)
    pub fn with_io<I, O>(conn: &'a Connection, input: I, output: O) -> Self
    where
        I: Input + 'static,
        O: Output + 'static,
    {
        Self {
            conn,
            input: Arc::new(input),
            output: Arc::new(output),
        }
    }

    /// List all previous sessions (returns data)
    pub fn list_sessions_data(&self) -> HarperResult<Vec<Session>> {
        let mut stmt = self
            .conn
            .prepare("SELECT id, created_at FROM sessions ORDER BY created_at DESC")?;
        let rows = stmt.query_map([], |row| {
            Ok(Session {
                id: row.get(0)?,
                created_at: row.get(1)?,
            })
        })?;
        let sessions = rows.collect::<Result<Vec<_>, _>>()?;
        Ok(sessions)
    }

    /// List all previous sessions
    pub fn list_sessions(&self) -> HarperResult<()> {
        let sessions = self.list_sessions_data()?;
        self.output.println(&"Previous Sessions:".bold().yellow())?;
        for (i, session) in sessions.iter().enumerate() {
            self.output.println(&format!(
                "{}: {} ({})",
                i + 1,
                session.id,
                session.created_at
            ))?;
        }
        Ok(())
    }

    /// View a specific session's history (returns data)
    pub fn view_session_data(&self, session_id: &str) -> HarperResult<Vec<crate::core::Message>> {
        let history = load_history(self.conn, session_id).unwrap_or_default();
        Ok(history)
    }

    /// View a specific session's history
    pub fn view_session(&self) -> HarperResult<()> {
        self.output.print("Enter session ID to view: ")?;
        self.output.flush()?;
        let session_id = self.input.read_line()?.trim().to_string();

        let history = self.view_session_data(&session_id)?;
        let total_messages = history.len();

        // Show only the last 20 messages to prevent overwhelming output
        const MAX_DISPLAY: usize = 20;
        let display_start = total_messages.saturating_sub(MAX_DISPLAY);

        self.output.println(&format!(
            "
{} (showing last {} of {} messages)
",
            "Session History:".bold().yellow(),
            total_messages.saturating_sub(display_start),
            total_messages
        ))?;

        for msg in &history[display_start..] {
            let line = match msg.role.as_str() {
                "user" => format!("{} {}", "You:".bold().blue(), msg.content.blue()),
                "assistant" => format!("{} {}", "Assistant:".bold().green(), msg.content.green()),
                "system" => format!("{} {}", "System:".bold().magenta(), msg.content.magenta()),
                _ => format!("{}: {}", msg.role, msg.content),
            };
            self.output.println(&line)?;
        }

        if total_messages > MAX_DISPLAY {
            self.output.println(&format!(
                "
{} Use export to view the full transcript.",
                "Note:".bold().cyan()
            ))?;
        }

        Ok(())
    }

    /// Export a session's history to a file
    pub fn export_session(&self) -> HarperResult<()> {
        self.output.print("Enter session ID to export: ")?;
        self.output.flush()?;
        let session_id = self.input.read_line()?.trim().to_string();

        self.output.print("Export format (txt/json) [txt]: ")?;
        self.output.flush()?;
        let format_choice = self.input.read_line()?.trim().to_lowercase();
        let is_json = format_choice == "json";

        let history = load_history(self.conn, &session_id).unwrap_or_default();

        if history.is_empty() {
            self.output
                .println(&format!("No history found for session {}", session_id))?;
            return Ok(());
        }

        let default_filename = format!("harper_export_{}", session_id);
        let extension = if is_json { "json" } else { "txt" };
        let default_path = format!("{}.{}", default_filename, extension);

        self.output
            .print(&format!("Enter output file path [{}]: ", default_path))?;
        self.output.flush()?;
        let output_path = self.input.read_line()?.trim().to_string();

        let output_path = if output_path.is_empty() {
            default_path
        } else {
            output_path
        };

        if is_json {
            let json = serde_json::to_string_pretty(&history)?;
            std::fs::write(&output_path, json)?;
        } else {
            let mut file = File::create(&output_path)?;
            for msg in &history {
                writeln!(
                    &mut file,
                    "[{}] {}: {}",
                    Local::now().format("%Y-%m-%d %H:%M:%S"),
                    msg.role,
                    msg.content.replace('\n', "\n  ")
                )?;
            }
        }

        self.output.println(&format!(
            "Successfully exported {} messages to {}",
            history.len(),
            output_path
        ))?;

        Ok(())
    }

    pub fn export_session_by_id(&self, session_id: &str) -> HarperResult<String> {
        let history = load_history(self.conn, session_id).unwrap_or_default();

        if history.is_empty() {
            return Err(HarperError::File(format!(
                "No history found for session {}",
                session_id
            )));
        }

        let default_filename = format!("harper_export_{}", session_id);
        let output_path = format!("{}.txt", default_filename);

        let mut file = File::create(&output_path)?;
        for msg in &history {
            writeln!(
                &mut file,
                "[{}] {}: {}",
                Local::now().format("%Y-%m-%d %H:%M:%S"),
                msg.role,
                msg.content.replace('\n', "\n  ")
            )?;
        }

        Ok(output_path)
    }
}
