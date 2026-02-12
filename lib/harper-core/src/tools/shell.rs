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
use crate::memory::storage::{self, CommandLogRecord};
use crate::runtime::config::ExecPolicyConfig;
use colored::*;
use rusqlite::Connection;
use std::io;
use std::time::Instant;

const OUTPUT_PREVIEW_LIMIT: usize = 512;

/// Context for persisting command audit logs
pub struct CommandAuditContext<'a> {
    pub conn: &'a Connection,
    pub session_id: Option<&'a str>,
    pub source: &'a str,
}

/// Execute a shell command with safety checks
pub fn execute_command(
    response: &str,
    _config: &ApiConfig,
    exec_policy: &ExecPolicyConfig,
    audit_ctx: Option<&CommandAuditContext>,
) -> crate::core::error::HarperResult<String> {
    let command_str = if let Some(pos) = response.find(' ') {
        response[pos..].trim_start().trim_end_matches(']')
    } else {
        ""
    };

    if command_str.is_empty() {
        maybe_log_command(
            audit_ctx,
            command_str,
            "invalid",
            true,
            false,
            None,
            None,
            None,
            None,
            Some("No command provided".to_string()),
        );
        return Err(HarperError::Command("No command provided".to_string()));
    }

    // Security check to prevent shell injection and dangerous commands
    // Note: This is a defense-in-depth measure. The primary security comes from user approval.
    let dangerous_chars = [
        ';', '|', '&', '`', '$', '(', ')', '<', '>', '*', '?', '[', ']', '{', '}', '!', '~',
    ];
    if command_str.chars().any(|c| dangerous_chars.contains(&c)) {
        let message = "Command contains potentially dangerous shell metacharacters. \
             Only basic commands without shell features are allowed.";
        maybe_log_command(
            audit_ctx,
            command_str,
            "blocked",
            true,
            false,
            None,
            None,
            None,
            None,
            Some(message.to_string()),
        );
        return Err(HarperError::Command(message.to_string()));
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
            let err = format!(
                "Command contains potentially dangerous pattern: '{}'. \
                        This command is not allowed for security reasons.",
                pattern
            );
            maybe_log_command(
                audit_ctx,
                command_str,
                "blocked",
                true,
                false,
                None,
                None,
                None,
                None,
                Some(err.clone()),
            );
            return Err(HarperError::Command(err));
        }
    }

    // Check exec policy
    let mut requires_approval = true;
    let mut approved = false;

    if let Some(blocked) = &exec_policy.blocked_commands {
        if blocked.iter().any(|cmd| command_str.starts_with(cmd)) {
            let err = format!("Command '{}' is blocked by exec policy.", command_str);
            maybe_log_command(
                audit_ctx,
                command_str,
                "blocked",
                requires_approval,
                false,
                None,
                None,
                None,
                None,
                Some(err.clone()),
            );
            return Err(HarperError::Command(err));
        }
    }

    if let Some(allowed) = &exec_policy.allowed_commands {
        if allowed.iter().any(|cmd| command_str.starts_with(cmd)) {
            requires_approval = false;
            approved = true;
        } else {
            // If allowed list is set, only allowed commands are permitted without approval
            requires_approval = true;
        }
    } else {
        approved = !requires_approval;
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
            maybe_log_command(
                audit_ctx,
                command_str,
                "cancelled",
                requires_approval,
                false,
                None,
                None,
                None,
                None,
                Some("User rejected command".to_string()),
            );
            return Ok("Command execution cancelled by user".to_string());
        }
        approved = true;
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

    let start = Instant::now();
    let output = std::process::Command::new("sh")
        .arg("-c")
        .arg(command_str)
        .output()
        .map_err(|e| {
            let err_msg = format!("Failed to execute command: {}", e);
            maybe_log_command(
                audit_ctx,
                command_str,
                "error",
                requires_approval,
                approved,
                None,
                None,
                None,
                None,
                Some(err_msg.clone()),
            );
            HarperError::Command(err_msg)
        })?;
    let duration_ms = start.elapsed().as_millis() as i64;
    let exit_code = output.status.code();
    let stdout_preview = bytes_to_preview(&output.stdout);
    let stderr_preview = bytes_to_preview(&output.stderr);

    let result = if output.status.success() {
        let out = String::from_utf8_lossy(&output.stdout).to_string();
        maybe_log_command(
            audit_ctx,
            command_str,
            "succeeded",
            requires_approval,
            approved,
            exit_code,
            Some(duration_ms),
            stdout_preview,
            stderr_preview,
            None,
        );
        out
    } else {
        let err_output = String::from_utf8_lossy(&output.stderr).to_string();
        maybe_log_command(
            audit_ctx,
            command_str,
            "failed",
            requires_approval,
            approved,
            exit_code,
            Some(duration_ms),
            stdout_preview,
            stderr_preview,
            Some("Command exited with non-zero status".to_string()),
        );
        err_output
    };

    Ok(result)
}

#[allow(clippy::too_many_arguments)]
fn maybe_log_command(
    audit_ctx: Option<&CommandAuditContext>,
    command: &str,
    status: &str,
    requires_approval: bool,
    approved: bool,
    exit_code: Option<i32>,
    duration_ms: Option<i64>,
    stdout_preview: Option<String>,
    stderr_preview: Option<String>,
    error_message: Option<String>,
) {
    if let Some(ctx) = audit_ctx {
        let record = CommandLogRecord::new(
            ctx.session_id,
            command,
            ctx.source,
            requires_approval,
            approved,
            status,
            exit_code,
            duration_ms,
            stdout_preview,
            stderr_preview,
            error_message,
        );
        if let Err(err) = storage::insert_command_log(ctx.conn, &record) {
            eprintln!("Warning: failed to persist command log: {}", err);
        }
    }
}

fn bytes_to_preview(bytes: &[u8]) -> Option<String> {
    if bytes.is_empty() {
        return None;
    }
    let text = String::from_utf8_lossy(bytes);
    let mut preview = text.chars().take(OUTPUT_PREVIEW_LIMIT).collect::<String>();
    if text.chars().count() > OUTPUT_PREVIEW_LIMIT {
        preview.push('…');
    }
    Some(preview)
}
