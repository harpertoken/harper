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

use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use tokio::sync::{mpsc, oneshot};

use crossterm::terminal::{
    disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen,
};
use crossterm::{cursor, execute};
use ratatui::backend::CrosstermBackend;
use ratatui::Terminal;

use super::app::{AppState, ApprovalState, ChatState, CommandOutputState, TuiApp};
use super::auth;
use super::events::{self, EventResult};
use super::theme::Theme;
use super::widgets;
use harper_core::agent::chat::ChatService;
use harper_core::core::io_traits::{RuntimeEventSink, UserApproval};
use harper_core::core::ApiConfig;
use harper_core::memory::session_service::SessionService;
use harper_core::runtime::config::ExecPolicyConfig;
use harper_core::{PlanState, ResolvedAgents, SessionStateView};
use rusqlite::Connection;

use async_trait::async_trait;
use harper_core::core::error::HarperResult;
use std::path::Path;

/// Type alias for the approval message sent via channels
type ApprovalMessage = (String, String, Arc<Mutex<Option<oneshot::Sender<bool>>>>);

/// Approval provider for TUI that uses channels to communicate with the UI loop
pub struct TuiApproval {
    approval_tx: mpsc::Sender<ApprovalMessage>,
}

pub struct TuiRuntimeEvents {
    ui_tx: mpsc::Sender<UiUpdate>,
}

#[async_trait]
impl UserApproval for TuiApproval {
    async fn approve(&self, prompt: &str, command: &str) -> HarperResult<bool> {
        let (tx, rx) = oneshot::channel();
        self.approval_tx
            .send((
                prompt.to_string(),
                command.to_string(),
                Arc::new(Mutex::new(Some(tx))),
            ))
            .await
            .map_err(|_| {
                harper_core::core::error::HarperError::Command(
                    "Failed to send approval request".to_string(),
                )
            })?;

        rx.await.map_err(|_| {
            harper_core::core::error::HarperError::Command(
                "Failed to receive approval response".to_string(),
            )
        })
    }
}

#[async_trait]
impl RuntimeEventSink for TuiRuntimeEvents {
    async fn plan_updated(&self, session_id: &str, plan: Option<PlanState>) -> HarperResult<()> {
        self.ui_tx
            .send(UiUpdate::PlanUpdated {
                session_id: session_id.to_string(),
                active_plan: plan,
            })
            .await
            .map_err(|_| {
                harper_core::core::error::HarperError::Command(
                    "Failed to send runtime plan update".to_string(),
                )
            })?;
        Ok(())
    }

    async fn agents_updated(
        &self,
        session_id: &str,
        agents: Option<ResolvedAgents>,
    ) -> HarperResult<()> {
        self.ui_tx
            .send(UiUpdate::AgentsUpdated {
                session_id: session_id.to_string(),
                active_agents: agents,
            })
            .await
            .map_err(|_| {
                harper_core::core::error::HarperError::Command(
                    "Failed to send runtime AGENTS update".to_string(),
                )
            })?;
        Ok(())
    }

    async fn activity_updated(&self, session_id: &str, status: Option<String>) -> HarperResult<()> {
        self.ui_tx
            .send(UiUpdate::ActivityUpdated {
                session_id: session_id.to_string(),
                status,
            })
            .await
            .map_err(|_| {
                harper_core::core::error::HarperError::Command(
                    "Failed to send runtime activity update".to_string(),
                )
            })?;
        Ok(())
    }

    async fn command_output_updated(
        &self,
        session_id: &str,
        command: String,
        chunk: String,
        is_error: bool,
        done: bool,
    ) -> HarperResult<()> {
        self.ui_tx
            .send(UiUpdate::CommandOutputUpdated {
                session_id: session_id.to_string(),
                command,
                chunk,
                is_error,
                done,
            })
            .await
            .map_err(|_| {
                harper_core::core::error::HarperError::Command(
                    "Failed to send runtime command output update".to_string(),
                )
            })?;
        Ok(())
    }
}

/// Messages sent to the background chat worker
enum WorkerMsg {
    SendMessage {
        user_msg: String,
        session_id: String,
        web_search: bool,
    },
}

/// Messages sent from the background chat worker to the UI
enum UiUpdate {
    MessageProcessed(SessionStateView),
    ActivityUpdated {
        session_id: String,
        status: Option<String>,
    },
    CommandOutputUpdated {
        session_id: String,
        command: String,
        chunk: String,
        is_error: bool,
        done: bool,
    },
    PlanUpdated {
        session_id: String,
        active_plan: Option<PlanState>,
    },
    AgentsUpdated {
        session_id: String,
        active_agents: Option<ResolvedAgents>,
    },
    SidebarEntries {
        entries: Vec<String>,
    },
    Error(String),
}

/// Helper function to spawn async sidebar gathering task
fn spawn_sidebar_gathering(chat_state: &ChatState, ui_tx: &mpsc::Sender<UiUpdate>) {
    let messages = chat_state.messages.clone();
    let ui_tx_clone = ui_tx.clone();
    tokio::spawn(async move {
        let entries = super::app::gather_sidebar_entries_async(&messages).await;
        let _ = ui_tx_clone.send(UiUpdate::SidebarEntries { entries }).await;
    });
}

pub async fn run_tui(
    conn: &Connection,
    api_config: &ApiConfig,
    session_service: &SessionService<'_>,
    theme: &Theme,
    custom_commands: HashMap<String, String>,
    exec_policy: &ExecPolicyConfig,
    server_base_url: Option<String>,
) -> Result<(), Box<dyn std::error::Error>> {
    // Set up terminal
    enable_raw_mode()?;
    let mut stdout = std::io::stdout();
    execute!(stdout, EnterAlternateScreen, cursor::Hide)?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    let mut app = TuiApp::new();
    app.model_label = format!("{:?} / {}", api_config.provider, api_config.model_name);
    app.auth_session = auth::load_auth_session();
    app.auth_server_base_url = server_base_url.clone();
    let auth_client = reqwest::Client::new();

    // Set up channels for background worker
    let (worker_tx, mut worker_rx) = mpsc::channel::<WorkerMsg>(10);
    let (ui_tx, mut ui_rx) = mpsc::channel::<UiUpdate>(10);
    let (approval_tx, mut approval_rx) = mpsc::channel::<ApprovalMessage>(1);

    // Clone data for worker
    let worker_api_config = api_config.clone();
    let worker_custom_commands = custom_commands.clone();
    let worker_exec_policy = exec_policy.clone();
    let db_path = conn
        .path()
        .and_then(|p| Path::new(p).to_str().map(|s| s.to_string()));

    // Wrap MCP client in Arc if present
    // Note: This requires McpClient to be thread-safe (Send + Sync)
    // We assume McpClient is Arc-cloneable or we wrap it.
    // For now, if McpClient is not cloneable, we'd need a proxy.
    // Let's assume the user can provide an Arc or we wrap it here if we had access to the type.
    // Since we only have Option<&McpClient>, we can't easily Arc it for the worker thread.
    // ARCHITECTURAL NOTE: To support MCP in TUI worker, McpClient should be passed as Arc<McpClient>.

    // Spawn background worker in a separate thread to handle non-Send Connection
    let ui_tx_clone = ui_tx.clone();
    std::thread::spawn(move || {
        let rt = tokio::runtime::Builder::new_current_thread()
            .enable_all()
            .build()
            .expect("Failed to build worker runtime");

        rt.block_on(async {
            // Open worker connection
            let worker_conn = if let Some(path) = db_path {
                Connection::open(path).expect("Worker failed to open database")
            } else {
                Connection::open_in_memory().expect("Worker failed to open in-memory database")
            };

            let mut api_cache = harper_core::core::cache::new_api_cache();
            let approver = Arc::new(TuiApproval { approval_tx });
            let runtime_events = Arc::new(TuiRuntimeEvents {
                ui_tx: ui_tx_clone.clone(),
            });

            while let Some(msg) = worker_rx.recv().await {
                match msg {
                    WorkerMsg::SendMessage {
                        user_msg,
                        session_id,
                        web_search,
                    } => {
                        let mut chat_service = ChatService::new(
                            &worker_conn,
                            &worker_api_config,
                            None, // TODO: Support MCP in worker thread
                            Some(&mut api_cache),
                            None,
                            worker_custom_commands.clone(),
                            worker_exec_policy.clone(),
                        )
                        .with_approver(approver.clone())
                        .with_runtime_events(runtime_events.clone());

                        // Load existing history
                        let mut history =
                            harper_core::memory::storage::load_history(&worker_conn, &session_id)
                                .unwrap_or_default();

                        match chat_service
                            .send_message(&user_msg, &mut history, web_search, &session_id)
                            .await
                        {
                            Ok(_) => {
                                let session_service = SessionService::new(&worker_conn);
                                let session_view = session_service
                                    .load_session_state_view(&session_id)
                                    .unwrap_or_else(|_| SessionStateView {
                                        session_id: session_id.clone(),
                                        user_id: None,
                                        messages: history.clone(),
                                        plan: None,
                                        agents: None,
                                        agents_rendered: None,
                                        agents_effective_rendered: None,
                                    });
                                let _ = ui_tx_clone
                                    .send(UiUpdate::MessageProcessed(session_view))
                                    .await;
                            }
                            Err(e) => {
                                let _ = ui_tx_clone.send(UiUpdate::Error(e.to_string())).await;
                            }
                        }
                    }
                }
            }
        });
    });

    loop {
        app.refresh_activity_status();
        terminal.draw(|f| widgets::draw(f, &app, theme))?;

        // Handle both UI events and worker updates
        tokio::select! {
            // UI Events - Bubble up errors from terminal IO
            event_res = async {
                if crossterm::event::poll(std::time::Duration::from_millis(100))? {
                    Ok::<_, std::io::Error>(Some(crossterm::event::read()?))
                } else {
                    Ok(None)
                }
            } => {
                let event = event_res?; // Proper error propagation
                if let Some(event) = event {
                    match events::handle_event(event, &mut app, session_service) {
                        EventResult::Quit => break,
                        EventResult::SendMessage(msg) => {
                            if let AppState::Chat(chat_state) = &mut app.state {
                                if let Some(command) = auth::parse_tui_auth_command(&msg) {
                                    match command {
                                        auth::TuiAuthCommand::Login { provider } => {
                                            let Some(base_url) = app.auth_server_base_url.clone() else {
                                                app.set_error_message("TUI auth requires the Harper server to be enabled".to_string());
                                                continue;
                                            };
                                            match auth::start_tui_auth_flow(&auth_client, &base_url, &provider).await {
                                                Ok(flow) => {
                                                    app.auth_flow_id = Some(flow.flow_id.clone());
                                                    app.auth_last_poll_at = None;
                                                    match auth::launch_browser(&flow.login_url) {
                                                        Ok(_) => app.set_status_message(format!("Opened browser for {} sign-in", provider)),
                                                        Err(_) => app.set_info_message(format!(
                                                            "Open this URL to sign in:\n{}",
                                                            flow.login_url
                                                        )),
                                                    }
                                                    app.set_activity_status(Some("waiting for browser sign-in".to_string()));
                                                }
                                                Err(err) => app.set_error_message(format!("Auth login failed: {}", err)),
                                            }
                                            continue;
                                        }
                                        auth::TuiAuthCommand::Logout => {
                                            if let Err(err) = auth::clear_auth_session() {
                                                app.set_error_message(format!("Auth logout failed: {}", err));
                                            } else {
                                                app.auth_session = None;
                                                app.auth_flow_id = None;
                                                app.auth_last_poll_at = None;
                                                app.set_status_message("Cleared local TUI auth session".to_string());
                                            }
                                            continue;
                                        }
                                        auth::TuiAuthCommand::Status => {
                                            let status = app.auth_session.as_ref().map(|session| {
                                                session.user.email.clone().unwrap_or_else(|| session.user.user_id.clone())
                                            }).unwrap_or_else(|| "not signed in".to_string());
                                            app.set_info_message(format!("TUI auth status: {}", status));
                                            continue;
                                        }
                                    }
                                }
                                let session_id = chat_state.session_id.clone();
                                let web_search = chat_state.web_search_enabled;

                                // optimistic update
                                chat_state.messages.push(harper_core::core::Message {
                                    role: "user".to_string(),
                                    content: msg.clone(),
                                });
                                chat_state.awaiting_response = true;
                                chat_state.command_output = None;
                                app.set_activity_status(Some("thinking".to_string()));

                                let _ = worker_tx.send(WorkerMsg::SendMessage {
                                    user_msg: msg,
                                    session_id,
                                    web_search,
                                }).await;
                            }
                        }
                        EventResult::LoadSessions => {
                            if let (Some(base_url), Some(session)) = (
                                app.auth_server_base_url.clone(),
                                app.auth_session.clone(),
                            ) {
                                match auth::fetch_remote_sessions(&auth_client, &base_url, &session).await {
                                    Ok(sessions) => {
                                        let session_infos = sessions
                                            .into_iter()
                                            .map(|s| super::app::SessionInfo {
                                                id: s.id.clone(),
                                                name: s.title.unwrap_or(s.id),
                                                created_at: s.created_at,
                                            })
                                            .collect();
                                        app.state = AppState::Sessions(session_infos, 0);
                                    }
                                    Err(err) => app.set_error_message(format!("Error loading remote sessions: {}", err)),
                                }
                            } else {
                                match session_service.list_sessions_data() {
                                    Ok(sessions) => {
                                        let session_infos = sessions
                                            .into_iter()
                                            .map(|s| super::app::SessionInfo {
                                                id: s.id.clone(),
                                                name: s.title.unwrap_or(s.id),
                                                created_at: s.created_at,
                                            })
                                            .collect();
                                        app.state = AppState::Sessions(session_infos, 0);
                                    }
                                    Err(e) => app.set_error_message(format!("Error loading sessions: {}", e)),
                                }
                            }
                        }
                        EventResult::OpenSession { session_id, preview } => {
                            if let (Some(base_url), Some(session)) = (
                                app.auth_server_base_url.clone(),
                                app.auth_session.clone(),
                            ) {
                                match auth::fetch_remote_session_state(&auth_client, &base_url, &session, &session_id).await {
                                    Ok(session_view) => {
                                        if preview {
                                            app.state = AppState::ViewSession(
                                                session_view.session_id,
                                                session_view.messages,
                                                0,
                                            );
                                        } else {
                                            app.state = AppState::Chat(Box::new(events::create_chat_state(
                                                session_view.session_id,
                                                session_view.messages,
                                                session_view.plan,
                                                session_view.agents,
                                            )));
                                            if let AppState::Chat(chat_state) = &mut app.state {
                                                spawn_sidebar_gathering(chat_state, &ui_tx);
                                            }
                                        }
                                    }
                                    Err(err) => app.set_error_message(format!("Error loading remote session: {}", err)),
                                }
                            } else if preview {
                                if let Ok(messages) = session_service.view_session_data(&session_id) {
                                    app.state = AppState::ViewSession(session_id, messages, 0);
                                }
                            } else {
                                match session_service.load_session_state_view(&session_id) {
                                    Ok(session_view) => {
                                        app.state = AppState::Chat(Box::new(events::create_chat_state(
                                            session_view.session_id,
                                            session_view.messages,
                                            session_view.plan,
                                            session_view.agents,
                                        )));
                                        if let AppState::Chat(chat_state) = &mut app.state {
                                            spawn_sidebar_gathering(chat_state, &ui_tx);
                                        }
                                    }
                                    Err(e) => app.set_error_message(format!("Error loading session: {}", e)),
                                }
                            }
                        }
                        EventResult::Continue => {}
                        EventResult::GatherSidebarEntries => {
                            if let AppState::Chat(chat_state) = &mut app.state {
                                spawn_sidebar_gathering(chat_state, &ui_tx);
                            }
                        }
                    }
                }
            }

            // Worker Updates
            update = ui_rx.recv() => {
                if let Some(update) = update {
                    match update {
                        UiUpdate::MessageProcessed(session_view) => {
                            let mut should_clear_activity = false;
                            if let AppState::Chat(chat_state) = &mut app.state {
                                if chat_state.session_id == session_view.session_id {
                                    chat_state.messages = session_view.messages;
                                    chat_state.awaiting_response = false;
                                    chat_state.active_plan = session_view.plan;
                                    chat_state.active_agents = session_view.agents;
                                    chat_state.refresh_plan_state();
                                    chat_state.refresh_review_state();
                                    should_clear_activity = true;
                                    if chat_state.sidebar_visible {
                                        spawn_sidebar_gathering(chat_state, &ui_tx);
                                    }
                                }
                            }
                            if should_clear_activity {
                                app.set_activity_status(None);
                            }
                        }
                        UiUpdate::ActivityUpdated { session_id, status } => {
                            let matches_session = if let AppState::Chat(chat_state) = &app.state {
                                chat_state.session_id == session_id
                            } else {
                                false
                            };
                            if matches_session {
                                app.set_activity_status(status);
                            }
                        }
                        UiUpdate::CommandOutputUpdated {
                            session_id,
                            command,
                            chunk,
                            is_error,
                            done,
                        } => {
                            if let AppState::Chat(chat_state) = &mut app.state {
                                if chat_state.session_id == session_id {
                                    let state = chat_state.command_output.get_or_insert(CommandOutputState {
                                        command: command.clone(),
                                        content: String::new(),
                                        has_error: false,
                                        done: false,
                                    });
                                    if state.command != command {
                                        *state = CommandOutputState {
                                            command,
                                            content: String::new(),
                                            has_error: is_error,
                                            done,
                                        };
                                    }
                                    state.content.push_str(&chunk);
                                    state.has_error |= is_error;
                                    state.done = done;
                                }
                            }
                        }
                        UiUpdate::PlanUpdated { session_id, active_plan } => {
                            if let AppState::Chat(chat_state) = &mut app.state {
                                if chat_state.session_id == session_id {
                                    chat_state.active_plan = active_plan;
                                    chat_state.refresh_plan_state();
                                }
                            }
                        }
                        UiUpdate::AgentsUpdated {
                            session_id,
                            active_agents,
                        } => {
                            if let AppState::Chat(chat_state) = &mut app.state {
                                if chat_state.session_id == session_id {
                                    chat_state.active_agents = active_agents;
                                }
                            }
                        }
                        UiUpdate::SidebarEntries { entries } => {
                            if let AppState::Chat(chat_state) = &mut app.state {
                                chat_state.sidebar_entries = entries;
                            }
                        }
                        UiUpdate::Error(err) => {
                            if let AppState::Chat(chat_state) = &mut app.state {
                                chat_state.awaiting_response = false;
                            }
                            app.set_activity_status(None);
                            app.set_error_message(err);
                        }
                    }
                }
            }

            // Approval Requests
            approval = approval_rx.recv() => {
                if let Some((prompt, command, tx)) = approval {
                    // Use the new pending_approval overlay instead of replacing state
                    app.set_activity_status(Some(format!("waiting approval: {}", command)));
                    app.pending_approval = Some(ApprovalState {
                        prompt,
                        command,
                        tx,
                        scroll_offset: 0,
                    });
                }
            }
        }

        if let (Some(base_url), Some(flow_id)) =
            (app.auth_server_base_url.clone(), app.auth_flow_id.clone())
        {
            let should_poll = app
                .auth_last_poll_at
                .is_none_or(|last| last.elapsed() >= std::time::Duration::from_millis(750));
            if should_poll {
                app.auth_last_poll_at = Some(std::time::Instant::now());
                match auth::poll_tui_auth_flow(&auth_client, &base_url, &flow_id).await {
                    Ok(flow) if flow.status == "complete" => {
                        if let Some(session) = flow.session {
                            match auth::save_auth_session(&session) {
                                Ok(_) => {
                                    let status = session
                                        .user
                                        .email
                                        .clone()
                                        .unwrap_or_else(|| session.user.user_id.clone());
                                    app.auth_session = Some(session);
                                    app.auth_flow_id = None;
                                    app.auth_last_poll_at = None;
                                    app.set_activity_status(None);
                                    app.set_status_message(format!("Signed in as {}", status));
                                }
                                Err(err) => app.set_error_message(format!(
                                    "Failed to save TUI auth session: {}",
                                    err
                                )),
                            }
                        }
                    }
                    Ok(_) => {}
                    Err(err) => {
                        app.auth_flow_id = None;
                        app.auth_last_poll_at = None;
                        app.set_activity_status(None);
                        app.set_error_message(format!("Auth polling failed: {}", err));
                    }
                }
            }
        }
    }

    // Restore terminal
    disable_raw_mode()?;
    execute!(terminal.backend_mut(), LeaveAlternateScreen, cursor::Show)?;

    Ok(())
}
