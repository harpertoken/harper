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

use harper_core::core::Message;

#[derive(Clone)]
pub struct ChatState {
    pub session_id: String,
    pub messages: Vec<Message>,
    pub input: String,
    pub web_search: bool,
    pub web_search_enabled: bool,
    pub completion_candidates: Vec<String>,
    pub completion_index: usize,
    pub scroll_offset: usize,
    pub completion_prefix: Option<String>,
}

impl ChatState {
    pub fn reset_completion(&mut self) {
        self.completion_candidates.clear();
        self.completion_index = 0;
        self.completion_prefix = None;
    }
}

#[derive(Clone)]
pub enum AppState {
    Menu(usize),
    Chat(ChatState),
    Sessions(Vec<SessionInfo>, usize),        // sessions, selected
    ExportSessions(Vec<SessionInfo>, usize),  // sessions, selected for export
    Tools(usize),                             // selected tool
    ViewSession(String, Vec<Message>, usize), // name, messages, selected
}

#[derive(Clone)]
pub struct SessionInfo {
    pub id: String,
    pub name: String,
    pub created_at: String,
}

#[derive(Clone, Debug)]
pub enum MessageType {
    Error,
    Help,
    Status,
    Info,
}

#[derive(Clone)]
pub struct UiMessage {
    pub content: String,
    pub message_type: MessageType,
}

#[derive(Clone)]
pub struct TuiApp {
    pub state: AppState,
    pub message: Option<UiMessage>,
}

impl Default for TuiApp {
    fn default() -> Self {
        Self {
            state: AppState::Menu(0),
            message: None,
        }
    }
}

impl TuiApp {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn set_error_message(&mut self, content: String) {
        self.message = Some(UiMessage {
            content,
            message_type: MessageType::Error,
        });
    }

    pub fn set_help_message(&mut self, content: String) {
        self.message = Some(UiMessage {
            content,
            message_type: MessageType::Help,
        });
    }

    pub fn set_status_message(&mut self, content: String) {
        self.message = Some(UiMessage {
            content,
            message_type: MessageType::Status,
        });
    }

    pub fn set_info_message(&mut self, content: String) {
        self.message = Some(UiMessage {
            content,
            message_type: MessageType::Info,
        });
    }

    pub fn clear_message(&mut self) {
        self.message = None;
    }

    pub fn next(&mut self) {
        match &mut self.state {
            AppState::Menu(sel) => *sel = (*sel + 1) % 6,
            AppState::Chat(chat_state) => {
                chat_state.scroll_offset =
                    (chat_state.scroll_offset + 1).min(chat_state.messages.len());
            }
            AppState::Sessions(sessions, sel) => {
                if !sessions.is_empty() {
                    *sel = (*sel + 1) % sessions.len();
                }
            }
            AppState::Tools(sel) => *sel = (*sel + 1) % 5,
            AppState::ExportSessions(sessions, sel) => {
                if !sessions.is_empty() {
                    *sel = (*sel + 1) % sessions.len();
                }
            }
            AppState::ViewSession(_, messages, sel) => {
                if !messages.is_empty() {
                    *sel = (*sel + 1) % messages.len();
                }
            }
        }
    }

    pub fn previous(&mut self) {
        match &mut self.state {
            AppState::Menu(sel) => *sel = if *sel == 0 { 5 } else { *sel - 1 },
            AppState::Chat(chat_state) => {
                chat_state.scroll_offset = chat_state.scroll_offset.saturating_sub(1);
            }
            AppState::Sessions(sessions, sel) => {
                if !sessions.is_empty() {
                    *sel = if *sel == 0 {
                        sessions.len() - 1
                    } else {
                        *sel - 1
                    };
                }
            }
            AppState::Tools(sel) => *sel = if *sel == 0 { 4 } else { *sel - 1 },
            AppState::ExportSessions(sessions, sel) => {
                if !sessions.is_empty() {
                    *sel = if *sel == 0 {
                        sessions.len() - 1
                    } else {
                        *sel - 1
                    };
                }
            }
            AppState::ViewSession(_, messages, sel) => {
                if !messages.is_empty() {
                    *sel = if *sel == 0 {
                        messages.len() - 1
                    } else {
                        *sel - 1
                    };
                }
            }
        }
    }
}
