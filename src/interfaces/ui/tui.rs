use std::time::Duration;

use crossterm::event;
use ratatui::{backend::CrosstermBackend, Terminal};
use rusqlite::Connection;

use super::app::{AppState, TuiApp};
use super::events::{handle_event, EventResult};
use super::widgets::draw;
use crate::agent::chat::ChatService;
use crate::core::ApiConfig;
use crate::memory::session_service::SessionService;

pub async fn run_tui(
    conn: &Connection,
    api_config: &ApiConfig,
    session_service: &SessionService<'_>,
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
    let mut chat_service = ChatService::new(conn, api_config, None, None);

    // Create app
    let mut app = TuiApp::new();

    loop {
        // Draw the UI
        terminal.draw(|frame| draw(frame, &app))?;

        // Handle events
        if event::poll(Duration::from_millis(100))? {
            let event = event::read()?;
            let result = handle_event(event, &mut app, session_service);
            match result {
                EventResult::Quit => break,
                EventResult::SendMessage(message) => {
                    if let AppState::Chat(session_id, messages, _, _, web_search_enabled) =
                        &mut app.state
                    {
                        let session_id = session_id.as_deref().unwrap();
                        // Send to AI
                        match chat_service
                            .send_message(&message, messages, *web_search_enabled, session_id)
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
