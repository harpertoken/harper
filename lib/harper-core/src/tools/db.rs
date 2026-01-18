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

//! Database queries tool
//!
//! This module provides functionality for running read-only SQL queries on SQLite databases.

use crate::core::error::HarperError;
use crate::tools::parsing;
use colored::*;
use rusqlite::{Connection, Result as SqlResult};

/// Run a database query
pub fn run_query(response: &str) -> crate::core::error::HarperResult<String> {
    let args = parsing::extract_tool_args(response, "[DB_QUERY", 2)?;
    let db_path = &args[0];
    let query = &args[1];

    println!(
        "{} Run query on DB {} ? (y/n): {}",
        "System:".bold().magenta(),
        db_path.magenta(),
        query.magenta()
    );
    let mut approval = String::new();
    std::io::stdin().read_line(&mut approval)?;
    if !approval.trim().eq_ignore_ascii_case("y") {
        return Ok("Query cancelled by user".to_string());
    }

    // Safety: only allow SELECT queries
    if !query.trim().to_uppercase().starts_with("SELECT") {
        return Err(HarperError::Command(
            "Only SELECT queries are allowed".to_string(),
        ));
    }

    println!(
        "{} Running query on: {}",
        "System:".bold().magenta(),
        db_path.magenta()
    );

    let conn = Connection::open(db_path)
        .map_err(|e| HarperError::Command(format!("Failed to open DB {}: {}", db_path, e)))?;

    let mut stmt = conn
        .prepare(query)
        .map_err(|e| HarperError::Command(format!("Failed to prepare query: {}", e)))?;

    let column_names: Vec<String> = stmt.column_names().iter().map(|s| s.to_string()).collect();
    let mut rows = stmt
        .query([])
        .map_err(|e| HarperError::Command(format!("Failed to execute query: {}", e)))?;

    let mut result = format!("Columns: {:?}\n", column_names);
    let mut count = 0;
    while let Some(row) = rows
        .next()
        .map_err(|e| HarperError::Command(format!("Failed to read row: {}", e)))?
    {
        let row_str = column_names
            .iter()
            .enumerate()
            .map(|(i, name)| {
                let value: SqlResult<String> = row.get(i);
                format!(
                    "'{:?}': '{:?}'",
                    name,
                    value.unwrap_or_else(|_| "NULL".to_string())
                )
            })
            .collect::<Vec<_>>()
            .join(", ");
        result.push_str(&format!("Row {}: {{{}}}\n", count, row_str));
        count += 1;
        if count > 100 {
            // limit rows
            result.push_str("... (truncated)\n");
            break;
        }
    }

    Ok(format!(
        "Query executed, {} rows returned.\n{}",
        count, result
    ))
}
