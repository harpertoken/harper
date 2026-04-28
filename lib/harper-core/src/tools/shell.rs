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

//! Shell command execution tool
//!
//! This module provides functionality for executing shell commands
//! with safety checks and user approval.

use crate::core::plan::PlanJobStatus;
use crate::core::{error::HarperError, ApiConfig};
use crate::memory::storage::{self, CommandLogRecord};
use crate::runtime::config::ExecPolicyConfig;
use colored::*;
use rusqlite::Connection;
use std::io::{self, Write};
use std::time::Instant;
use tokio::io::{AsyncBufReadExt, BufReader};
use tokio::process::Command as TokioCommand;

const OUTPUT_PREVIEW_LIMIT: usize = 512;

/// Context for persisting command audit logs
pub struct CommandAuditContext<'a> {
    pub conn: &'a Connection,
    pub session_id: Option<&'a str>,
    pub source: &'a str,
}

use crate::core::io_traits::{RuntimeEventSink, UserApproval};
use std::sync::Arc;

async fn emit_activity_update(
    runtime_events: Option<&Arc<dyn RuntimeEventSink>>,
    session_id: Option<&str>,
    status: Option<String>,
) {
    if let (Some(sink), Some(session_id)) = (runtime_events, session_id) {
        let _ = sink.activity_updated(session_id, status).await;
    }
}

async fn emit_command_output(
    runtime_events: Option<&Arc<dyn RuntimeEventSink>>,
    session_id: Option<&str>,
    command: &str,
    chunk: String,
    is_error: bool,
    done: bool,
) {
    if let (Some(sink), Some(session_id)) = (runtime_events, session_id) {
        let _ = sink
            .command_output_updated(session_id, command.to_string(), chunk, is_error, done)
            .await;
    }
}

fn database_path(conn: &Connection) -> Option<String> {
    let mut stmt = conn.prepare("PRAGMA database_list").ok()?;
    let mut rows = stmt.query([]).ok()?;
    while let Ok(Some(row)) = rows.next() {
        let name: String = row.get(1).ok()?;
        let path: String = row.get(2).ok()?;
        if name == "main" && !path.trim().is_empty() {
            return Some(path);
        }
    }
    None
}

/// Execute a shell command with safety checks
pub async fn execute_command(
    response: &str,
    _config: &ApiConfig,
    exec_policy: &ExecPolicyConfig,
    audit_ctx: Option<&CommandAuditContext<'_>>,
    approver: Option<Arc<dyn UserApproval>>,
    runtime_events: Option<Arc<dyn RuntimeEventSink>>,
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
    // We allow wildcards (*, ?) and git revision syntax (~, ^) but block chaining and subshells.
    let dangerous_chars = [';', '|', '&', '`', '$', '(', ')', '<', '>', '\n', '\r'];
    if command_str.chars().any(|c| dangerous_chars.contains(&c)) {
        let message = "Command contains potentially dangerous shell metacharacters (like ;, |, &) or newlines. \
             Command chaining and redirection are not allowed for security.";
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
        emit_activity_update(
            runtime_events.as_ref(),
            audit_ctx.and_then(|ctx| ctx.session_id),
            Some(format!("waiting approval: {}", command_str)),
        )
        .await;
        if let Some(ctx) =
            audit_ctx.and_then(|ctx| ctx.session_id.map(|session_id| (ctx.conn, session_id)))
        {
            let _ = crate::tools::plan::start_plan_job(
                ctx.0,
                ctx.1,
                "run_command",
                Some(command_str.to_string()),
                PlanJobStatus::WaitingApproval,
            );
            emit_plan_update(runtime_events.as_ref(), ctx.0, ctx.1).await;
        }
        let is_approved = if let Some(appr) = approver {
            appr.approve("Execute command?", command_str).await?
        } else {
            // Fallback to spawn_blocking for stdin if no approver provided (legacy support)
            let prompt = "Execute command?".to_string();
            let cmd = command_str.to_string();
            tokio::task::spawn_blocking(move || {
                println!(
                    "{} {} {} (y/n): ",
                    "System:".bold().magenta(),
                    prompt.bold().magenta(),
                    cmd.magenta()
                );
                io::stdout()
                    .flush()
                    .map_err(|e| HarperError::Io(e.to_string()))?;

                let mut approval = String::new();
                io::stdin()
                    .read_line(&mut approval)
                    .map_err(|e| HarperError::Io(e.to_string()))?;
                Ok::<bool, HarperError>(approval.trim().eq_ignore_ascii_case("y"))
            })
            .await
            .map_err(|e| HarperError::Command(format!("Task execution failed: {}", e)))??
        };

        if !is_approved {
            emit_activity_update(
                runtime_events.as_ref(),
                audit_ctx.and_then(|ctx| ctx.session_id),
                Some("approval rejected".to_string()),
            )
            .await;
            if let Some(ctx) =
                audit_ctx.and_then(|ctx| ctx.session_id.map(|session_id| (ctx.conn, session_id)))
            {
                let _ = crate::tools::plan::finish_active_plan_job(
                    ctx.0,
                    ctx.1,
                    PlanJobStatus::Blocked,
                );
                emit_plan_update(runtime_events.as_ref(), ctx.0, ctx.1).await;
            }
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
    }

    if let Some(ctx) =
        audit_ctx.and_then(|ctx| ctx.session_id.map(|session_id| (ctx.conn, session_id)))
    {
        emit_activity_update(
            runtime_events.as_ref(),
            Some(ctx.1),
            Some(format!("running command: {}", command_str)),
        )
        .await;
        let _ = crate::tools::plan::update_active_plan_job(ctx.0, ctx.1, PlanJobStatus::Running)
            .or_else(|_| {
                crate::tools::plan::start_plan_job(
                    ctx.0,
                    ctx.1,
                    "run_command",
                    Some(command_str.to_string()),
                    PlanJobStatus::Running,
                )
            });
        emit_plan_update(runtime_events.as_ref(), ctx.0, ctx.1).await;
    }

    let start = Instant::now();
    let mut child = TokioCommand::new("sh")
        .arg("-c")
        .arg(command_str)
        .stdout(std::process::Stdio::piped())
        .stderr(std::process::Stdio::piped())
        .spawn()
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

    let mut stdout_task = None;
    if let Some(stdout) = child.stdout.take() {
        let runtime_events = runtime_events.clone();
        let session_id = audit_ctx.and_then(|ctx| ctx.session_id).map(str::to_string);
        let db_path = audit_ctx.and_then(|ctx| database_path(ctx.conn));
        let command = command_str.to_string();
        stdout_task = Some(tokio::spawn(async move {
            let live_conn = db_path
                .as_deref()
                .and_then(|path| crate::memory::storage::create_connection(path).ok());
            let mut lines = BufReader::new(stdout).lines();
            let mut collected = String::new();
            while let Ok(Some(line)) = lines.next_line().await {
                collected.push_str(&line);
                collected.push('\n');
                if let (Some(conn), Some(session_id)) = (&live_conn, session_id.as_deref()) {
                    let _ = crate::tools::plan::append_active_plan_job_output(
                        conn,
                        session_id,
                        &format!("{}\n", line),
                        false,
                    );
                    if let Some(sink) = runtime_events.as_ref() {
                        let plan = crate::memory::storage::load_plan_state(conn, session_id)
                            .ok()
                            .flatten();
                        let _ = sink.plan_updated(session_id, plan).await;
                    }
                }
                emit_command_output(
                    runtime_events.as_ref(),
                    session_id.as_deref(),
                    &command,
                    format!("{}\n", line),
                    false,
                    false,
                )
                .await;
            }
            collected
        }));
    }

    let mut stderr_task = None;
    if let Some(stderr) = child.stderr.take() {
        let runtime_events = runtime_events.clone();
        let session_id = audit_ctx.and_then(|ctx| ctx.session_id).map(str::to_string);
        let db_path = audit_ctx.and_then(|ctx| database_path(ctx.conn));
        let command = command_str.to_string();
        stderr_task = Some(tokio::spawn(async move {
            let live_conn = db_path
                .as_deref()
                .and_then(|path| crate::memory::storage::create_connection(path).ok());
            let mut lines = BufReader::new(stderr).lines();
            let mut collected = String::new();
            while let Ok(Some(line)) = lines.next_line().await {
                collected.push_str(&line);
                collected.push('\n');
                if let (Some(conn), Some(session_id)) = (&live_conn, session_id.as_deref()) {
                    let _ = crate::tools::plan::append_active_plan_job_output(
                        conn,
                        session_id,
                        &format!("{}\n", line),
                        true,
                    );
                    if let Some(sink) = runtime_events.as_ref() {
                        let plan = crate::memory::storage::load_plan_state(conn, session_id)
                            .ok()
                            .flatten();
                        let _ = sink.plan_updated(session_id, plan).await;
                    }
                }
                emit_command_output(
                    runtime_events.as_ref(),
                    session_id.as_deref(),
                    &command,
                    format!("{}\n", line),
                    true,
                    false,
                )
                .await;
            }
            collected
        }));
    }

    let status = child.wait().await.map_err(|e| {
        let err_msg = format!("Failed to wait for command: {}", e);
        HarperError::Command(err_msg)
    })?;
    let duration_ms = start.elapsed().as_millis() as i64;
    let stdout_text = match stdout_task {
        Some(task) => task.await.unwrap_or_default(),
        None => String::new(),
    };
    let stderr_text = match stderr_task {
        Some(task) => task.await.unwrap_or_default(),
        None => String::new(),
    };
    emit_command_output(
        runtime_events.as_ref(),
        audit_ctx.and_then(|ctx| ctx.session_id),
        command_str,
        String::new(),
        false,
        true,
    )
    .await;
    let exit_code = status.code();
    let stdout_preview = bytes_to_preview(stdout_text.as_bytes());
    let stderr_preview = bytes_to_preview(stderr_text.as_bytes());
    let output_preview = if status.success() {
        preview_text(&stdout_text)
    } else if !stderr_text.trim().is_empty() {
        preview_text(&stderr_text)
    } else {
        preview_text(&stdout_text)
    };
    let has_error_output = !status.success() || !stderr_text.trim().is_empty();

    let result = if status.success() {
        let out = stdout_text;
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
        let err_output = stderr_text;
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

    if let Some(ctx) =
        audit_ctx.and_then(|ctx| ctx.session_id.map(|session_id| (ctx.conn, session_id)))
    {
        let _ = crate::tools::plan::finish_active_plan_job_with_output(
            ctx.0,
            ctx.1,
            if status.success() {
                PlanJobStatus::Succeeded
            } else {
                PlanJobStatus::Failed
            },
            output_preview,
            has_error_output,
        );
        emit_plan_update(runtime_events.as_ref(), ctx.0, ctx.1).await;
    }

    Ok(result)
}

async fn emit_plan_update(
    runtime_events: Option<&Arc<dyn RuntimeEventSink>>,
    conn: &Connection,
    session_id: &str,
) {
    if let Some(sink) = runtime_events {
        let plan = crate::memory::storage::load_plan_state(conn, session_id)
            .ok()
            .flatten();
        let _ = sink.plan_updated(session_id, plan).await;
    }
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

fn preview_text(text: &str) -> Option<String> {
    let trimmed = text.trim();
    if trimmed.is_empty() {
        return None;
    }

    let preview: String = trimmed.chars().take(OUTPUT_PREVIEW_LIMIT).collect();
    if trimmed.chars().count() > OUTPUT_PREVIEW_LIMIT {
        Some(format!("{}…", preview))
    } else {
        Some(preview)
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
