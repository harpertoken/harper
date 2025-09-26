use crossterm::cursor::{Hide, MoveTo, Show};
use crossterm::event::{self, Event, KeyCode};
use crossterm::execute;
use crossterm::style::{Color, Print, ResetColor, SetForegroundColor};
use crossterm::terminal::{
    disable_raw_mode, enable_raw_mode, Clear, ClearType, EnterAlternateScreen, LeaveAlternateScreen,
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
    ListSessions(Vec<Session>, usize),
    ViewSession(String, Vec<Message>, usize),
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
            AppState::ListSessions(sessions, _) => sessions.len(),
            AppState::ViewSession(_, history, _) => history.len(),
        }
    }

    fn get_current_selected(&self) -> usize {
        match &self.state {
            AppState::Menu(sel) => *sel,
            AppState::ListSessions(_, sel) => *sel,
            AppState::ViewSession(_, _, sel) => *sel,
        }
    }

    fn set_current_selected(&mut self, sel: usize) {
        match &mut self.state {
            AppState::Menu(s) => *s = sel,
            AppState::ListSessions(_, s) => *s = sel,
            AppState::ViewSession(_, _, s) => *s = sel,
        }
    }

    pub fn next(&mut self) {
        let len = self.get_current_len();
        if len > 0 {
            let current = self.get_current_selected();
            self.set_current_selected((current + 1) % len);
        }
    }

    pub fn previous(&mut self) {
        let len = self.get_current_len();
        if len > 0 {
            let current = self.get_current_selected();
            self.set_current_selected(if current > 0 { current - 1 } else { len - 1 });
        }
    }

    pub fn select(&self) -> Option<MenuItem> {
        match &self.state {
            AppState::Menu(sel) => self.menu_items.get(*sel).cloned(),
            _ => None,
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
                    KeyCode::Char('q') | KeyCode::Char('5') => app.should_quit = true,
                    KeyCode::Esc => {
                        match &app.state {
                            AppState::Menu(_) => app.should_quit = true,
                            AppState::ListSessions(_, _) => {
                                app.state = AppState::Menu(0);
                            }
                            AppState::ViewSession(_, _, _) => {
                                // Go back to list
                                match session_service.list_sessions_data() {
                                    Ok(sessions) => {
                                        app.state = AppState::ListSessions(sessions, 0);
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
                                            // Temporarily disable raw mode for chat
                                            disable_raw_mode().ok();
                                            execute!(stdout, LeaveAlternateScreen, Show).ok();
                                            let mut chat_service = ChatService::new(
                                                conn,
                                                api_config,
                                                Some(&mut api_cache),
                                            );
                                            if let Err(e) = chat_service.start_session().await {
                                                app.message =
                                                    Some(format!("Error in chat session: {}", e));
                                            }
                                            // Re-enable for TUI
                                            enable_raw_mode().ok();
                                            execute!(stdout, EnterAlternateScreen, Hide).ok();
                                        }
                                        MenuItem::ListSessions => {
                                            match session_service.list_sessions_data() {
                                                Ok(sessions) => {
                                                    app.state = AppState::ListSessions(sessions, 0);
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
                                                    app.state = AppState::ListSessions(sessions, 0);
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
                            AppState::ListSessions(sessions, sel) => {
                                if let Some(session) = sessions.get(*sel) {
                                    match session_service.view_session_data(&session.id) {
                                        Ok(history) => {
                                            app.state = AppState::ViewSession(
                                                session.id.clone(),
                                                history,
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
                            AppState::ViewSession(_, _, _) => {
                                // Maybe scroll or something, but for now, do nothing
                            }
                        }
                    }
                    _ => {}
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
        AppState::ListSessions(sessions, sel) => {
            execute!(stdout, MoveTo(0, 2), Print("Sessions:"), MoveTo(0, start_y))?;
            for (i, session) in sessions.iter().enumerate() {
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
                execute!(stdout, MoveTo(0, start_y + i as u16 + 1))?;
            }
        }
        AppState::ViewSession(session_id, history, sel) => {
            execute!(
                stdout,
                MoveTo(0, 2),
                Print(format!("Session: {}", session_id)),
                MoveTo(0, start_y)
            )?;
            for (i, msg) in history.iter().enumerate() {
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
                execute!(stdout, MoveTo(0, start_y + i as u16 + 1))?;
            }
        }
    }

    // Help or message
    let bottom_y = 20; // Assume some height
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
            AppState::ListSessions(_, _) => {
                "Use ↑/↓ or j/k to navigate, Enter to view, Esc to back"
            }
            AppState::ViewSession(_, _, _) => "Use ↑/↓ or j/k to scroll, Esc to back",
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
