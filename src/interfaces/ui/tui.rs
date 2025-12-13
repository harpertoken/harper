use super::app::{AppState, MenuItem, TuiApp};
use super::draw::draw;
use crossterm::cursor::{Hide, Show};
use crossterm::event::{self, Event, KeyCode, KeyModifiers};
use crossterm::execute;
use crossterm::terminal::{
    disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen,
};
use rusqlite;
use std::io;
use std::time::Duration;

use crate::agent::chat::ChatService;
use crate::core::error::HarperError;
use crate::core::ApiConfig;
use crate::memory::session_service::SessionService;

pub async fn run_tui(
    conn: &rusqlite::Connection,
    api_config: &ApiConfig,
    prompt_id: Option<String>,
) -> Result<(), HarperError> {
    // Setup terminal
    enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen, Hide)?;

    // Create app
    let mut app = TuiApp::new();

    let session_service = SessionService::new(conn);
    let mut api_cache = crate::core::cache::new_api_cache();

    loop {
        draw(&mut stdout, &app)?;

        if event::poll(Duration::from_millis(100))? {
            if let Event::Key(key) = event::read()? {
                // Clear message on any key press
                if app.message.is_some() {
                    app.message = None;
                }
                match key.code {
                    KeyCode::Char('q') | KeyCode::Char('5') => {
                        if let AppState::Chat(_, _, _, _, _) = &app.state {
                            app.state = AppState::Menu(0);
                        } else {
                            app.should_quit = true;
                        }
                    }
                    KeyCode::Esc => {
                        match &app.state {
                            AppState::Menu(_) => app.should_quit = true,
                            AppState::ListSessions(_, _, _) => {
                                app.state = AppState::Menu(0);
                            }
                            AppState::PromptWebSearch => {
                                app.state = AppState::Menu(0);
                            }
                            AppState::Chat(_, _, _, _, _) => {
                                app.state = AppState::Menu(0);
                            }
                            AppState::ViewSession(_, _, _, _) => {
                                // Go back to list
                                match session_service.list_sessions_data() {
                                    Ok(sessions) => {
                                        app.state = AppState::ListSessions(sessions, 0, 0);
                                    }
                                    Err(e) => {
                                        app.message = Some(format!("Error: {}", e));
                                        app.state = AppState::Menu(0);
                                    }
                                }
                            }
                        }
                    }
                    KeyCode::Down | KeyCode::Char('j') => app.next(),
                    KeyCode::Up | KeyCode::Char('k') => app.previous(),
                    KeyCode::Enter => {
                        match &app.state {
                            AppState::Menu(_) => {
                                if let Some(selected) = app.select() {
                                    match selected {
                                        MenuItem::StartChat => {
                                            app.state = AppState::PromptWebSearch;
                                        }
                                        MenuItem::ListSessions => {
                                            match session_service.list_sessions_data() {
                                                Ok(sessions) => {
                                                    app.state =
                                                        AppState::ListSessions(sessions, 0, 0);
                                                }
                                                Err(e) => {
                                                    app.message = Some(format!(
                                                        "Error listing sessions: {}",
                                                        e
                                                    ));
                                                }
                                            }
                                        }
                                        MenuItem::ViewSession => {
                                            match session_service.list_sessions_data() {
                                                Ok(sessions) => {
                                                    app.state =
                                                        AppState::ListSessions(sessions, 0, 0);
                                                }
                                                Err(e) => {
                                                    app.message = Some(format!(
                                                        "Error listing sessions: {}",
                                                        e
                                                    ));
                                                }
                                            }
                                        }
                                        MenuItem::ExportSession => {
                                            if let Err(e) = session_service.export_session() {
                                                app.message =
                                                    Some(format!("Error exporting session: {}", e));
                                            } else {
                                                app.message = Some(
                                                    "Session exported successfully".to_string(),
                                                );
                                            }
                                        }
                                        MenuItem::Quit => app.should_quit = true,
                                    }
                                }
                            }
                            AppState::ListSessions(sessions, sel, _) => {
                                if let Some(session) = sessions.get(*sel) {
                                    match session_service.view_session_data(&session.id) {
                                        Ok(history) => {
                                            app.state = AppState::ViewSession(
                                                session.id.clone(),
                                                history,
                                                0,
                                                0,
                                            );
                                        }
                                        Err(e) => {
                                            app.message =
                                                Some(format!("Error viewing session: {}", e));
                                        }
                                    }
                                }
                            }
                            AppState::PromptWebSearch => {}
                            AppState::Chat(_, _, _, _, _) => {}
                            AppState::ViewSession(_, _, _, _) => {
                                // Maybe scroll or something, but for now, do nothing
                            }
                        }
                    }

                    _ => {}
                }
                if let AppState::PromptWebSearch = &app.state {
                    match key.code {
                        KeyCode::Char('y') | KeyCode::Char('Y') => {
                            let chat_service = ChatService::new(
                                conn,
                                api_config,
                                Some(&mut api_cache),
                                prompt_id.clone(),
                            );
                            match chat_service.create_session(true) {
                                Ok((history, session_id)) => {
                                    app.state =
                                        AppState::Chat(history, 0, String::new(), true, session_id);
                                }
                                Err(e) => {
                                    app.message =
                                        Some(format!("Error creating chat session: {}", e));
                                    app.state = AppState::Menu(0);
                                }
                            }
                        }
                        KeyCode::Char('n') | KeyCode::Char('N') => {
                            let chat_service = ChatService::new(
                                conn,
                                api_config,
                                Some(&mut api_cache),
                                prompt_id.clone(),
                            );
                            match chat_service.create_session(false) {
                                Ok((history, session_id)) => {
                                    app.state = AppState::Chat(
                                        history,
                                        0,
                                        String::new(),
                                        false,
                                        session_id,
                                    );
                                }
                                Err(e) => {
                                    app.message =
                                        Some(format!("Error creating chat session: {}", e));
                                    app.state = AppState::Menu(0);
                                }
                            }
                        }
                        _ => {}
                    }
                }
                if let AppState::Chat(history, _, input, web_search, session_id) = &mut app.state {
                    match key.code {
                        KeyCode::Char(c) => {
                            if key.modifiers.contains(KeyModifiers::CONTROL) && c == 'j' {
                                input.push('\n');
                            } else {
                                input.push(c);
                            }
                            app.history_index = app.history.len(); // Reset history index on typing
                        }
                        KeyCode::Backspace => {
                            input.pop();
                            app.history_index = app.history.len();
                        }
                        KeyCode::Up => {
                            if app.history_index > 0 {
                                app.history_index -= 1;
                                *input = app.history[app.history_index].clone();
                            }
                        }
                        KeyCode::Down => {
                            if app.history_index < app.history.len() {
                                app.history_index += 1;
                                if app.history_index == app.history.len() {
                                    *input = String::new();
                                } else {
                                    *input = app.history[app.history_index].clone();
                                }
                            }
                        }
                        KeyCode::Tab => {
                            if input.ends_with('@') {
                                // Simple file completion
                                if let Ok(entries) = std::fs::read_dir(".") {
                                    let files: Vec<String> = entries
                                        .filter_map(|e| e.ok())
                                        .filter_map(|e| e.file_name().into_string().ok())
                                        .filter(|name| !name.starts_with('.'))
                                        .collect();
                                    if !files.is_empty() {
                                        input.push_str(&files[0]);
                                    }
                                }
                            }
                        }
                        KeyCode::Enter => {
                            if !input.is_empty() {
                                let input_clone = input.clone();
                                app.history.push(input_clone.clone());
                                app.history_index = app.history.len();
                                *input = String::new();
                                let mut chat_service = ChatService::new(
                                    conn,
                                    api_config,
                                    Some(&mut api_cache),
                                    prompt_id.clone(),
                                );
                                if let Err(e) = chat_service
                                    .send_message(&input_clone, history, *web_search, session_id)
                                    .await
                                {
                                    app.message = Some(format!("Error: {}", e));
                                }
                            }
                        }
                        _ => {}
                    }
                }
            }
        }

        if app.should_quit {
            break;
        }
    }

    // Restore terminal
    disable_raw_mode()?;
    execute!(stdout, LeaveAlternateScreen, Show)?;

    Ok(())
}
