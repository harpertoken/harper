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

//! Shell command execution tool
//!
//! This module provides functionality for executing shell commands
//! with safety checks and user approval.

use crate::core::{error::HarperError, ApiConfig};
use crate::runtime::config::ExecPolicyConfig;
use colored::*;
use std::io;

/// Execute a shell command with safety checks
pub fn execute_command(
    response: &str,
    _config: &ApiConfig,
    exec_policy: &ExecPolicyConfig,
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
    let dangerous_chars = [
        ';', '|', '&', '`', '$', '(', ')', '<', '>', '*', '?', '[', ']', '{', '}', '!', '~',
    ];
    if command_str.chars().any(|c| dangerous_chars.contains(&c)) {
        return Err(HarperError::Command(
            "Command contains potentially dangerous shell metacharacters. \
             Only basic commands without shell features are allowed."
                .to_string(),
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
            return Err(HarperError::Command(format!(
                "Command contains potentially dangerous pattern: '{}'. \
                        This command is not allowed for security reasons.",
                pattern
            )));
        }
    }

    // Check exec policy
    let mut requires_approval = true;

    if let Some(blocked) = &exec_policy.blocked_commands {
        if blocked.iter().any(|cmd| command_str.starts_with(cmd)) {
            return Err(HarperError::Command(format!(
                "Command '{}' is blocked by exec policy.",
                command_str
            )));
        }
    }

    if let Some(allowed) = &exec_policy.allowed_commands {
        if allowed.iter().any(|cmd| command_str.starts_with(cmd)) {
            requires_approval = false;
        } else {
            // If allowed list is set, only allowed commands are permitted without approval
            requires_approval = true;
        }
    }

    // Ask for approval if required
    if requires_approval {
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
    } else {
        println!(
            "{} Executing allowed command: {}",
            "System:".bold().magenta(),
            command_str.magenta()
        );
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
