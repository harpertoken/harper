//! Todo management tool
//!
//! This module provides functionality for managing todo lists.
//! Currently uses in-memory storage - persistent storage would be needed for production use.

use crate::core::error::HarperError;
use crate::tools::parsing;
use std::sync::Mutex;
use once_cell::sync::Lazy;

// Simple in-memory todo storage
// TODO: Replace with persistent storage (database/file)
static TODOS: Lazy<Mutex<Vec<String>>> = Lazy::new(|| Mutex::new(Vec::new()));

/// Manage todo list operations
/// Supports: [TODO add "task description"], [TODO list], [TODO remove N]
pub fn manage_todo(response: &str) -> crate::core::error::HarperResult<String> {
    let args = parsing::extract_tool_args(response, "[TODO", 1)?;

    if args.is_empty() {
        return Err(HarperError::Command("No todo command provided".to_string()));
    }

    let command = &args[0];

    match command.as_str() {
        "add" => {
            if args.len() < 2 {
                return Err(HarperError::Command("Todo add requires a description".to_string()));
            }
            let description = args[1..].join(" ");
            let mut todos = TODOS.lock().map_err(|_| {
                HarperError::Command("Failed to access todo storage".to_string())
            })?;
            todos.push(description.clone());
            Ok(format!("Added todo: {}", description))
        }
        "list" => {
            let todos = TODOS.lock().map_err(|_| {
                HarperError::Command("Failed to access todo storage".to_string())
            })?;
            if todos.is_empty() {
                Ok("No todos found".to_string())
            } else {
                let mut result = "Current todos:\n".to_string();
                for (i, todo) in todos.iter().enumerate() {
                    result.push_str(&format!("{}. {}\n", i + 1, todo));
                }
                Ok(result.trim_end().to_string())
            }
        }
        "remove" => {
            if args.len() < 2 {
                return Err(HarperError::Command("Todo remove requires an index".to_string()));
            }
            let index: usize = args[1].parse().map_err(|_| {
                HarperError::Command("Invalid todo index".to_string())
            })?;
            let mut todos = TODOS.lock().map_err(|_| {
                HarperError::Command("Failed to access todo storage".to_string())
            })?;
            if index == 0 || index > todos.len() {
                return Err(HarperError::Command(format!("Invalid todo index: {}", index)));
            }
            let removed = todos.remove(index - 1);
            Ok(format!("Removed todo: {}", removed))
        }
        "clear" => {
            let mut todos = TODOS.lock().map_err(|_| {
                HarperError::Command("Failed to access todo storage".to_string())
            })?;
            let count = todos.len();
            todos.clear();
            Ok(format!("Cleared {} todos", count))
        }
        _ => {
            Err(HarperError::Command(format!(
                "Unknown todo command: {}. Supported: add, list, remove, clear",
                command
            )))
        }
    }
}
