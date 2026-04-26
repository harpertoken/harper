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

use axum::{
    extract::{Path, State},
    http::StatusCode,
    response::Json,
    routing::{delete, get, post},
    Router,
};
use reqwest::Client;
use rusqlite::Connection;
use serde::{Deserialize, Serialize};
use std::sync::{Arc, Mutex};

use crate::agent::intent::route_intent;
use crate::core::error::{HarperError, HarperResult};
use crate::core::llm_client::call_llm;
use crate::core::{ApiConfig, Message};
use crate::memory::storage::{save_message, CommandLogRecord};
use crate::runtime::config::ExecPolicyConfig;
use rusqlite::params;

#[derive(Clone)]
pub struct ServerState {
    pub conn: Arc<Mutex<Connection>>,
    pub api_config: ApiConfig,
    pub client: Client,
    pub exec_policy: ExecPolicyConfig,
}

#[derive(Serialize)]
pub struct HealthResponse {
    pub status: String,
    pub version: String,
}

#[derive(Serialize)]
pub struct SessionListItem {
    pub id: String,
    pub created_at: String,
    pub updated_at: String,
    pub title: String,
}

#[derive(Deserialize)]
pub struct ChatRequest {
    pub message: String,
    pub session_id: Option<String>,
}

#[derive(Serialize)]
pub struct ChatResponse {
    pub message: String,
    pub session_id: String,
    pub status: String,
    pub pending_id: Option<String>,
    pub pending_tools: Option<Vec<PendingTool>>,
}

#[derive(Serialize, Deserialize, Clone)]
pub struct PendingTool {
    pub id: String,
    pub tool: String,
    pub args: serde_json::Value,
}

#[derive(Serialize)]
pub struct PendingApproval {
    pub id: i64,
    pub command: String,
    pub source: String,
    pub session_id: String,
    pub created_at: String,
}

#[derive(Deserialize)]
pub struct ApprovalRequest {
    pub approved: bool,
}

#[derive(Debug, Deserialize)]
pub struct ReviewRequest {
    pub file_path: String,
    pub content: String,
    pub language: Option<String>,
    pub workspace_root: Option<String>,
    pub instructions: Option<String>,
    pub selection: Option<ReviewRange>,
    pub max_findings: Option<usize>,
}

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct ReviewRange {
    pub start_line: usize,
    pub start_column: usize,
    pub end_line: usize,
    pub end_column: usize,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ReviewResponse {
    pub summary: String,
    pub findings: Vec<CodeReviewFinding>,
    pub model: String,
}

#[derive(Debug, Deserialize)]
struct ModelReviewResponse {
    pub summary: String,
    pub findings: Vec<CodeReviewFinding>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct CodeReviewFinding {
    pub title: String,
    pub severity: String,
    pub message: String,
    pub range: ReviewRange,
    pub suggestion: Option<CodeSuggestion>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct CodeSuggestion {
    pub description: String,
    pub replacement: String,
}

pub async fn health() -> Json<HealthResponse> {
    Json(HealthResponse {
        status: "ok".to_string(),
        version: crate::core::constants::VERSION.to_string(),
    })
}

pub async fn list_sessions(State(state): State<Arc<ServerState>>) -> Json<Vec<SessionListItem>> {
    let conn = state
        .conn
        .lock()
        .expect("Failed to lock database connection");
    let mut stmt = conn
        .prepare("SELECT id, created_at, updated_at, title FROM sessions ORDER BY updated_at DESC LIMIT 50")
        .expect("Failed to prepare SQL statement");

    let sessions: Vec<SessionListItem> = stmt
        .query_map([], |row| {
            Ok(SessionListItem {
                id: row.get(0)?,
                created_at: row.get(1)?,
                updated_at: row.get(2)?,
                title: row.get(3)?,
            })
        })
        .expect("Failed to execute query")
        .filter_map(|r| r.ok())
        .collect();

    Json(sessions)
}

pub async fn get_session(
    State(state): State<Arc<ServerState>>,
    Path(session_id): Path<String>,
) -> Result<Json<serde_json::Value>, StatusCode> {
    let conn = state
        .conn
        .lock()
        .expect("Failed to lock database connection");
    let session_service = crate::memory::session_service::SessionService::new(&conn);
    let session_view = session_service
        .load_session_state_view(&session_id)
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    Ok(Json(serde_json::json!(session_view)))
}

pub async fn delete_session(
    State(state): State<Arc<ServerState>>,
    Path(session_id): Path<String>,
) -> StatusCode {
    let conn = state
        .conn
        .lock()
        .expect("Failed to lock database connection");

    match conn.execute("DELETE FROM sessions WHERE id = ?", [&session_id]) {
        Ok(_) => StatusCode::NO_CONTENT,
        Err(_) => StatusCode::INTERNAL_SERVER_ERROR,
    }
}

pub async fn chat_endpoint(
    State(state): State<Arc<ServerState>>,
    Json(payload): Json<ChatRequest>,
) -> Result<Json<ChatResponse>, (StatusCode, String)> {
    let message = payload.message.clone();
    let session_id = payload
        .session_id
        .clone()
        .unwrap_or_else(|| "api".to_string());

    let system_prompt = r#"You are Harper, a CLI assistant. Use JSON for commands:
{"tool": "run_command", "args": {"command": "ls -la"}}
{"tool": "read_file", "args": {"filePath": "/path/to/file"}}
{"tool": "write_file", "args": {"filePath": "/path", "content": "text"}}
{"tool": "grep", "args": {"pattern": "search", "path": "."}}
{"tool": "update_plan", "args": {"explanation": "optional note", "items": [{"step": "Inspect files", "status": "in_progress"}]}}"#;

    let history = vec![
        Message {
            role: "system".to_string(),
            content: system_prompt.to_string(),
        },
        Message {
            role: "user".to_string(),
            content: message.clone(),
        },
    ];

    let response = call_llm(&state.client, &state.api_config, &history)
        .await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;

    if let Some(intent) = route_intent(&message) {
        match intent {
            crate::agent::intent::DeterministicIntent::ListChangedFiles(intent_args) => {
                let mut git_cmd = std::process::Command::new("git");
                git_cmd.arg("diff");

                if let Some(since) = intent_args.since {
                    git_cmd.arg(format!("--since={}", since));
                }
                git_cmd.arg("--name-only");

                if intent_args.ext.is_some() {
                    git_cmd.arg("--");
                    git_cmd.arg("*");
                }

                let output = git_cmd
                    .output()
                    .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;
                let files = String::from_utf8_lossy(&output.stdout);

                let result = if files.trim().is_empty() {
                    "No changed files found".to_string()
                } else {
                    format!("Changed files:\n{}", files)
                };

                let conn = state.conn.lock().map_err(|e| {
                    (
                        StatusCode::INTERNAL_SERVER_ERROR,
                        format!("Lock error: {:?}", e),
                    )
                })?;
                let _ = save_message(&conn, &session_id, "user", &message);
                let _ = save_message(&conn, &session_id, "assistant", &result);
                drop(conn);

                return Ok(Json(ChatResponse {
                    message: result,
                    session_id,
                    status: "completed".to_string(),
                    pending_id: None,
                    pending_tools: None,
                }));
            }
        }
    }

    let conn = state.conn.lock().map_err(|e| {
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            format!("Lock error: {:?}", e),
        )
    })?;
    let _ = save_message(&conn, &session_id, "user", &message);
    drop(conn);

    let final_response = if let Ok(json) = serde_json::from_str::<serde_json::Value>(&response) {
        if let Some(tool) = json.get("tool").and_then(|t| t.as_str()) {
            let args = json.get("args").and_then(|a| a.as_object());

            if tool == "run_command" {
                if let Some(cmd) = args.and_then(|a| a.get("command")).and_then(|c| c.as_str()) {
                    let cmd_str = cmd.to_string();
                    let session_clone = session_id.clone();

                    let output = std::process::Command::new("sh")
                        .arg("-c")
                        .arg(&cmd_str)
                        .output()
                        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;

                    let stdout = output.stdout.clone();
                    let stderr = output.stderr.clone();
                    let exit_code = output.status.code();

                    let output_str = if exit_code == Some(0) {
                        format!("$ {}\n{}", cmd_str, String::from_utf8_lossy(&stdout))
                    } else {
                        format!(
                            "$ {}\nError (exit {:?}):\n{}",
                            cmd_str,
                            exit_code,
                            String::from_utf8_lossy(&stderr)
                        )
                    };

                    let conn = state.conn.lock().map_err(|e| {
                        (
                            StatusCode::INTERNAL_SERVER_ERROR,
                            format!("Lock error: {:?}", e),
                        )
                    })?;
                    let _ = save_message(&conn, &session_clone, "assistant", &output_str);
                    let record = CommandLogRecord {
                        session_id: Some(session_clone),
                        command: cmd_str,
                        source: "api_chat".to_string(),
                        requires_approval: false,
                        approved: true,
                        status: "completed".to_string(),
                        exit_code,
                        duration_ms: None,
                        stdout_preview: Some(String::from_utf8_lossy(&stdout).to_string()),
                        stderr_preview: Some(String::from_utf8_lossy(&stderr).to_string()),
                        error_message: None,
                    };
                    let _ = crate::memory::storage::insert_command_log(&conn, &record);
                    drop(conn);

                    return Ok(Json(ChatResponse {
                        message: output_str,
                        session_id,
                        status: "completed".to_string(),
                        pending_id: None,
                        pending_tools: None,
                    }));
                }
            }

            if tool == "read_file" {
                if let Some(path) = args
                    .and_then(|a| a.get("filePath"))
                    .and_then(|p| p.as_str())
                {
                    let path_str = path.to_string();
                    let content = std::fs::read_to_string(&path_str).map_err(|e| {
                        (
                            StatusCode::INTERNAL_SERVER_ERROR,
                            format!("Read error: {}", e),
                        )
                    })?;

                    let truncated = if content.len() > 50000 {
                        format!(
                            "{}...\n(truncated {} bytes)",
                            &content[..50000],
                            content.len()
                        )
                    } else {
                        content
                    };

                    let conn = state.conn.lock().map_err(|e| {
                        (
                            StatusCode::INTERNAL_SERVER_ERROR,
                            format!("Lock error: {:?}", e),
                        )
                    })?;
                    let _ = save_message(&conn, &session_id, "assistant", &truncated);
                    let record = CommandLogRecord {
                        session_id: Some(session_id.clone()),
                        command: format!("read_file {}", path_str),
                        source: "api_chat".to_string(),
                        requires_approval: false,
                        approved: true,
                        status: "completed".to_string(),
                        exit_code: Some(0),
                        duration_ms: None,
                        stdout_preview: Some(truncated.clone()),
                        stderr_preview: None,
                        error_message: None,
                    };
                    let _ = crate::memory::storage::insert_command_log(&conn, &record);
                    drop(conn);

                    return Ok(Json(ChatResponse {
                        message: truncated,
                        session_id,
                        status: "completed".to_string(),
                        pending_id: None,
                        pending_tools: None,
                    }));
                }
            }

            if tool == "write_file" {
                if let Some(path) = args
                    .and_then(|a| a.get("filePath"))
                    .and_then(|p| p.as_str())
                {
                    if let Some(content) =
                        args.and_then(|a| a.get("content")).and_then(|c| c.as_str())
                    {
                        let path_str = path.to_string();
                        let content_str = content.to_string();

                        std::fs::write(&path_str, &content_str).map_err(|e| {
                            (
                                StatusCode::INTERNAL_SERVER_ERROR,
                                format!("Write error: {}", e),
                            )
                        })?;

                        let output_str =
                            format!("Written {} bytes to {}", content_str.len(), path_str);

                        let conn = state.conn.lock().map_err(|e| {
                            (
                                StatusCode::INTERNAL_SERVER_ERROR,
                                format!("Lock error: {:?}", e),
                            )
                        })?;
                        let _ = save_message(&conn, &session_id, "assistant", &output_str);
                        let record = CommandLogRecord {
                            session_id: Some(session_id.clone()),
                            command: format!("write_file {}", path_str),
                            source: "api_chat".to_string(),
                            requires_approval: false,
                            approved: true,
                            status: "completed".to_string(),
                            exit_code: Some(0),
                            duration_ms: None,
                            stdout_preview: Some(output_str.clone()),
                            stderr_preview: None,
                            error_message: None,
                        };
                        let _ = crate::memory::storage::insert_command_log(&conn, &record);
                        drop(conn);

                        return Ok(Json(ChatResponse {
                            message: output_str,
                            session_id,
                            status: "completed".to_string(),
                            pending_id: None,
                            pending_tools: None,
                        }));
                    }
                }
            }

            if tool == "grep" {
                if let Some(pattern) = args.and_then(|a| a.get("pattern")).and_then(|p| p.as_str())
                {
                    if let Some(path) = args.and_then(|a| a.get("path")).and_then(|p| p.as_str()) {
                        let pattern_str = pattern.to_string();
                        let path_str = path.to_string();

                        let output = std::process::Command::new("grep")
                            .arg("-n")
                            .arg("-r")
                            .arg(&pattern_str)
                            .arg(&path_str)
                            .output()
                            .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;

                        let stdout = String::from_utf8_lossy(&output.stdout);
                        let stderr = String::from_utf8_lossy(&output.stderr);
                        let exit_code = output.status.code();

                        let output_str = if stdout.is_empty() {
                            format!("No matches found for '{}' in {}", pattern_str, path_str)
                        } else {
                            format!("{}\n(matched {} lines)", stdout, stdout.lines().count())
                        };

                        let conn = state.conn.lock().map_err(|e| {
                            (
                                StatusCode::INTERNAL_SERVER_ERROR,
                                format!("Lock error: {:?}", e),
                            )
                        })?;
                        let _ = save_message(&conn, &session_id, "assistant", &output_str);
                        let stderr_str = stderr.to_string();
                        let record = CommandLogRecord {
                            session_id: Some(session_id.clone()),
                            command: format!("grep {} {}", pattern_str, path_str),
                            source: "api_chat".to_string(),
                            requires_approval: false,
                            approved: true,
                            status: "completed".to_string(),
                            exit_code,
                            duration_ms: None,
                            stdout_preview: Some(stdout.to_string()),
                            stderr_preview: if stderr_str.is_empty() {
                                None
                            } else {
                                Some(stderr_str)
                            },
                            error_message: None,
                        };
                        let _ = crate::memory::storage::insert_command_log(&conn, &record);
                        drop(conn);

                        return Ok(Json(ChatResponse {
                            message: output_str,
                            session_id,
                            status: "completed".to_string(),
                            pending_id: None,
                            pending_tools: None,
                        }));
                    }
                }
            }

            if tool == "update_plan" {
                let args_json = json
                    .get("args")
                    .cloned()
                    .unwrap_or_else(|| serde_json::json!({}));
                let conn = state.conn.lock().map_err(|e| {
                    (
                        StatusCode::INTERNAL_SERVER_ERROR,
                        format!("Lock error: {:?}", e),
                    )
                })?;
                let output_str = crate::tools::plan::update_plan(&conn, &session_id, &args_json)
                    .map_err(|e| (StatusCode::BAD_REQUEST, e.to_string()))?;
                let _ = save_message(&conn, &session_id, "assistant", &output_str);
                drop(conn);

                return Ok(Json(ChatResponse {
                    message: output_str,
                    session_id,
                    status: "completed".to_string(),
                    pending_id: None,
                    pending_tools: None,
                }));
            }
        }
        response.trim().to_string()
    } else {
        response.trim().to_string()
    };

    let conn = state.conn.lock().map_err(|e| {
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            format!("Lock error: {:?}", e),
        )
    })?;
    let _ = save_message(&conn, &session_id, "assistant", &final_response);
    drop(conn);

    Ok(Json(ChatResponse {
        message: final_response,
        session_id,
        status: "completed".to_string(),
        pending_id: None,
        pending_tools: None,
    }))
}

pub async fn review_code(
    State(state): State<Arc<ServerState>>,
    Json(payload): Json<ReviewRequest>,
) -> Result<Json<ReviewResponse>, (StatusCode, String)> {
    validate_review_request(&payload).map_err(into_http_error)?;

    let review = generate_review(&state.client, &state.api_config, &payload)
        .await
        .map_err(into_http_error)?;

    Ok(Json(review))
}

fn validate_review_request(request: &ReviewRequest) -> HarperResult<()> {
    if request.file_path.trim().is_empty() {
        return Err(HarperError::Validation(
            "file_path must not be empty".to_string(),
        ));
    }

    if request.content.trim().is_empty() {
        return Err(HarperError::Validation(
            "content must not be empty".to_string(),
        ));
    }

    if let Some(max_findings) = request.max_findings {
        if max_findings == 0 {
            return Err(HarperError::Validation(
                "max_findings must be greater than 0".to_string(),
            ));
        }
    }

    Ok(())
}

async fn generate_review(
    client: &Client,
    api_config: &ApiConfig,
    request: &ReviewRequest,
) -> HarperResult<ReviewResponse> {
    let max_findings = request.max_findings.unwrap_or(8).clamp(1, 20);
    let selection_text = request
        .selection
        .as_ref()
        .map(format_range)
        .unwrap_or_else(|| "entire file".to_string());
    let instructions = request.instructions.as_deref().unwrap_or(
        "Focus on correctness, regressions, missing validation, and concrete fix suggestions.",
    );

    let system_prompt = format!(
        "You are Harper's code review engine. Review code like a senior engineer.
Return JSON only with this exact schema:
{{
  \"summary\": \"short review summary\",
  \"findings\": [
    {{
      \"title\": \"brief title\",
      \"severity\": \"error|warning|info\",
      \"message\": \"clear explanation of the issue and why it matters\",
      \"range\": {{
        \"start_line\": 1,
        \"start_column\": 1,
        \"end_line\": 1,
        \"end_column\": 1
      }},
      \"suggestion\": {{
        \"description\": \"optional fix description\",
        \"replacement\": \"replacement text for the selected range\"
      }}
    }}
  ]
}}
Rules:
- Return only valid JSON, no markdown fences.
- Use 1-based line and column numbers.
- Report at most {} findings.
- Only include findings you can support from the provided code.
- Omit the suggestion field when you do not have a precise replacement.",
        max_findings
    );

    let user_prompt = format!(
        "Review this file for IDE inline diagnostics.
File: {}
Language: {}
Workspace root: {}
Selection: {}
Instructions: {}

Code with line numbers:
{}",
        request.file_path,
        request.language.as_deref().unwrap_or("unknown"),
        request.workspace_root.as_deref().unwrap_or("unknown"),
        selection_text,
        instructions,
        add_line_numbers(&request.content)
    );

    let messages = vec![
        Message {
            role: "system".to_string(),
            content: system_prompt,
        },
        Message {
            role: "user".to_string(),
            content: user_prompt,
        },
    ];

    let raw = call_llm(client, api_config, &messages).await?;
    let cleaned = extract_json_payload(&raw);
    let review_payload: ModelReviewResponse = serde_json::from_str(&cleaned)
        .map_err(|e| HarperError::Api(format!("Failed to parse review response: {}", e)))?;
    let mut review = ReviewResponse {
        summary: review_payload.summary,
        findings: review_payload.findings,
        model: api_config.model_name.clone(),
    };

    if review.findings.len() > max_findings {
        review.findings.truncate(max_findings);
    }

    for finding in &mut review.findings {
        normalize_finding_range(finding, &request.content);
    }
    Ok(review)
}

fn extract_json_payload(raw: &str) -> String {
    let trimmed = raw.trim();
    if let Some(stripped) = trimmed.strip_prefix("```") {
        let without_lang = stripped
            .split_once('\n')
            .map(|(_, rest)| rest)
            .unwrap_or(stripped);
        let without_fence = without_lang.strip_suffix("```").unwrap_or(without_lang);
        return without_fence.trim().to_string();
    }

    let start = trimmed.find('{');
    let end = trimmed.rfind('}');
    match (start, end) {
        (Some(start_idx), Some(end_idx)) if start_idx <= end_idx => {
            trimmed[start_idx..=end_idx].to_string()
        }
        _ => trimmed.to_string(),
    }
}

fn add_line_numbers(content: &str) -> String {
    content
        .lines()
        .enumerate()
        .map(|(index, line)| format!("{:>4} | {}", index + 1, line))
        .collect::<Vec<_>>()
        .join("\n")
}

fn format_range(range: &ReviewRange) -> String {
    format!(
        "{}:{}-{}:{}",
        range.start_line, range.start_column, range.end_line, range.end_column
    )
}

fn normalize_finding_range(finding: &mut CodeReviewFinding, content: &str) {
    let line_count = content.lines().count().max(1);
    let range = &mut finding.range;

    range.start_line = range.start_line.clamp(1, line_count);
    range.end_line = range.end_line.clamp(range.start_line, line_count);
    range.start_column = range.start_column.max(1);
    range.end_column = range.end_column.max(range.start_column);
}

fn into_http_error(error: HarperError) -> (StatusCode, String) {
    match error {
        HarperError::Validation(message) => (StatusCode::BAD_REQUEST, message),
        HarperError::Api(message) => (StatusCode::BAD_GATEWAY, message),
        HarperError::Config(message) => (StatusCode::INTERNAL_SERVER_ERROR, message),
        HarperError::Database(message) => (StatusCode::INTERNAL_SERVER_ERROR, message),
        HarperError::Command(message) => (StatusCode::INTERNAL_SERVER_ERROR, message),
        HarperError::Io(message) => (StatusCode::INTERNAL_SERVER_ERROR, message),
        HarperError::File(message) => (StatusCode::INTERNAL_SERVER_ERROR, message),
        HarperError::Crypto(message) => (StatusCode::INTERNAL_SERVER_ERROR, message),
        HarperError::Mcp(message) => (StatusCode::BAD_GATEWAY, message),
        HarperError::WebSearch(message) => (StatusCode::BAD_GATEWAY, message),
        HarperError::Firmware(error) => (StatusCode::INTERNAL_SERVER_ERROR, error.to_string()),
        HarperError::Sandbox(error) => (StatusCode::FORBIDDEN, error.to_string()),
    }
}

/// Creates the Axum router with all API routes configured
pub fn create_router(
    conn: Arc<Mutex<Connection>>,
    api_config: ApiConfig,
    exec_policy: ExecPolicyConfig,
) -> Router {
    let state = Arc::new(ServerState {
        conn,
        api_config,
        client: Client::new(),
        exec_policy,
    });

    Router::new()
        .route("/health", get(health))
        .route("/api/sessions", get(list_sessions))
        .route("/api/sessions/{id}", get(get_session))
        .route("/api/sessions/{id}", delete(delete_session))
        .route("/api/chat", post(chat_endpoint))
        .route("/api/approvals/{session_id}", get(list_pending_approvals))
        .route("/api/approvals/{session_id}", post(approve_command))
        .route("/api/chat/approve/{pending_id}", post(approve_pending_tool))
        .route("/api/review", post(review_code))
        .with_state(state)
}

pub async fn run_server(
    addr: &str,
    conn: Arc<Mutex<Connection>>,
    api_config: ApiConfig,
    exec_policy: ExecPolicyConfig,
) -> HarperResult<()> {
    let router = create_router(conn, api_config, exec_policy);

    let listener = tokio::net::TcpListener::bind(addr).await?;
    log::info!("Harper API server running on {}", addr);

    axum::serve(listener, router).await?;
    Ok(())
}

pub async fn list_pending_approvals(
    State(state): State<Arc<ServerState>>,
    Path(session_id): Path<String>,
) -> Result<Json<Vec<serde_json::Value>>, (StatusCode, String)> {
    let conn = state.conn.lock().map_err(|e| {
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            format!("Lock error: {:?}", e),
        )
    })?;

    let mut stmt = conn.prepare(
        "SELECT id, command, source, created_at FROM command_logs WHERE session_id = ?1 AND status = 'pending' AND requires_approval = 1"
    ).map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;

    let rows = stmt
        .query_map([&session_id], |row| {
            Ok(serde_json::json!({
                "id": row.get::<_, i64>(0)?,
                "command": row.get::<_, String>(1)?,
                "source": row.get::<_, String>(2)?,
                "created_at": row.get::<_, String>(3)?,
            }))
        })
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;

    let results: Vec<serde_json::Value> = rows.filter_map(|r| r.ok()).collect();

    Ok(Json(results))
}

pub async fn approve_command(
    State(state): State<Arc<ServerState>>,
    Path(session_id): Path<String>,
    Json(payload): Json<ApprovalRequest>,
) -> Result<Json<serde_json::Value>, (StatusCode, String)> {
    let conn = state.conn.lock().map_err(|e| {
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            format!("Lock error: {:?}", e),
        )
    })?;

    let status = if payload.approved {
        "completed"
    } else {
        "rejected"
    };

    conn.execute(
        "UPDATE command_logs SET approved = ?1, status = ?2 WHERE session_id = ?3 AND status = 'pending' AND requires_approval = 1 ORDER BY id DESC LIMIT 1",
        params![payload.approved as i32, status, session_id],
    ).map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;

    Ok(Json(serde_json::json!({
        "success": true,
        "session_id": session_id,
        "approved": payload.approved,
    })))
}

pub async fn approve_pending_tool(
    State(state): State<Arc<ServerState>>,
    Path(pending_id): Path<String>,
    Json(payload): Json<ApprovalRequest>,
) -> Result<Json<ChatResponse>, (StatusCode, String)> {
    let conn = state.conn.lock().map_err(|e| {
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            format!("Lock error: {:?}", e),
        )
    })?;

    let tool = crate::memory::storage::get_pending_tool(&conn, &pending_id)
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?
        .ok_or((StatusCode::NOT_FOUND, "Pending tool not found".to_string()))?;
    let session_id = tool.session_id.clone();

    if !payload.approved {
        crate::memory::storage::delete_pending_tool(&conn, &pending_id)
            .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;
        let _ = save_message(&conn, &session_id, "assistant", "Tool execution rejected");
        drop(conn);
        return Ok(Json(ChatResponse {
            message: "Tool execution rejected".to_string(),
            session_id,
            status: "rejected".to_string(),
            pending_id: None,
            pending_tools: None,
        }));
    }

    let args: serde_json::Value = serde_json::from_str(&tool.args)
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;

    let result = match tool.tool.as_str() {
        "run_command" => {
            if let Some(cmd) = args.get("command").and_then(|c| c.as_str()) {
                let output = std::process::Command::new("sh")
                    .arg("-c")
                    .arg(cmd)
                    .output()
                    .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;
                let exit_code = output.status.code();
                let stdout = String::from_utf8_lossy(&output.stdout);
                let stderr = String::from_utf8_lossy(&output.stderr);
                let output_str = if exit_code == Some(0) {
                    format!("$ {}\n{}", cmd, stdout)
                } else {
                    format!("$ {}\nError (exit {:?}):\n{}", cmd, exit_code, stderr)
                };
                let record = CommandLogRecord {
                    session_id: Some(session_id.clone()),
                    command: cmd.to_string(),
                    source: "http".to_string(),
                    requires_approval: true,
                    approved: true,
                    status: "completed".to_string(),
                    exit_code,
                    duration_ms: None,
                    stdout_preview: Some(stdout.to_string()),
                    stderr_preview: if stderr.is_empty() {
                        None
                    } else {
                        Some(stderr.to_string())
                    },
                    error_message: None,
                };
                let _ = crate::memory::storage::insert_command_log(&conn, &record);
                output_str
            } else {
                "Invalid command".to_string()
            }
        }
        "read_file" => {
            if let Some(path) = args.get("filePath").and_then(|p| p.as_str()) {
                let content = std::fs::read_to_string(path).map_err(|e| {
                    (
                        StatusCode::INTERNAL_SERVER_ERROR,
                        format!("Read error: {}", e),
                    )
                })?;
                let truncated = if content.len() > 50000 {
                    format!(
                        "{}...\n(truncated {} bytes)",
                        &content[..50000],
                        content.len()
                    )
                } else {
                    content
                };
                truncated
            } else {
                "Invalid path".to_string()
            }
        }
        "write_file" => {
            if let Some(path) = args.get("filePath").and_then(|p| p.as_str()) {
                if let Some(content) = args.get("content").and_then(|c| c.as_str()) {
                    std::fs::write(path, content).map_err(|e| {
                        (
                            StatusCode::INTERNAL_SERVER_ERROR,
                            format!("Write error: {}", e),
                        )
                    })?;
                    format!("Written {} bytes to {}", content.len(), path)
                } else {
                    "Invalid content".to_string()
                }
            } else {
                "Invalid path".to_string()
            }
        }
        "grep" => {
            if let Some(pattern) = args.get("pattern").and_then(|p| p.as_str()) {
                if let Some(path) = args.get("path").and_then(|p| p.as_str()) {
                    let output = std::process::Command::new("grep")
                        .arg("-n")
                        .arg("-r")
                        .arg(pattern)
                        .arg(path)
                        .output()
                        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;
                    let stdout = String::from_utf8_lossy(&output.stdout);
                    if stdout.is_empty() {
                        format!("No matches found for '{}' in {}", pattern, path)
                    } else {
                        format!("{}\n(matched {} lines)", stdout, stdout.lines().count())
                    }
                } else {
                    "Invalid path".to_string()
                }
            } else {
                "Invalid pattern".to_string()
            }
        }
        _ => "Unknown tool".to_string(),
    };

    crate::memory::storage::delete_pending_tool(&conn, &pending_id)
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;
    let _ = save_message(&conn, &session_id, "assistant", &result);
    drop(conn);

    Ok(Json(ChatResponse {
        message: result,
        session_id,
        status: "completed".to_string(),
        pending_id: None,
        pending_tools: None,
    }))
}

#[cfg(test)]
mod tests {
    use super::{
        extract_json_payload, normalize_finding_range, review_code, CodeReviewFinding,
        CodeSuggestion, ReviewRange, ReviewRequest, ServerState,
    };
    use crate::core::{ApiConfig, ApiProvider};
    use axum::extract::State;
    use axum::Json;
    use reqwest::Client;
    use rusqlite::Connection;
    use std::sync::{Arc, Mutex};

    #[test]
    fn extracts_json_from_fenced_block() {
        let raw = "```json\n{\"summary\":\"ok\",\"findings\":[]}\n```";
        assert_eq!(
            extract_json_payload(raw),
            "{\"summary\":\"ok\",\"findings\":[]}"
        );
    }

    #[test]
    fn clamps_review_ranges_to_file_bounds() {
        let mut finding = CodeReviewFinding {
            title: "Issue".to_string(),
            severity: "warning".to_string(),
            message: "Details".to_string(),
            range: ReviewRange {
                start_line: 0,
                start_column: 0,
                end_line: 99,
                end_column: 0,
            },
            suggestion: Some(CodeSuggestion {
                description: "Fix".to_string(),
                replacement: "value".to_string(),
            }),
        };

        normalize_finding_range(&mut finding, "first\nsecond");

        assert_eq!(finding.range.start_line, 1);
        assert_eq!(finding.range.start_column, 1);
        assert_eq!(finding.range.end_line, 2);
        assert_eq!(finding.range.end_column, 1);
    }

    #[tokio::test]
    async fn review_endpoint_rejects_empty_content() {
        let conn = Connection::open_in_memory().expect("in-memory db");
        crate::memory::storage::init_db(&conn).expect("init db");

        let state = Arc::new(ServerState {
            conn: Arc::new(Mutex::new(conn)),
            api_config: ApiConfig {
                provider: ApiProvider::OpenAI,
                api_key: "test-key".to_string(),
                base_url: "https://api.openai.com/v1/chat/completions".to_string(),
                model_name: "gpt-4".to_string(),
            },
            client: Client::new(),
            exec_policy: crate::runtime::config::ExecPolicyConfig::default(),
        });

        let request = ReviewRequest {
            file_path: "src/main.rs".to_string(),
            content: String::new(),
            language: Some("rust".to_string()),
            workspace_root: None,
            instructions: None,
            selection: None,
            max_findings: Some(5),
        };

        let result = review_code(State(state), Json(request)).await;
        let error = result.expect_err("request should be rejected");
        assert_eq!(error.0, axum::http::StatusCode::BAD_REQUEST);
        assert!(error.1.contains("content must not be empty"));
    }
}
