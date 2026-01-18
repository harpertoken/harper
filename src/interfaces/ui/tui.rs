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

use std::time::Duration;

use crossterm::event;
use ratatui::{backend::CrosstermBackend, Terminal};
use rusqlite::Connection;
use turul_mcp_client::McpClient;

use super::app::{AppState, TuiApp};
use super::events::{handle_event, EventResult};
use super::theme::Theme;
use super::widgets::draw;
use crate::agent::chat::ChatService;
use crate::core::ApiConfig;
use crate::memory::session_service::SessionService;
use crate::runtime::config::ExecPolicyConfig;
use std::collections::HashMap;

pub async fn run_tui(
    conn: &Connection,
    api_config: &ApiConfig,
    session_service: &SessionService<'_>,
    theme: &Theme,
    custom_commands: HashMap<String, String>,
    exec_policy: &ExecPolicyConfig,
    mcp_client: Option<&McpClient>,
) -> Result<(), Box<dyn std::error::Error>> {
    // Setup terminal
    crossterm::terminal::enable_raw_mode()?;
    let mut stdout = std::io::stdout();
    crossterm::execute!(
        stdout,
        crossterm::terminal::EnterAlternateScreen,
        crossterm::cursor::Hide
    )?;

    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    // Create chat service
    let mut chat_service = ChatService::new(
        conn,
        api_config,
        mcp_client,
        None,
        None,
        custom_commands,
        exec_policy.clone(),
    );

    // Create app
    let mut app = TuiApp::new();

    loop {
        // Draw the UI
        terminal.draw(|frame| draw(frame, &app, theme))?;

        // Handle events
        if event::poll(Duration::from_millis(100))? {
            let event = event::read()?;
            let result = handle_event(event, &mut app, session_service);
            match result {
                EventResult::Quit => break,
                EventResult::SendMessage(message) => {
                    if let AppState::Chat(chat_state) = &mut app.state {
                        let session_id = &chat_state.session_id;
                        // Send to AI
                        match chat_service
                            .send_message(
                                &message,
                                &mut chat_state.messages,
                                chat_state.web_search_enabled,
                                session_id,
                            )
                            .await
                        {
                            Ok(_) => {
                                // Response is already added to messages by send_message
                                app.message = Some("AI responded".to_string());
                            }
                            Err(e) => {
                                app.message = Some(format!("Error: {}", e));
                            }
                        }
                    }
                }
                crate::interfaces::ui::events::EventResult::Continue => {}
            }
        }
    }

    // Restore terminal
    crossterm::terminal::disable_raw_mode()?;
    crossterm::execute!(
        terminal.backend_mut(),
        crossterm::terminal::LeaveAlternateScreen,
        crossterm::cursor::Show
    )?;

    Ok(())
}
