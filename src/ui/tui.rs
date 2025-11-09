use crossterm::cursor::{Hide, MoveTo, Show};
use crossterm::event::{self, Event, KeyCode};
use crossterm::execute;
use crossterm::style::{Color, Print, ResetColor, SetForegroundColor};
use crossterm::terminal::{
    disable_raw_mode, enable_raw_mode, size, Clear, ClearType, EnterAlternateScreen,
    LeaveAlternateScreen,
};
use rusqlite;
use std::io::{self, Write};
use std::time::Duration;

use crate::core::chat_service::ChatService;
use crate::core::error::HarperError;
use crate::core::session_service::{Session, SessionService};
use crate::core::ApiConfig;
use crate::core::Message;

#[derive(Clone)]
pub enum MenuItem {
    StartChat,
    ListSessions,
    ViewSession,
    ExportSession,
    Quit,
}

#[derive(Clone)]
pub enum AppState {
    Menu(usize),
    PromptWebSearch,
    Chat(Vec<Message>, usize, String, bool, String),
    ListSessions(Vec<Session>, usize, usize),
    ViewSession(String, Vec<Message>, usize, usize),
}

impl MenuItem {
    pub fn all() -> Vec<Self> {
        vec![
            Self::StartChat,
            Self::ListSessions,
            Self::ViewSession,
            Self::ExportSession,
            Self::Quit,
        ]
    }

    pub fn text(&self) -> &'static str {
        match self {
            Self::StartChat => "Start new chat session",
            Self::ListSessions => "List previous sessions",
            Self::ViewSession => "View a session's history",
            Self::ExportSession => "Export a session's history",
            Self::Quit => "Quit",
        }
    }
}

pub struct TuiApp {
    pub state: AppState,
    pub menu_items: Vec<MenuItem>,
    pub should_quit: bool,
    pub message: Option<String>,
}

impl Default for TuiApp {
    fn default() -> Self {
        Self {
            state: AppState::Menu(0),
            menu_items: MenuItem::all(),
            should_quit: false,
            message: None,
        }
    }
}

impl TuiApp {
    pub fn new() -> Self {
        Self {
            state: AppState::Menu(0),
            menu_items: MenuItem::all(),
            should_quit: false,
            message: None,
        }
    }

    fn get_current_len(&self) -> usize {
        match &self.state {
            AppState::Menu(_) => self.menu_items.len(),
            AppState::PromptWebSearch => 0,
            AppState::Chat(history, _, _, _, _) => history.len(),
            AppState::ListSessions(sessions, _, _) => sessions.len(),
            AppState::ViewSession(_, history, _, _) => history.len(),
        }
    }

    fn get_current_selected(&self) -> usize {
        match &self.state {
            AppState::Menu(sel) => *sel,
            AppState::PromptWebSearch => 0,
            AppState::Chat(_, _, _, _, _) => 0,
            AppState::ListSessions(_, sel, _) => *sel,
            AppState::ViewSession(_, _, sel, _) => *sel,
        }
    }

    fn set_current_selected(&mut self, sel: usize) {
        match &mut self.state {
            AppState::Menu(s) => *s = sel,
            AppState::PromptWebSearch => {}
            AppState::Chat(_, _, _, _, _) => {}
            AppState::ListSessions(_, s, _) => *s = sel,
            AppState::ViewSession(_, _, s, _) => *s = sel,
        }
    }

    pub fn next(&mut self) {
        match &mut self.state {
            AppState::Menu(_)
            | AppState::ListSessions(_, _, _)
            | AppState::ViewSession(_, _, _, _) => {
                let len = self.get_current_len();
                if len > 0 {
                    let current = self.get_current_selected();
                    self.set_current_selected((current + 1) % len);
                    self.adjust_offset();
                }
            }
            AppState::Chat(history, offset, _, _, _) => {
                let visible = {
                    let (_, height) = crossterm::terminal::size().unwrap_or((80, 24));
                    let start_y = 3;
                    let bottom_y = height.saturating_sub(1);
                    (bottom_y - start_y) as usize
                };
                *offset = (*offset + 1).min(history.len().saturating_sub(visible));
            }
            _ => {}
        }
    }

    pub fn previous(&mut self) {
        match &mut self.state {
            AppState::Menu(_)
            | AppState::ListSessions(_, _, _)
            | AppState::ViewSession(_, _, _, _) => {
                let len = self.get_current_len();
                if len > 0 {
                    let current = self.get_current_selected();
                    self.set_current_selected(if current > 0 { current - 1 } else { len - 1 });
                    self.adjust_offset();
                }
            }
            AppState::Chat(_, offset, _, _, _) => {
                *offset = offset.saturating_sub(1);
            }
            _ => {}
        }
    }

    pub fn select(&self) -> Option<MenuItem> {
        match &self.state {
            AppState::Menu(sel) => self.menu_items.get(*sel).cloned(),
            _ => None,
        }
    }

    fn get_offset(&self) -> usize {
        match &self.state {
            AppState::Menu(_) => 0,
            AppState::PromptWebSearch => 0,
            AppState::Chat(_, o, _, _, _) => *o,
            AppState::ListSessions(_, _, o) => *o,
            AppState::ViewSession(_, _, _, o) => *o,
        }
    }

    fn set_offset(&mut self, o: usize) {
        match &mut self.state {
            AppState::Menu(_) => {}
            AppState::PromptWebSearch => {}
            AppState::Chat(_, off, _, _, _) => *off = o,
            AppState::ListSessions(_, _, off) => *off = o,
            AppState::ViewSession(_, _, _, off) => *off = o,
        }
    }

    fn get_visible_items(&self) -> usize {
        let (_, height) = crossterm::terminal::size().unwrap_or((80, 24));
        let start_y = 3;
        let bottom_y = height.saturating_sub(1);
        (bottom_y - start_y) as usize
    }

    fn adjust_offset(&mut self) {
        let visible_items = self.get_visible_items();
        let len = self.get_current_len();
        if len == 0 {
            return;
        }
        let sel = self.get_current_selected();
        let offset = self.get_offset();
        if sel < offset {
            self.set_offset(sel);
        } else if sel >= offset + visible_items {
            self.set_offset(sel.saturating_sub(visible_items).saturating_add(1));
        }
    }
}

pub async fn run_tui(
    conn: &rusqlite::Connection,
    api_config: &ApiConfig,
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
                            let chat_service =
                                ChatService::new(conn, api_config, Some(&mut api_cache));
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
                            let chat_service =
                                ChatService::new(conn, api_config, Some(&mut api_cache));
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
                        KeyCode::Char(c) => input.push(c),
                        KeyCode::Backspace => {
                            input.pop();
                        }
                        KeyCode::Enter => {
                            if !input.is_empty() {
                                let input_clone = input.clone();
                                *input = String::new();
                                let mut chat_service =
                                    ChatService::new(conn, api_config, Some(&mut api_cache));
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

fn draw(stdout: &mut impl Write, app: &TuiApp) -> Result<(), HarperError> {
    execute!(stdout, Clear(ClearType::All), MoveTo(0, 0))?;

    // Title
    execute!(
        stdout,
        SetForegroundColor(Color::Cyan),
        Print("Harper AI Agent"),
        ResetColor,
        MoveTo(0, 1)
    )?;

    let start_y = 3;
    let (_width, height) = size().unwrap_or((80, 24));
    let bottom_y = height.saturating_sub(1);
    let visible_items = (bottom_y - start_y) as usize;

    match &app.state {
        AppState::Menu(sel) => {
            execute!(stdout, MoveTo(0, 2), Print("Menu:"), MoveTo(0, start_y))?;
            for (i, item) in app.menu_items.iter().enumerate() {
                if i == *sel {
                    execute!(
                        stdout,
                        SetForegroundColor(Color::Yellow),
                        Print(format!(">> {}", item.text())),
                        ResetColor
                    )?;
                } else {
                    execute!(stdout, Print(format!("   {}", item.text())))?;
                }
                execute!(stdout, MoveTo(0, start_y + i as u16 + 1))?;
            }
        }
        AppState::ListSessions(sessions, sel, offset) => {
            execute!(stdout, MoveTo(0, 2), Print("Sessions:"), MoveTo(0, start_y))?;
            for (i, session) in sessions
                .iter()
                .enumerate()
                .skip(*offset)
                .take(visible_items)
            {
                let display_i = (i - *offset) as u16;
                let text = format!("{} ({})", session.id, session.created_at);
                if i == *sel {
                    execute!(
                        stdout,
                        SetForegroundColor(Color::Yellow),
                        Print(format!(">> {}", text)),
                        ResetColor
                    )?;
                } else {
                    execute!(stdout, Print(format!("   {}", text)))?;
                }
                execute!(stdout, MoveTo(0, start_y + display_i + 1))?;
            }
        }
        AppState::ViewSession(session_id, history, sel, offset) => {
            execute!(
                stdout,
                MoveTo(0, 2),
                Print(format!("Session: {}", session_id)),
                MoveTo(0, start_y)
            )?;
            for (i, msg) in history.iter().enumerate().skip(*offset).take(visible_items) {
                let display_i = (i - *offset) as u16;
                let color = match msg.role.as_str() {
                    "user" => Color::Blue,
                    "assistant" => Color::Green,
                    "system" => Color::Magenta,
                    _ => Color::White,
                };
                let text = format!("{}: {}", msg.role, msg.content);
                if i == *sel {
                    execute!(
                        stdout,
                        SetForegroundColor(Color::Yellow),
                        Print(format!(">> {}", text)),
                        ResetColor
                    )?;
                } else {
                    execute!(
                        stdout,
                        SetForegroundColor(color),
                        Print(format!("   {}", text)),
                        ResetColor
                    )?;
                }
                execute!(stdout, MoveTo(0, start_y + display_i + 1))?;
            }
        }
        AppState::PromptWebSearch => {
            execute!(
                stdout,
                MoveTo(0, 2),
                Print("Enable web search for this session? (y/n)")
            )?;
        }
        AppState::Chat(history, offset, input, _, _) => {
            execute!(stdout, MoveTo(0, 2), Print("Chat:"), MoveTo(0, start_y))?;
            let mut display_i: u16 = 0;
            for msg in history.iter().skip(*offset) {
                if msg.role == "system" {
                    continue;
                }
                if display_i as usize >= visible_items {
                    break;
                }
                let color = match msg.role.as_str() {
                    "user" => Color::Blue,
                    "assistant" => Color::Green,
                    _ => Color::White,
                };
                let text = format!("{}: {}", msg.role, msg.content);
                execute!(stdout, SetForegroundColor(color), Print(text), ResetColor)?;
                execute!(stdout, MoveTo(0, start_y + display_i + 1))?;
                display_i += 1;
            }
            let input_y = bottom_y - 1;
            execute!(stdout, MoveTo(0, input_y), Print(format!("You: {}", input)))?;
        }
    }

    // Help or message
    execute!(stdout, MoveTo(0, bottom_y))?;
    if let Some(ref message) = app.message {
        execute!(
            stdout,
            SetForegroundColor(Color::Red),
            Print(format!("Message: {}", message)),
            ResetColor
        )?;
    } else {
        let help = match &app.state {
            AppState::Menu(_) => "Use ↑/↓ or j/k to navigate, Enter to select, q/5 to quit",
            AppState::PromptWebSearch => "Press y or n to enable/disable web search",
            AppState::Chat(_, _, _, _, _) => {
                "Type message, Enter to send, ↑/↓ to scroll, q to quit"
            }
            AppState::ListSessions(_, _, _) => {
                "Use ↑/↓ or j/k to navigate, Enter to view, Esc to back"
            }
            AppState::ViewSession(_, _, _, _) => "Use ↑/↓ or j/k to scroll, Esc to back",
        };
        execute!(
            stdout,
            SetForegroundColor(Color::Grey),
            Print(help),
            ResetColor
        )?;
    }

    stdout.flush()?;
    Ok(())
}
