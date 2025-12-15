use crossterm::event::{Event, KeyCode, KeyModifiers};
use std::fs;
use std::path::Path;
use uuid::Uuid;

use super::app::{AppState, SessionInfo, TuiApp};
use crate::memory::session_service::SessionService;

pub enum EventResult {
    Continue,
    SendMessage(String),
    Quit,
}

fn load_sessions_into_state(app: &mut TuiApp, session_service: &SessionService) {
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
                if let AppState::Chat(_, _, _, _, web_search_enabled, ..) = &mut app.state {
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
                if matches!(app.state, AppState::Chat(..)) {
                    app.state = AppState::Menu(0);
                } else {
                    return EventResult::Quit;
                }
            }
            KeyCode::Esc => match &app.state {
                AppState::Menu(_) => return EventResult::Quit,
                AppState::Sessions(_, _) => app.state = AppState::Menu(0),
                AppState::Chat(..) => app.state = AppState::Menu(0),
                AppState::Tools(_) => app.state = AppState::Menu(0),
                AppState::ViewSession(_, _, _) => app.state = AppState::Menu(0),
            },
            KeyCode::Down | KeyCode::Char('j') => app.next(),
            KeyCode::Up | KeyCode::Char('k') => app.previous(),
            KeyCode::Enter => return handle_enter(app, session_service),
            KeyCode::Tab => handle_tab(app),
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
                0 => {
                    app.state = AppState::Chat(
                        Uuid::new_v4().to_string(),
                        vec![],
                        String::new(),
                        false,
                        false,
                        vec![],
                        0,
                    )
                } // Start Chat
                1 => load_sessions_into_state(app, session_service),
                2 => load_sessions_into_state(app, session_service),
                3 => app.state = AppState::Tools(0), // Tools
                4 => app.message = Some("Export not implemented yet".to_string()), // Export
                5 => return EventResult::Quit,       // Quit
                _ => {}
            }
        }
        AppState::Chat(_, _messages, input, ..) => {
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
                        app.state = AppState::Chat(
                            session.id.clone(),
                            messages,
                            String::new(),
                            false,
                            false,
                            vec![],
                            0,
                        );
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
    if let AppState::Chat(_, _, input, _, _, candidates, index) = &mut app.state {
        input.push(c);
        candidates.clear();
        *index = 0;
    }
}

fn handle_backspace(app: &mut TuiApp) {
    if let AppState::Chat(_, _, input, _, _, candidates, index) = &mut app.state {
        input.pop();
        candidates.clear();
        *index = 0;
    }
}

fn handle_tab(app: &mut TuiApp) {
    if let AppState::Chat(_, _, input, _, _, candidates, index) = &mut app.state {
        if input.starts_with('@') {
            // File completion
            if candidates.is_empty() {
                let prefix = &input[1..];
                let path = Path::new(prefix);

                let dir_to_read = if prefix.is_empty() || prefix.ends_with('/') {
                    path
                } else {
                    path.parent().unwrap_or_else(|| Path::new("."))
                };

                let file_prefix = if !prefix.is_empty() && !prefix.ends_with('/') {
                    path.file_name().unwrap_or_default().to_str().unwrap_or("")
                } else {
                    ""
                };

                if let Ok(entries) = fs::read_dir(dir_to_read) {
                    for entry in entries.flatten() {
                        if let Some(name) = entry.file_name().to_str() {
                            if name.starts_with(file_prefix) {
                                let full_path = dir_to_read.join(name);
                                if let Some(full_path_str) = full_path.to_str() {
                                    // Normalize path separators for consistency
                                    candidates
                                        .push(format!("@{}", full_path_str.replace('\\', "/")));
                                }
                            }
                        }
                    }
                }
                candidates.sort();
                *index = 0;
            }
            if !candidates.is_empty() {
                *input = candidates[*index].clone();
                *index = (*index + 1) % candidates.len();
            }
        } else if input.starts_with('/') {
            // Command completion
            if candidates.is_empty() {
                let commands = vec!["/help", "/quit", "/clear", "/exit"];
                for cmd in commands {
                    if cmd.starts_with(&*input) {
                        candidates.push(cmd.to_string());
                    }
                }
                candidates.sort();
                *index = 0;
            }
            if !candidates.is_empty() {
                *input = candidates[*index].clone();
                *index = (*index + 1) % candidates.len();
            }
        } else {
            candidates.clear();
            *index = 0;
        }
    }
}
