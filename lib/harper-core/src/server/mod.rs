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
    let conn = state.conn.lock().unwrap();
    let mut stmt = conn.prepare("SELECT id, created_at, updated_at, title FROM sessions ORDER BY updated_at DESC LIMIT 50").unwrap();

    let sessions: Vec<SessionListItem> = stmt
        .query_map([], |row| {
            Ok(SessionListItem {
                id: row.get(0)?,
                created_at: row.get(1)?,
                updated_at: row.get(2)?,
                title: row.get(3)?,
            })
        })
        .unwrap()
        .filter_map(|r| r.ok())
        .collect();

    Json(sessions)
}

pub async fn get_session(
    State(state): State<Arc<ServerState>>,
    Path(session_id): Path<String>,
) -> Result<Json<serde_json::Value>, StatusCode> {
    let conn = state.conn.lock().unwrap();

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
    let conn = state.conn.lock().unwrap();

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
