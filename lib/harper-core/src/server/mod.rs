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
use rusqlite::Connection;
use serde::{Deserialize, Serialize};
use std::sync::{Arc, Mutex};

use crate::core::error::HarperResult;

pub struct ServerState {
    pub conn: Arc<Mutex<Connection>>,
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
    let mut stmt = conn.prepare("SELECT id, created_at, updated_at, title FROM sessions ORDER BY updated_at DESC LIMIT 50").expect("Failed to prepare SQL statement");

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
                serde_json::from_str(&msgs).unwrap_or(serde_json::json!({
                    "messages": []
                }));
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

/// Creates the Axum router with all API routes configured
///
/// # Arguments
/// * `conn` - Database connection wrapped in Arc<Mutex<>>
///
/// # Returns
/// Configured Axum Router with health and session endpoints
pub fn create_router(conn: Arc<Mutex<Connection>>) -> Router {
    let state = Arc::new(ServerState { conn });

    Router::new()
        .route("/health", get(health))
        .route("/api/sessions", get(list_sessions))
        .route("/api/sessions/:id", get(get_session))
        .route("/api/sessions/:id", delete(delete_session))
        .route("/api/chat", post(chat))
        .with_state(state)
}

pub async fn run_server(addr: &str, conn: Arc<Mutex<Connection>>) -> HarperResult<()> {
    let router = create_router(conn);

    let listener = tokio::net::TcpListener::bind(addr).await?;
    log::info!("Harper API server running on {}", addr);

    axum::serve(listener, router).await?;
    Ok(())
}
