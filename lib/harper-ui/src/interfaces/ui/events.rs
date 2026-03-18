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

use arboard::{Clipboard, ImageData};
use crossterm::event::{Event, KeyCode, KeyModifiers};
use harper_core;
use std::fs;
use std::path::Path;
use std::path::PathBuf;
use uuid::Uuid;

// Keyboard shortcut constants
const HELP_MESSAGE: &str =
    "Ctrl+H:Help | @+Tab:File Complete | Esc:Back | ↑↓:Navigate | Enter:Select | Ctrl+C:Quit | Ctrl+W:Toggle Web Search";

use super::app::{AppState, ChatState, SessionInfo, TuiApp};
use harper_core::memory::session_service::SessionService;

pub enum EventResult {
    Continue,
    SendMessage(String),
    GatherSidebarEntries,
    Quit,
}

fn create_chat_state(session_id: String, messages: Vec<harper_core::core::Message>) -> ChatState {
    ChatState {
        session_id,
        messages,
        input: String::new(),
        web_search: false,
        web_search_enabled: false,
        completion_candidates: vec![],
        completion_index: 0,
        scroll_offset: 0,
        completion_prefix: None,
        sidebar_visible: false,
        sidebar_entries: Vec::new(),
    }
}

fn record_approval_history(app: &mut TuiApp, command: &str, approved: bool) {
    let marker = if approved { "[Y]" } else { "[N]" };
    app.approval_history.push(format!("{} {}", marker, command));
    if app.approval_history.len() > 6 {
        let excess = app.approval_history.len() - 6;
        app.approval_history.drain(0..excess);
    }
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
            app.set_error_message(format!("Error loading sessions: {}", e));
        }
    }
}

fn load_export_sessions_into_state(app: &mut TuiApp, session_service: &SessionService) {
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
            app.state = AppState::ExportSessions(session_infos, 0);
        }
        Err(e) => {
            app.set_error_message(format!("Error loading sessions: {}", e));
        }
    }
}

pub fn handle_event(
    event: Event,
    app: &mut TuiApp,
    session_service: &SessionService,
) -> EventResult {
    match event {
        Event::Key(key) => {
            // Clear message on any key press
            if app.message.is_some() {
                app.clear_message();
            }

            // PRIORITIZE: Handle input for the security approval modal if active
            if let Some(approval) = &mut app.pending_approval {
                match key.code {
                    KeyCode::Char('y') | KeyCode::Char('Y') => {
                        let command = approval.command.clone();
                        if let Some(tx) = approval.tx.lock().unwrap().take() {
                            let _ = tx.send(true);
                        }
                        app.pending_approval = None;
                        record_approval_history(app, &command, true);
                        return EventResult::Continue;
                    }
                    KeyCode::Char('n') | KeyCode::Char('N') | KeyCode::Esc => {
                        let command = approval.command.clone();
                        if let Some(tx) = approval.tx.lock().unwrap().take() {
                            let _ = tx.send(false);
                        }
                        app.pending_approval = None;
                        record_approval_history(app, &command, false);
                        return EventResult::Continue;
                    }
                    KeyCode::Down | KeyCode::Char('j') => {
                        app.next(); // app.next() handles scroll offset for approval
                        return EventResult::Continue;
                    }
                    KeyCode::Up | KeyCode::Char('k') => {
                        app.previous(); // app.previous() handles scroll offset for approval
                        return EventResult::Continue;
                    }
                    _ => return EventResult::Continue, // Consume all other keys while modal is up
                }
            }

            // Handle Ctrl shortcuts first
            if key.modifiers.contains(KeyModifiers::CONTROL) {
                match key.code {
                    KeyCode::Char('g') => {
                        app.set_help_message(HELP_MESSAGE.to_string());
                        return EventResult::Continue;
                    }
                    KeyCode::Char('x') => {
                        return EventResult::Quit;
                    }
                    KeyCode::Char('o') => {
                        match &app.state {
                            AppState::Chat(chat_state) => {
                                let session_id = chat_state.session_id.clone();
                                match session_service.export_session_by_id(&session_id) {
                                    Ok(path) => app
                                        .set_info_message(format!("Session exported to {}", path)),
                                    Err(e) => {
                                        app.set_error_message(format!("Export failed: {}", e))
                                    }
                                }
                            }
                            _ => load_export_sessions_into_state(app, session_service),
                        }
                        return EventResult::Continue;
                    }
                    KeyCode::Char('r') => {
                        load_sessions_into_state(app, session_service);
                        return EventResult::Continue;
                    }
                    KeyCode::Char('w') => {
                        if let AppState::Chat(chat_state) = &mut app.state {
                            chat_state.web_search_enabled = !chat_state.web_search_enabled;
                            let enabled = chat_state.web_search_enabled;
                            app.set_status_message(format!(
                                "Web search {}",
                                if enabled { "enabled" } else { "disabled" }
                            ));
                        }
                        return EventResult::Continue;
                    }
                    KeyCode::Char('k') => {
                        if let AppState::Chat(chat_state) = &mut app.state {
                            if !chat_state.input.is_empty() {
                                app.cut_buffer = chat_state.input.clone();
                                chat_state.input.clear();
                                chat_state.reset_completion();
                                app.set_status_message("Text cut to buffer".to_string());
                            }
                        }
                        return EventResult::Continue;
                    }
                    KeyCode::Char('u') => {
                        if let AppState::Chat(chat_state) = &mut app.state {
                            if !app.cut_buffer.is_empty() {
                                chat_state.input.push_str(&app.cut_buffer);
                                chat_state.reset_completion();
                            }
                        }
                        return EventResult::Continue;
                    }
                    KeyCode::Char('t') => {
                        return handle_enter(app, session_service);
                    }
                    KeyCode::Char('c') => {
                        if let AppState::Chat(chat_state) = &app.state {
                            app.set_info_message(format!("Session ID: {}", chat_state.session_id));
                        } else {
                            app.set_info_message("State: Menu".to_string());
                        }
                        return EventResult::Continue;
                    }
                    KeyCode::Char('j') => {
                        handle_shell_commands(app);
                        return EventResult::Continue;
                    }
                    KeyCode::Char('b') => {
                        if let AppState::Chat(chat_state) = &mut app.state {
                            chat_state.sidebar_visible = !chat_state.sidebar_visible;
                            if chat_state.sidebar_visible {
                                app.set_status_message("Harvest navigator pinned".to_string());
                                return EventResult::GatherSidebarEntries;
                            } else {
                                app.set_status_message("Harvest navigator hidden".to_string());
                            }
                        }
                        return EventResult::Continue;
                    }
                    KeyCode::Char('y') => {
                        app.previous();
                        return EventResult::Continue;
                    }
                    KeyCode::Char('v') => {
                        app.next();
                        return EventResult::Continue;
                    }
                    _ => {}
                }
            }

            // Normal state handling
            match key.code {
                KeyCode::F(1) => {
                    // Show help overlay
                    app.set_help_message(HELP_MESSAGE.to_string());
                }
                KeyCode::Char('h') if key.modifiers.contains(KeyModifiers::CONTROL) => {
                    // Alternative help key for Mac users
                    app.set_help_message(HELP_MESSAGE.to_string());
                }
                KeyCode::Char('c')
                    if key.modifiers.contains(KeyModifiers::CONTROL)
                        && key.modifiers.contains(KeyModifiers::SHIFT) =>
                {
                    handle_copy(app);
                }
                KeyCode::Char('v')
                    if key.modifiers.contains(KeyModifiers::CONTROL)
                        && key.modifiers.contains(KeyModifiers::SHIFT) =>
                {
                    handle_image_paste(app);
                }
                KeyCode::Esc => match &app.state {
                    AppState::Menu(_) => {}
                    AppState::Chat(_) => app.state = AppState::Menu(0),
                    AppState::Sessions(_, _) => app.state = AppState::Menu(0),
                    AppState::ExportSessions(_, _) => app.state = AppState::Menu(0),
                    AppState::Tools(_) => app.state = AppState::Menu(0),
                    AppState::ViewSession(_, _, _) => app.state = AppState::Menu(0),
                },
                KeyCode::Down | KeyCode::Char('j') => {
                    if matches!(app.state, AppState::Chat(_)) {
                        if matches!(key.code, KeyCode::Down) {
                            app.next();
                        } else {
                            handle_char_input(app, 'j');
                        }
                    } else {
                        app.next();
                    }
                }
                KeyCode::Up | KeyCode::Char('k') => {
                    if matches!(app.state, AppState::Chat(_)) {
                        if matches!(key.code, KeyCode::Up) {
                            app.previous();
                        } else {
                            handle_char_input(app, 'k');
                        }
                    } else {
                        app.previous();
                    }
                }
                KeyCode::Enter => return handle_enter(app, session_service),
                KeyCode::Tab => handle_tab(app),
                KeyCode::Char(c) => {
                    // Handle q for quitting/returning only in non-input states
                    if c == 'q' && !matches!(app.state, AppState::Chat(_)) {
                        if matches!(app.state, AppState::Menu(_)) {
                            return EventResult::Quit;
                        } else {
                            app.state = AppState::Menu(0);
                            return EventResult::Continue;
                        }
                    }
                    handle_char_input(app, c)
                }
                KeyCode::Backspace => handle_backspace(app),
                _ => {}
            }
        }
        Event::Paste(content) => {
            handle_paste(app, content);
        }
        _ => {}
    }
    EventResult::Continue
}

fn handle_enter(app: &mut TuiApp, session_service: &SessionService) -> EventResult {
    match &mut app.state {
        AppState::Menu(selected) => {
            match *selected {
                0 => {
                    app.state =
                        AppState::Chat(create_chat_state(Uuid::new_v4().to_string(), vec![]));
                    return EventResult::GatherSidebarEntries;
                } // Start Chat
                1 => load_sessions_into_state(app, session_service),
                2 => load_export_sessions_into_state(app, session_service),
                3 => app.state = AppState::Tools(0), // Tools
                4 => return EventResult::Quit,       // Exit
                _ => {}
            }
        }
        AppState::Chat(chat_state) => {
            if !chat_state.input.is_empty() {
                let message = chat_state.input.clone();
                // Clear input
                chat_state.input = String::new();
                return EventResult::SendMessage(message);
            }
        }
        AppState::Sessions(sessions, selected) => {
            if !sessions.is_empty() && *selected < sessions.len() {
                let session = &sessions[*selected];
                match session_service.view_session_data(&session.id) {
                    Ok(messages) => {
                        app.state = AppState::Chat(create_chat_state(session.id.clone(), messages));
                        return EventResult::GatherSidebarEntries;
                    }
                    Err(e) => {
                        app.set_error_message(format!("Error loading session: {}", e));
                    }
                }
            }
        }
        AppState::ExportSessions(sessions, selected) => {
            if !sessions.is_empty() && *selected < sessions.len() {
                let session = &sessions[*selected];
                match session_service.export_session_by_id(&session.id) {
                    Ok(path) => app.set_info_message(format!("Session exported to {}", path)),
                    Err(e) => app.set_error_message(format!("Export failed: {}", e)),
                }
                app.state = AppState::Menu(0);
            }
        }
        AppState::Tools(selected) => match *selected {
            0 => handle_web_search(app),
            1 => handle_system_info(app),
            2 => handle_process_list(app),
            3 => handle_git_commands(app),
            _ => {}
        },
        _ => {}
    }
    EventResult::Continue
}

fn handle_git_commands(app: &mut TuiApp) {
    match harper_core::tools::git::git_status() {
        Ok(status) => app.set_info_message(format!("Git Status:\n{}", status)),
        Err(e) => app.set_error_message(format!("Git error: {}", e)),
    }
}

fn handle_web_search(app: &mut TuiApp) {
    app.set_info_message(
        "Web Search: Press Ctrl+W in chat mode to toggle web search.\nOr use AI chat with search queries."
            .to_string(),
    );
}

fn handle_shell_commands(app: &mut TuiApp) {
    let command_output = {
        #[cfg(unix)]
        {
            std::process::Command::new("ls").arg("-la").output()
        }
        #[cfg(windows)]
        {
            std::process::Command::new("cmd")
                .args(["/C", "dir"])
                .output()
        }
        #[cfg(not(any(unix, windows)))]
        {
            Err(std::io::Error::new(
                std::io::ErrorKind::Unsupported,
                "Shell command tool not supported on this OS.",
            ))
        }
    };

    match command_output {
        Ok(output) => {
            if output.status.success() {
                let result = String::from_utf8_lossy(&output.stdout);
                app.set_info_message(format!("Directory listing:\n{}", result));
            } else {
                let error = String::from_utf8_lossy(&output.stderr);
                app.set_error_message(format!("Shell error:\n{}", error));
            }
        }
        Err(e) => app.set_error_message(format!("Shell error: {}", e)),
    }
}

fn handle_system_info(app: &mut TuiApp) {
    let info = {
        #[cfg(unix)]
        {
            std::process::Command::new("uname").arg("-a").output()
        }
        #[cfg(windows)]
        {
            std::process::Command::new("systeminfo").output()
        }
        #[cfg(not(any(unix, windows)))]
        {
            Err(std::io::Error::new(
                std::io::ErrorKind::Unsupported,
                "System info tool not supported on this OS.",
            ))
        }
    };

    match info {
        Ok(output) => {
            if output.status.success() {
                app.set_info_message(format!(
                    "System Information:\n{}",
                    String::from_utf8_lossy(&output.stdout)
                ));
            } else {
                app.set_error_message("Failed to retrieve system information".to_string());
            }
        }
        Err(e) => app.set_error_message(format!("System info error: {}", e)),
    }
}

fn handle_process_list(app: &mut TuiApp) {
    let list = {
        #[cfg(unix)]
        {
            std::process::Command::new("ps").arg("aux").output()
        }
        #[cfg(windows)]
        {
            std::process::Command::new("tasklist").output()
        }
        #[cfg(not(any(unix, windows)))]
        {
            Err(std::io::Error::new(
                std::io::ErrorKind::Unsupported,
                "Process list tool not supported on this OS.",
            ))
        }
    };

    match list {
        Ok(output) => {
            if output.status.success() {
                app.set_info_message(format!(
                    "Process List (partial):\n{}",
                    String::from_utf8_lossy(&output.stdout)
                        .lines()
                        .take(20)
                        .collect::<Vec<_>>()
                        .join("\n")
                ));
            } else {
                app.set_error_message("Failed to retrieve process list".to_string());
            }
        }
        Err(e) => app.set_error_message(format!("Process list error: {}", e)),
    }
}

fn handle_char_input(app: &mut TuiApp, c: char) {
    if let AppState::Chat(chat_state) = &mut app.state {
        chat_state.input.push(c);
        chat_state.reset_completion();
    }
}

fn handle_backspace(app: &mut TuiApp) {
    if let AppState::Chat(chat_state) = &mut app.state {
        chat_state.input.pop();
        chat_state.reset_completion();
    }
}
fn handle_paste(app: &mut TuiApp, content: String) {
    if let AppState::Chat(chat_state) = &mut app.state {
        chat_state.input.push_str(&content);
        chat_state.reset_completion();
    }
}

fn handle_image_paste(app: &mut TuiApp) {
    if let AppState::Chat(chat_state) = &mut app.state {
        let mut clipboard = match Clipboard::new() {
            Ok(clipboard) => clipboard,
            Err(e) => {
                app.set_error_message(format!("Clipboard not available: {}", e));
                return;
            }
        };

        let image_data = match clipboard.get_image() {
            Ok(image) => image,
            Err(_) => {
                app.set_error_message("No image found in clipboard".to_string());
                return;
            }
        };

        match save_image_to_temp(&image_data) {
            Ok(file_path) => {
                let reference = format!("@{}", file_path.display());
                chat_state.input.push_str(&reference);
                chat_state.reset_completion();
                app.set_info_message(format!(
                    "Image pasted and saved as: {}",
                    file_path.display()
                ));
            }
            Err(e) => {
                app.set_error_message(format!("Failed to save pasted image: {}", e));
            }
        }
    }
}

pub(crate) fn save_image_to_temp(
    image_data: &ImageData,
) -> harper_core::core::error::HarperResult<PathBuf> {
    // Create temp directory if it doesn't exist
    let temp_dir = std::env::temp_dir().join("harper_images");
    fs::create_dir_all(&temp_dir)?;

    // Generate unique filename
    let timestamp = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map_err(|e| {
            harper_core::core::error::HarperError::Api(format!("System time error: {}", e))
        })?
        .as_millis();
    let filename = format!("pasted_image_{}.png", timestamp);
    let file_path = temp_dir.join(filename);

    // Create image buffer from raw bytes and save as PNG
    let width = image_data.width as u32;
    let height = image_data.height as u32;

    // Create image buffer from raw bytes
    let img =
        image::RgbaImage::from_raw(width, height, image_data.bytes.to_vec()).ok_or_else(|| {
            harper_core::core::error::HarperError::File(
                "Failed to create image from clipboard data".to_string(),
            )
        })?;

    // Save as PNG
    img.save(&file_path).map_err(|e| {
        harper_core::core::error::HarperError::File(format!("Failed to save image: {}", e))
    })?;

    Ok(file_path)
}

fn handle_copy(app: &mut TuiApp) {
    if let AppState::Chat(chat_state) = &mut app.state {
        if !chat_state.input.is_empty() {
            match Clipboard::new() {
                Ok(mut clipboard) => {
                    if clipboard.set_text(&chat_state.input).is_ok() {
                        app.set_info_message("Copied to clipboard".to_string());
                    } else {
                        app.set_error_message("Failed to copy to clipboard".to_string());
                    }
                }
                Err(e) => {
                    app.set_error_message(format!("Clipboard not available: {}", e));
                }
            }
        }
    }
}

fn handle_tab(app: &mut TuiApp) {
    if let AppState::Chat(chat_state) = &mut app.state {
        if chat_state.input.starts_with('@') {
            let prefix = &chat_state.input[1..];
            let path = Path::new(prefix);

            // Determine directory to read and file prefix to match
            let (dir_to_read, file_prefix) = if prefix.is_empty() {
                // Just "@" - show all files/directories in current dir
                (Path::new("."), "")
            } else if prefix.ends_with('/') {
                // "@somedir/" - show contents of somedir
                (path, "")
            } else if prefix == "." {
                // "@." - show hidden files/directories in current dir
                (Path::new("."), ".")
            } else if prefix.starts_with('.') && !prefix.contains('/') {
                // "@.something" - show hidden files starting with ".something"
                (Path::new("."), prefix)
            } else {
                // "@somedir/file" - show files in somedir starting with "file"
                let parent = path.parent().unwrap_or_else(|| Path::new("."));
                let file_part = path.file_name().and_then(|n| n.to_str()).unwrap_or("");
                (parent, file_part)
            };

            // Check if we need to refresh candidates
            // Only refresh if this is truly a new completion (not cycling existing results)
            let is_showing_candidate = !chat_state.completion_candidates.is_empty()
                && chat_state.completion_candidates.contains(&chat_state.input);
            let needs_refresh = chat_state.completion_candidates.is_empty()
                || (!is_showing_candidate
                    && chat_state.completion_prefix.as_deref() != Some(prefix));

            if needs_refresh {
                chat_state.completion_prefix = Some(prefix.to_string());
                chat_state.completion_candidates.clear();
                chat_state.completion_index = 0;

                // Read directory and collect matching entries
                if let Ok(entries) = fs::read_dir(dir_to_read) {
                    for entry in entries.flatten() {
                        if let Some(name) = entry.file_name().to_str() {
                            // Skip hidden files/directories unless explicitly requested
                            let show_hidden = file_prefix.starts_with('.');
                            if !show_hidden && name.starts_with('.') {
                                continue;
                            }

                            if name.starts_with(file_prefix) {
                                // Build the full path relative to where we started
                                let full_path = dir_to_read.join(name);

                                if let Some(full_path_str) = full_path.to_str() {
                                    // Normalize path separators and add @ prefix
                                    let normalized_path: String = full_path_str.replace('\\', "/");
                                    chat_state
                                        .completion_candidates
                                        .push(format!("@{}", normalized_path));
                                }
                            }
                        }
                    }
                }

                chat_state.completion_candidates.sort();
            }

            // Cycle through candidates
            if !chat_state.completion_candidates.is_empty() {
                chat_state.input =
                    chat_state.completion_candidates[chat_state.completion_index].clone();
                chat_state.completion_index =
                    (chat_state.completion_index + 1) % chat_state.completion_candidates.len();
            }
        } else if chat_state.input.starts_with('/') {
            // Command completion
            if chat_state.completion_candidates.is_empty() {
                let commands = vec!["/help", "/quit", "/clear", "/exit", "/audit"];
                for cmd in commands {
                    if cmd.starts_with(&chat_state.input) {
                        chat_state.completion_candidates.push(cmd.to_string());
                    }
                }
                chat_state.completion_candidates.sort();
                chat_state.completion_index = 0;
            }
            if !chat_state.completion_candidates.is_empty() {
                chat_state.input =
                    chat_state.completion_candidates[chat_state.completion_index].clone();
                chat_state.completion_index =
                    (chat_state.completion_index + 1) % chat_state.completion_candidates.len();
            }
        } else {
            chat_state.completion_candidates.clear();
            chat_state.completion_index = 0;
        }
    }
}

#[cfg(test)]
#[allow(dead_code)]
mod tests {
    use super::*;
    use crate::interfaces::ui::app::{AppState, TuiApp};
    use harper_core::memory::session_service::SessionService;

    #[test]
    fn test_menu_navigation() {
        let mut app = TuiApp::new();
        assert!(matches!(app.state, AppState::Menu(0)));

        app.next();
        assert!(matches!(app.state, AppState::Menu(1)));

        app.next();
        assert!(matches!(app.state, AppState::Menu(2)));

        app.previous();
        assert!(matches!(app.state, AppState::Menu(1)));
    }

    #[test]
    fn test_enter_menu_start_chat() {
        let mut app = TuiApp::new();
        let conn = rusqlite::Connection::open_in_memory().unwrap();
        harper_core::memory::storage::init_db(&conn).unwrap();
        let session_service = SessionService::new(&conn);

        // Menu at 0 (Start Chat)
        let result = handle_enter(&mut app, &session_service);
        assert!(matches!(result, EventResult::Continue));
        assert!(matches!(app.state, AppState::Chat(_)));
    }

    #[test]
    fn test_enter_menu_load_sessions() {
        let mut app = TuiApp::new();
        app.state = AppState::Menu(1); // Load Sessions
        let conn = rusqlite::Connection::open_in_memory().unwrap();
        harper_core::memory::storage::init_db(&conn).unwrap();
        let session_service = SessionService::new(&conn);

        let result = handle_event(
            Event::Key(KeyCode::Enter.into()),
            &mut app,
            &session_service,
        );
        assert!(matches!(result, EventResult::Continue));
        assert!(matches!(app.state, AppState::Sessions(_, 0)));
    }

    #[test]
    fn test_enter_menu_export_sessions() {
        let mut app = TuiApp::new();
        app.state = AppState::Menu(2); // Export Sessions
        let conn = rusqlite::Connection::open_in_memory().unwrap();
        harper_core::memory::storage::init_db(&conn).unwrap();
        let session_service = SessionService::new(&conn);

        let result = handle_event(
            Event::Key(KeyCode::Enter.into()),
            &mut app,
            &session_service,
        );
        assert!(matches!(result, EventResult::Continue));
        assert!(matches!(app.state, AppState::ExportSessions(_, 0)));
    }
}
