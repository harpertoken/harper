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

use crate::core::error::{HarperError, HarperResult};
use crate::core::llm_client::call_llm;
use crate::core::{ApiConfig, Message};

#[derive(Clone)]
pub struct ServerState {
    pub conn: Arc<Mutex<Connection>>,
    pub api_config: ApiConfig,
    pub client: Client,
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

    let mut stmt = conn
        .prepare("SELECT messages FROM sessions WHERE id = ?")
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    let messages: Option<String> = stmt.query_row([&session_id], |row| row.get(0)).ok();

    match messages {
        Some(msgs) => {
            let parsed: serde_json::Value =
                serde_json::from_str(&msgs).unwrap_or(serde_json::json!({ "messages": [] }));
            Ok(Json(parsed))
        }
        None => Err(StatusCode::NOT_FOUND),
    }
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

pub async fn chat(
    State(_state): State<Arc<ServerState>>,
    Json(payload): Json<ChatRequest>,
) -> Json<ChatResponse> {
    Json(ChatResponse {
        message: format!(
            "Chat endpoint: '{}'. Full chat API coming soon!",
            payload.message
        ),
        session_id: payload.session_id.unwrap_or_else(|| "new".to_string()),
    })
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
pub fn create_router(conn: Arc<Mutex<Connection>>, api_config: ApiConfig) -> Router {
    let state = Arc::new(ServerState {
        conn,
        api_config,
        client: Client::new(),
    });

    Router::new()
        .route("/health", get(health))
        .route("/api/sessions", get(list_sessions))
        .route("/api/sessions/{id}", get(get_session))
        .route("/api/sessions/{id}", delete(delete_session))
        .route("/api/chat", post(chat))
        .route("/api/review", post(review_code))
        .with_state(state)
}

pub async fn run_server(
    addr: &str,
    conn: Arc<Mutex<Connection>>,
    api_config: ApiConfig,
) -> HarperResult<()> {
    let router = create_router(conn, api_config);

    let listener = tokio::net::TcpListener::bind(addr).await?;
    log::info!("Harper API server running on {}", addr);

    axum::serve(listener, router).await?;
    Ok(())
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
