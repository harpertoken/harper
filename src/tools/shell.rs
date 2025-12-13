//! Shell command execution tool
//!
//! This module provides functionality for executing shell commands
//! with safety checks and user approval.

use crate::core::{error::HarperError, ApiConfig};
use colored::*;
use std::io;

/// Execute a shell command with safety checks
pub fn execute_command(
    response: &str,
    _config: &ApiConfig,
) -> crate::core::error::HarperResult<String> {
    let command_str = if let Some(pos) = response.find(' ') {
        response[pos..].trim_start().trim_end_matches(']')
    } else {
        ""
    };

    if command_str.is_empty() {
        return Err(HarperError::Command("No command provided".to_string()));
    }

    // Basic security check to prevent shell injection
    if command_str
        .chars()
        .any(|c| matches!(c, ';' | '|' | '&' | '`' | '$' | '(' | ')'))
    {
        return Err(HarperError::Command(
            "Command contains potentially dangerous characters".to_string(),
        ));
    }

    // Ask for approval
    println!(
        "{} Execute command? {} (y/n): ",
        "System:".bold().magenta(),
        command_str.magenta()
    );
    let mut approval = String::new();
    io::stdin().read_line(&mut approval)?;
    if !approval.trim().eq_ignore_ascii_case("y") {
        return Ok("Command execution cancelled by user".to_string());
    }

    println!(
        "{} Running command: {}",
        "System:".bold().magenta(),
        command_str.magenta()
    );

    let output = std::process::Command::new("sh")
        .arg("-c")
        .arg(command_str)
        .output()
        .map_err(|e| HarperError::Command(format!("Failed to execute command: {}", e)))?;

    let result = if output.status.success() {
        String::from_utf8_lossy(&output.stdout).to_string()
    } else {
        String::from_utf8_lossy(&output.stderr).to_string()
    };

    Ok(result)
}
