use crate::core::Message;

#[derive(Clone)]
pub enum AppState {
    Menu(usize),
    #[allow(dead_code)]
    Chat(Vec<Message>, String, bool, bool), // messages, input, web_search, web_search_enabled
    Sessions(Vec<SessionInfo>, usize), // sessions, selected
    Tools(usize),                      // selected tool
    #[allow(dead_code)]
    ViewSession(String, Vec<Message>, usize), // name, messages, selected
}

#[derive(Clone)]
pub struct SessionInfo {
    #[allow(dead_code)]
    pub id: String,
    pub name: String,
    pub created_at: String,
}

#[derive(Clone)]
pub struct TuiApp {
    pub state: AppState,
    #[allow(dead_code)]
    pub should_quit: bool,
    pub message: Option<String>,
}

impl Default for TuiApp {
    fn default() -> Self {
        Self {
            state: AppState::Menu(0),
            should_quit: false,
            message: None,
        }
    }
}

impl TuiApp {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn next(&mut self) {
        match &mut self.state {
            AppState::Menu(sel) => *sel = (*sel + 1) % 6,
            AppState::Chat(_, _, _, _) => {} // TODO: scroll messages
            AppState::Sessions(_, sel) => *sel = (*sel + 1) % 10, // TODO: proper length
            AppState::Tools(sel) => *sel = (*sel + 1) % 5,
            AppState::ViewSession(_, _, sel) => *sel = (*sel + 1) % 10,
        }
    }

    pub fn previous(&mut self) {
        match &mut self.state {
            AppState::Menu(sel) => *sel = if *sel == 0 { 5 } else { *sel - 1 },
            AppState::Chat(_, _, _, _) => {} // TODO: scroll messages
            AppState::Sessions(_, sel) => *sel = if *sel == 0 { 9 } else { *sel - 1 },
            AppState::Tools(sel) => *sel = if *sel == 0 { 4 } else { *sel - 1 },
            AppState::ViewSession(_, _, sel) => *sel = if *sel == 0 { 9 } else { *sel - 1 },
        }
    }
}
