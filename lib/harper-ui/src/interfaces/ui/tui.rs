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

use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use tokio::sync::{mpsc, oneshot};

use crossterm::terminal::{
    disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen,
};
use crossterm::{cursor, execute};
use ratatui::backend::CrosstermBackend;
use ratatui::Terminal;

use harper_core::agent::chat::ChatService;
use harper_core::core::io_traits::UserApproval;
use harper_core::core::ApiConfig;
use harper_core::memory::session_service::SessionService;
use harper_core::runtime::config::ExecPolicyConfig;
use rusqlite::Connection;
use turul_mcp_client::McpClient;

use super::app::{AppState, ApprovalState, ChatState, TuiApp};
use super::events::{self, EventResult};
use super::theme::Theme;
use super::widgets;

use async_trait::async_trait;
use harper_core::core::error::HarperResult;
use std::path::Path;

/// Type alias for the approval message sent via channels
type ApprovalMessage = (String, String, Arc<Mutex<Option<oneshot::Sender<bool>>>>);

/// Approval provider for TUI that uses channels to communicate with the UI loop
pub struct TuiApproval {
    approval_tx: mpsc::Sender<ApprovalMessage>,
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
    MessageProcessed {
        session_id: String,
        messages: Vec<harper_core::core::Message>,
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
    _mcp_client: Option<&McpClient>,
) -> Result<(), Box<dyn std::error::Error>> {
    // Set up terminal
    enable_raw_mode()?;
    let mut stdout = std::io::stdout();
    execute!(stdout, EnterAlternateScreen, cursor::Hide)?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    let mut app = TuiApp::new();

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
                        .with_approver(approver.clone());

                        // Load existing history
                        let mut history =
                            harper_core::memory::storage::load_history(&worker_conn, &session_id)
                                .unwrap_or_default();

                        match chat_service
                            .send_message(&user_msg, &mut history, web_search, &session_id)
                            .await
                        {
                            Ok(_) => {
                                let _ = ui_tx_clone
                                    .send(UiUpdate::MessageProcessed {
                                        session_id,
                                        messages: history,
                                    })
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
                                let session_id = chat_state.session_id.clone();
                                let web_search = chat_state.web_search_enabled;

                                // optimistic update
                                chat_state.messages.push(harper_core::core::Message {
                                    role: "user".to_string(),
                                    content: msg.clone(),
                                });

                                let _ = worker_tx.send(WorkerMsg::SendMessage {
                                    user_msg: msg,
                                    session_id,
                                    web_search,
                                }).await;
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
                        UiUpdate::MessageProcessed { session_id, messages } => {
                            if let AppState::Chat(chat_state) = &mut app.state {
                                if chat_state.session_id == session_id {
                                    chat_state.messages = messages;
                                    if chat_state.sidebar_visible {
                                        spawn_sidebar_gathering(chat_state, &ui_tx);
                                    }
                                }
                            }
                        }
                        UiUpdate::SidebarEntries { entries } => {
                            if let AppState::Chat(chat_state) = &mut app.state {
                                chat_state.sidebar_entries = entries;
                            }
                        }
                        UiUpdate::Error(err) => {
                            app.set_error_message(err);
                        }
                    }
                }
            }

            // Approval Requests
            approval = approval_rx.recv() => {
                if let Some((prompt, command, tx)) = approval {
                    // Use the new pending_approval overlay instead of replacing state
                    app.pending_approval = Some(ApprovalState {
                        prompt,
                        command,
                        tx,
                        scroll_offset: 0,
                    });
                }
            }
        }
    }

    // Restore terminal
    disable_raw_mode()?;
    execute!(terminal.backend_mut(), LeaveAlternateScreen, cursor::Show)?;

    Ok(())
}
