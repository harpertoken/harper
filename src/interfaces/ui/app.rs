use crossterm::terminal::size;

#[derive(Clone)]
pub enum AppState {
    Menu(usize),
    PromptWebSearch,
    Chat(Vec<crate::core::Message>, usize, String, bool, String),
    ListSessions(Vec<crate::memory::session_service::Session>, usize, usize),
    ViewSession(String, Vec<crate::core::Message>, usize, usize),
}

#[derive(Clone)]
pub struct TuiApp {
    pub state: AppState,
    pub menu_items: Vec<MenuItem>,
    pub should_quit: bool,
    pub message: Option<String>,
    pub history: Vec<String>,
    pub history_index: usize,
}

#[derive(Clone)]
pub enum MenuItem {
    StartChat,
    ListSessions,
    ViewSession,
    ExportSession,
    Quit,
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

impl Default for TuiApp {
    fn default() -> Self {
        Self {
            state: AppState::Menu(0),
            menu_items: MenuItem::all(),
            should_quit: false,
            message: None,
            history: Vec::new(),
            history_index: 0,
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
            history: Vec::new(),
            history_index: 0,
        }
    }

    pub fn select(&self) -> Option<MenuItem> {
        match &self.state {
            AppState::Menu(sel) => self.menu_items.get(*sel).cloned(),
            _ => None,
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
        let (_, height) = size().unwrap_or((80, 24));
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
                    let (_, height) = size().unwrap_or((80, 24));
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
}
