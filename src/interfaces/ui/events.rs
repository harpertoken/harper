use crossterm::event::{Event, KeyCode, KeyModifiers};

use super::app::{AppState, SessionInfo, TuiApp};
use crate::memory::session_service::SessionService;

pub enum EventResult {
    Continue,
    SendMessage(String),
    Quit,
}

pub fn handle_event(
    event: Event,
    app: &mut TuiApp,
    session_service: &SessionService,
) -> EventResult {
    if let Event::Key(key) = event {
        // Clear message on any key press
        if app.message.is_some() {
            app.message = None;
        }

        match key.code {
            KeyCode::Char('c') if key.modifiers.contains(KeyModifiers::CONTROL) => {
                return EventResult::Quit;
            }
            KeyCode::Char('w') if key.modifiers.contains(KeyModifiers::CONTROL) => {
                if let AppState::Chat(_, _, _, _, web_search_enabled) = &mut app.state {
                    *web_search_enabled = !*web_search_enabled;
                    app.message = Some(format!(
                        "Web search {}",
                        if *web_search_enabled {
                            "enabled"
                        } else {
                            "disabled"
                        }
                    ));
                }
            }
            KeyCode::Char('q') => {
                if matches!(app.state, AppState::Chat(_, _, _, _, _)) {
                    app.state = AppState::Menu(0);
                } else {
                    return EventResult::Quit;
                }
            }
            KeyCode::Esc => match &app.state {
                AppState::Menu(_) => return EventResult::Quit,
                AppState::Sessions(_, _) => app.state = AppState::Menu(0),
                AppState::Chat(_, _, _, _, _) => app.state = AppState::Menu(0),
                AppState::Tools(_) => app.state = AppState::Menu(0),
                AppState::ViewSession(_, _, _) => app.state = AppState::Menu(0),
            },
            KeyCode::Down | KeyCode::Char('j') => app.next(),
            KeyCode::Up | KeyCode::Char('k') => app.previous(),
            KeyCode::Enter => return handle_enter(app, session_service),
            KeyCode::Char(c) => handle_char_input(app, c),
            KeyCode::Backspace => handle_backspace(app),
            _ => {}
        }
    }
    EventResult::Continue
}

fn handle_enter(app: &mut TuiApp, session_service: &SessionService) -> EventResult {
    match &mut app.state {
        AppState::Menu(selected) => {
            match *selected {
                0 => app.state = AppState::Chat(None, vec![], String::new(), false, false), // Start Chat
                1 => {
                    // Load real sessions
                    match session_service.list_sessions_data() {
                        Ok(sessions) => {
                            let session_infos: Vec<SessionInfo> = sessions
                                .into_iter()
                                .map(|s| SessionInfo {
                                    id: s.id.clone(),
                                    name: s.id, // Use ID as name for now
                                    created_at: s.created_at,
                                })
                                .collect();
                            app.state = AppState::Sessions(session_infos, 0);
                        }
                        Err(e) => {
                            app.message = Some(format!("Error loading sessions: {}", e));
                        }
                    }
                }
                2 => {
                    app.state =
                        AppState::ViewSession("Select a session first".to_string(), vec![], 0)
                } // View Session
                3 => app.state = AppState::Tools(0), // Tools
                4 => app.message = Some("Export not implemented yet".to_string()), // Export
                5 => return EventResult::Quit,       // Quit
                _ => {}
            }
        }
        AppState::Chat(_, _messages, input, _, _) => {
            if !input.is_empty() {
                let message = input.clone();
                // Clear input
                *input = String::new();
                return EventResult::SendMessage(message);
            }
        }
        AppState::Sessions(sessions, selected) => {
            if !sessions.is_empty() && *selected < sessions.len() {
                let session = &sessions[*selected];
                match session_service.view_session_data(&session.id) {
                    Ok(messages) => {
                        app.state = AppState::ViewSession(session.name.clone(), messages, 0);
                    }
                    Err(e) => {
                        app.message = Some(format!("Error loading session: {}", e));
                    }
                }
            }
        }
        AppState::Tools(selected) => {
            match *selected {
                0 => {
                    app.message =
                        Some("File operations: Use AI chat to request file operations".to_string())
                }
                1 => {
                    app.message =
                        Some("Git commands: Use AI chat to request git operations".to_string())
                }
                2 => app.message = Some("Web search: Toggle with Ctrl+W in chat".to_string()),
                3 => {
                    app.message =
                        Some("Shell commands: Use AI chat to request shell operations".to_string())
                }
                4 => app.state = AppState::Menu(0), // Back to Menu
                _ => {}
            }
        }
        _ => {}
    }
    EventResult::Continue
}

fn handle_char_input(app: &mut TuiApp, c: char) {
    if let AppState::Chat(_, _, input, _, _) = &mut app.state {
        input.push(c);
    }
}

fn handle_backspace(app: &mut TuiApp) {
    if let AppState::Chat(_, _, input, _, _) = &mut app.state {
        input.pop();
    }
}
