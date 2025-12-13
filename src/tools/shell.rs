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

    // Security check to prevent shell injection and dangerous commands
    // Note: This is a defense-in-depth measure. The primary security comes from user approval.
    let dangerous_chars = [';', '|', '&', '`', '$', '(', ')', '<', '>', '*', '?', '[', ']', '{', '}', '!', '~'];
    if command_str.chars().any(|c| dangerous_chars.contains(&c)) {
        return Err(HarperError::Command(
            "Command contains potentially dangerous shell metacharacters. \
             Only basic commands without shell features are allowed.".to_string(),
        ));
    }

    // Additional check for common dangerous patterns
    let dangerous_patterns = [
        "rm -rf",
        "rmdir",
        "del ",
        "format",
        "fdisk",
        "mkfs",
        "dd if=",
        "shutdown",
        "reboot",
        "halt",
        "poweroff",
        "sudo",
        "su ",
        "chmod 777",
        "chown root",
        "passwd",
        "/etc/",
        "/bin/",
        "/sbin/",
        "/usr/bin/",
        "/usr/sbin/",
    ];

    for pattern in &dangerous_patterns {
        if command_str.contains(pattern) {
            return Err(HarperError::Command(
                format!("Command contains potentially dangerous pattern: '{}'. \
                        This command is not allowed for security reasons.", pattern),
            ));
        }
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
