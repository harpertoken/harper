// Copyright 2026 harpertoken
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

//! Codebase investigation tool for deep structural analysis

use crate::core::error::{HarperError, HarperResult};
use crate::core::io_traits::UserApproval;
use crate::tools::parsing;
use colored::*;
use std::ffi::OsStr;
use std::path::Path;
use std::process::Command;
use std::sync::Arc;
use tempfile::tempdir;

/// Investigate codebase structural graph and relationships
pub async fn investigate_codebase(
    response: &str,
    approver: Option<Arc<dyn UserApproval>>,
) -> HarperResult<String> {
    let action_prefix = "[CODEBASE_INVESTIGATE";
    let action = parsing::extract_tool_args(response, action_prefix, 2)?[0].clone();

    match action.as_str() {
        "find_calls" => {
            let symbol = parsing::extract_tool_args(response, action_prefix, 2)?[1].clone();
            find_symbol_calls(&symbol).await
        }
        "trace_relationship" => {
            let args = parsing::extract_tool_args(response, action_prefix, 3)?;
            let x = &args[1];
            let y = &args[2];
            trace_relationship(x, y).await
        }
        "clone_context" => {
            let repo_url = parsing::extract_tool_args(response, action_prefix, 2)?[1].clone();
            clone_temp_context(&repo_url, approver).await
        }
        _ => Err(HarperError::Command(format!(
            "Unknown investigation action: {}",
            action
        ))),
    }
}

async fn find_symbol_calls(symbol: &str) -> HarperResult<String> {
    println!(
        "{} Searching for all callers of: {}",
        "System:".bold().magenta(),
        symbol.magenta()
    );

    let output = run_search_command(
        "rg",
        [
            "-n",
            "-F",
            "--hidden",
            "--glob",
            "!target",
            "--glob",
            "!node_modules",
            symbol,
            ".",
        ],
    )
    .or_else(|_| run_search_command("grep", ["-R", "-n", "-F", symbol, "."]))?;

    if output.status.code() == Some(1) || output.stdout.is_empty() {
        Ok(format!("No callers found for symbol: {}", symbol))
    } else if !output.status.success() {
        Err(HarperError::Command(format!(
            "Search failed: {}",
            String::from_utf8_lossy(&output.stderr).trim()
        )))
    } else {
        let result = String::from_utf8_lossy(&output.stdout);
        Ok(format!("Callers found for {}:\n{}", symbol, result))
    }
}

async fn trace_relationship(x: &str, y: &str) -> HarperResult<String> {
    println!(
        "{} Tracing relationship between {} and {}",
        "System:".bold().magenta(),
        x.magenta(),
        y.magenta()
    );

    let output = run_search_command(
        "rg",
        [
            "-l",
            "-F",
            "--hidden",
            "--glob",
            "!target",
            "--glob",
            "!node_modules",
            x,
            ".",
        ],
    )
    .or_else(|_| run_search_command("grep", ["-l", "-R", "-F", x, "."]))?;

    if output.status.code() != Some(0) && output.status.code() != Some(1) {
        return Err(HarperError::Command(format!(
            "Search failed: {}",
            String::from_utf8_lossy(&output.stderr).trim()
        )));
    }

    let x_files = String::from_utf8_lossy(&output.stdout);
    let mut relationships = Vec::new();

    for file in x_files.lines() {
        let has_y = file_contains_symbol(file, y)?;

        if has_y {
            relationships.push(format!("Found both in: {}", file));
        }
    }

    if relationships.is_empty() {
        Ok(format!(
            "No direct file-level relationship found between {} and {}",
            x, y
        ))
    } else {
        Ok(format!(
            "Relationships found:\n{}",
            relationships.join("\n")
        ))
    }
}

async fn clone_temp_context(
    repo_url: &str,
    approver: Option<Arc<dyn UserApproval>>,
) -> HarperResult<String> {
    println!(
        "{} Cloning temporary context from: {}",
        "System:".bold().magenta(),
        repo_url.magenta()
    );

    if let Some(appr) = approver {
        if !appr
            .approve("Clone repository for investigation?", repo_url)
            .await?
        {
            return Ok("Repository clone cancelled by user".to_string());
        }
    }

    let dir =
        tempdir().map_err(|e| HarperError::Command(format!("Failed to create temp dir: {}", e)))?;
    let path = dir.path();
    let path_str = path
        .to_str()
        .ok_or_else(|| HarperError::Command("Temporary path is not valid UTF-8".to_string()))?;

    let output = Command::new("git")
        .args(["clone", "--depth", "1", repo_url, path_str])
        .output()
        .map_err(|e| HarperError::Command(format!("Git clone failed: {}", e)))?;

    if !output.status.success() {
        return Err(HarperError::Command(format!(
            "Failed to clone repository {}: {}",
            repo_url,
            String::from_utf8_lossy(&output.stderr).trim()
        )));
    }

    // Analyze the cloned repo briefly
    let file_count = walkdir::WalkDir::new(path).into_iter().count();

    Ok(format!(
        "Cloned {} to temporary directory. Found {} items for context.",
        repo_url, file_count
    ))
}

fn run_search_command<I, S>(program: &str, args: I) -> HarperResult<std::process::Output>
where
    I: IntoIterator<Item = S>,
    S: AsRef<OsStr>,
{
    Command::new(program)
        .args(args)
        .output()
        .map_err(|e| HarperError::Command(format!("{} failed: {}", program, e)))
}

fn file_contains_symbol(file: &str, symbol: &str) -> HarperResult<bool> {
    let output = run_search_command("rg", ["-q", "-F", symbol, file])
        .or_else(|_| run_search_command("grep", ["-q", "-F", symbol, file]))?;

    match output.status.code() {
        Some(0) => Ok(true),
        Some(1) => Ok(false),
        _ => Err(HarperError::Command(format!(
            "Failed to inspect {}: {}",
            Path::new(file).display(),
            String::from_utf8_lossy(&output.stderr).trim()
        ))),
    }
}
