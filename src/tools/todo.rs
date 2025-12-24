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

//! Todo management tool
//!
//! This module provides functionality for managing todo lists.
//! Uses persistent SQLite storage.

use crate::core::error::HarperError;
use crate::memory::storage;
use crate::tools::parsing;

/// Manage todo list operations
/// Supports: [TODO add "task description"], [TODO list], [TODO remove N]
pub fn manage_todo(
    conn: &rusqlite::Connection,
    response: &str,
) -> crate::core::error::HarperResult<String> {
    let args = parsing::extract_tool_args(response, "[TODO", 1)?;

    if args.is_empty() {
        return Err(HarperError::Command("No todo command provided".to_string()));
    }

    let command = &args[0];

    match command.as_str() {
        "add" => {
            if args.len() < 2 {
                return Err(HarperError::Command(
                    "Todo add requires a description".to_string(),
                ));
            }
            let description = args[1..].join(" ");
            storage::save_todo(conn, &description)?;
            Ok(format!("Added todo: {}", description))
        }
        "list" => {
            let todos = storage::load_todos(conn)?;
            if todos.is_empty() {
                Ok("No todos found".to_string())
            } else {
                let mut result = "Current todos:\n".to_string();
                for (i, (_id, desc)) in todos.iter().enumerate() {
                    result.push_str(&format!("{}. {}\n", i + 1, desc));
                }
                Ok(result.trim_end().to_string())
            }
        }
        "remove" => {
            if args.len() < 2 {
                return Err(HarperError::Command(
                    "Todo remove requires an index".to_string(),
                ));
            }
            let index: usize = args[1]
                .parse()
                .map_err(|_| HarperError::Command("Invalid todo index".to_string()))?;
            let todos = storage::load_todos(conn)?;
            if index == 0 || index > todos.len() {
                return Err(HarperError::Command(format!(
                    "Invalid todo index: {}",
                    index
                )));
            }
            let (id, desc) = &todos[index - 1];
            storage::delete_todo(conn, *id)?;
            Ok(format!("Removed todo: {}", desc))
        }
        "clear" => {
            let todos = storage::load_todos(conn)?;
            let count = todos.len();
            storage::clear_todos(conn)?;
            Ok(format!("Cleared {} todos", count))
        }
        _ => Err(HarperError::Command(format!(
            "Unknown todo command: {}. Supported: add, list, remove, clear",
            command
        ))),
    }
}
