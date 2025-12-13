//! Git operations tool
//!
//! This module provides functionality for git operations
//! with safety checks and user approval.

use crate::core::error::HarperError;
use colored::*;
use std::io;

/// Get git status
pub fn git_status() -> crate::core::error::HarperResult<String> {
    println!("{} Running git status...", "System:".bold().magenta());

    let output = std::process::Command::new("git")
        .arg("status")
        .arg("--porcelain")
        .output()
        .map_err(|e| HarperError::Command(format!("Failed to run git status: {}", e)))?;

    let result = if output.status.success() {
        let stdout = String::from_utf8_lossy(&output.stdout);
        if stdout.trim().is_empty() {
            "Git working directory is clean".to_string()
        } else {
            format!("Git status:\n{}", stdout)
        }
    } else {
        String::from_utf8_lossy(&output.stderr).to_string()
    };

    Ok(result)
}

/// Show git diff
pub fn git_diff() -> crate::core::error::HarperResult<String> {
    println!("{} Running git diff...", "System:".bold().magenta());

    let output = std::process::Command::new("git")
        .arg("diff")
        .output()
        .map_err(|e| HarperError::Command(format!("Failed to run git diff: {}", e)))?;

    let result = if output.status.success() {
        let stdout = String::from_utf8_lossy(&output.stdout);
        if stdout.trim().is_empty() {
            "No changes to show".to_string()
        } else {
            format!("Git diff:\n{}", stdout)
        }
    } else {
        String::from_utf8_lossy(&output.stderr).to_string()
    };

    Ok(result)
}

/// Commit changes
pub fn git_commit(response: &str) -> crate::core::error::HarperResult<String> {
    let message = extract_commit_message(response)?;

    println!(
        "{} Commit with message: '{}' ? (y/n): ",
        "System:".bold().magenta(),
        message.magenta()
    );
    let mut approval = String::new();
    io::stdin().read_line(&mut approval)?;
    if !approval.trim().eq_ignore_ascii_case("y") {
        return Ok("Git commit cancelled by user".to_string());
    }

    println!("{} Running git commit...", "System:".bold().magenta());

    let output = std::process::Command::new("git")
        .arg("commit")
        .arg("-m")
        .arg(&message)
        .output()
        .map_err(|e| HarperError::Command(format!("Failed to run git commit: {}", e)))?;

    let result = if output.status.success() {
        String::from_utf8_lossy(&output.stdout).to_string()
    } else {
        String::from_utf8_lossy(&output.stderr).to_string()
    };

    Ok(result)
}

/// Add files to git
pub fn git_add(response: &str) -> crate::core::error::HarperResult<String> {
    let files = extract_files(response)?;

    println!(
        "{} Add files: {} ? (y/n): ",
        "System:".bold().magenta(),
        files.join(", ").magenta()
    );
    let mut approval = String::new();
    io::stdin().read_line(&mut approval)?;
    if !approval.trim().eq_ignore_ascii_case("y") {
        return Ok("Git add cancelled by user".to_string());
    }

    println!("{} Running git add...", "System:".bold().magenta());

    let mut command = std::process::Command::new("git");
    command.arg("add");
    for file in &files {
        command.arg(file);
    }

    let output = command
        .output()
        .map_err(|e| HarperError::Command(format!("Failed to run git add: {}", e)))?;

    let result = if output.status.success() {
        format!("Added {} files to git", files.len())
    } else {
        String::from_utf8_lossy(&output.stderr).to_string()
    };

    Ok(result)
}

/// Extract commit message from response
fn extract_commit_message(response: &str) -> crate::core::error::HarperResult<String> {
    let message = response
        .strip_prefix("[GIT_COMMIT")
        .and_then(|s| s.strip_suffix(']'))
        .ok_or_else(|| HarperError::Command("Invalid git commit format".to_string()))?
        .trim();

    if message.is_empty() {
        return Err(HarperError::Command(
            "No commit message provided".to_string(),
        ));
    }

    Ok(message.to_string())
}

/// Extract files from response
fn extract_files(response: &str) -> crate::core::error::HarperResult<Vec<String>> {
    let files_str = response
        .strip_prefix("[GIT_ADD")
        .and_then(|s| s.strip_suffix(']'))
        .ok_or_else(|| HarperError::Command("Invalid git add format".to_string()))?
        .trim();

    if files_str.is_empty() {
        return Ok(vec![".".to_string()]); // Add all if no files specified
    }

    let files: Vec<String> = files_str
        .split_whitespace()
        .map(|s| s.to_string())
        .collect();
    Ok(files)
}
