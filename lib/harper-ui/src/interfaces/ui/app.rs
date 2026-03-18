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
use std::fs;
use std::sync::{Arc, Mutex};
use tokio::sync::oneshot;

// Constants for sidebar entry limits
const MAX_SIDEBAR_PROBE_ENTRIES: usize = 5;
const MAX_SIDEBAR_GIT_ENTRIES: usize = 10;
const MAX_SIDEBAR_FILE_ENTRIES: usize = 12;

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
    pub sidebar_visible: bool,
    pub sidebar_entries: Vec<String>,
}

pub fn gather_sidebar_entries(chat_state: Option<&ChatState>) -> Vec<String> {
    let mut entries = Vec::new();
    if let Some(chat) = chat_state {
        for msg in chat.messages.iter().rev() {
            for line in msg.content.lines().rev() {
                let trimmed = line.trim();
                if trimmed.starts_with("$ ") {
                    entries.push(format!("[probe] {}", trimmed.trim_start_matches("$ ")));
                }
                if entries.len() >= 5 {
                    break;
                }
            }
            if entries.len() >= 5 {
                break;
            }
        }
    }

    if let Ok(status) = harper_core::tools::git::git_status() {
        for line in status.lines() {
            let trimmed = line.trim();
            if trimmed.is_empty()
                || trimmed.starts_with("Git status")
                || trimmed.starts_with("Git working")
            {
                continue;
            }
            entries.push(format!("[git] {}", trimmed));
            if entries.len() >= 10 {
                break;
            }
        }
    }

    if entries.len() < 10 {
        if let Ok(dir) = fs::read_dir(".") {
            for entry in dir.flatten() {
                let name = entry.file_name().to_string_lossy().into_owned();
                if name.starts_with('.') {
                    continue;
                }
                let kind = entry
                    .file_type()
                    .map(|ft| if ft.is_dir() { "dir" } else { "file" })
                    .unwrap_or("item");
                entries.push(format!("[{}] {}", kind, name));
                if entries.len() >= 12 {
                    break;
                }
            }
        }
    }

    if entries.is_empty() {
        entries.push("No harvest context yet".to_string());
    }
    entries
}

/// Gather sidebar entries asynchronously
pub async fn gather_sidebar_entries_async(messages: &[Message]) -> Vec<String> {
    let mut entries = Vec::new();
    for msg in messages.iter().rev() {
        for line in msg.content.lines().rev() {
            let trimmed = line.trim();
            if trimmed.starts_with("$ ") {
                entries.push(format!("[probe] {}", trimmed.trim_start_matches("$ ")));
            }
            if entries.len() >= MAX_SIDEBAR_PROBE_ENTRIES {
                break;
            }
        }
        if entries.len() >= MAX_SIDEBAR_PROBE_ENTRIES {
            break;
        }
    }

    if let Ok(status) = harper_core::tools::git::git_status_async().await {
        for line in status.lines() {
            let trimmed = line.trim();
            if trimmed.is_empty()
                || trimmed.starts_with("Git status")
                || trimmed.starts_with("Git working")
            {
                continue;
            }
            entries.push(format!("[git] {}", trimmed));
            if entries.len() >= MAX_SIDEBAR_GIT_ENTRIES {
                break;
            }
        }
    }

    if entries.len() < MAX_SIDEBAR_GIT_ENTRIES {
        let mut dir = match tokio::fs::read_dir(".").await {
            Ok(dir) => dir,
            Err(_) => {
                // If we can't read the directory, just skip it
                if entries.is_empty() {
                    entries.push("No harvest context yet".to_string());
                }
                return entries;
            }
        };
        while let Ok(Some(entry)) = dir.next_entry().await {
            let name = entry.file_name().to_string_lossy().into_owned();
            if name.starts_with('.') {
                continue;
            }
            let kind = entry
                .file_type()
                .await
                .map(|ft| if ft.is_dir() { "dir" } else { "file" })
                .unwrap_or("item");
            entries.push(format!("[{}] {}", kind, name));
            if entries.len() >= MAX_SIDEBAR_FILE_ENTRIES {
                break;
            }
        }
    }

    if entries.is_empty() {
        entries.push("No harvest context yet".to_string());
    }
    entries
}

impl ChatState {
    pub fn reset_completion(&mut self) {
        self.completion_candidates.clear();
        self.completion_index = 0;
        self.completion_prefix = None;
    }
}

pub struct ApprovalState {
    pub prompt: String,
    pub command: String,
    pub tx: Arc<Mutex<Option<oneshot::Sender<bool>>>>,
    pub scroll_offset: u16,
}

impl Clone for ApprovalState {
    fn clone(&self) -> Self {
        Self {
            prompt: self.prompt.clone(),
            command: self.command.clone(),
            tx: self.tx.clone(),
            scroll_offset: self.scroll_offset,
        }
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
    pub pending_approval: Option<ApprovalState>,
    pub cut_buffer: String,
    pub approval_history: Vec<String>,
}

impl Default for TuiApp {
    fn default() -> Self {
        Self {
            state: AppState::Menu(0),
            message: None,
            pending_approval: None,
            cut_buffer: String::new(),
            approval_history: Vec::new(),
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
        if let Some(approval) = &mut self.pending_approval {
            approval.scroll_offset = approval.scroll_offset.saturating_add(1);
            return;
        }

        match &mut self.state {
            AppState::Menu(sel) => *sel = (*sel + 1) % 5,
            AppState::Chat(chat_state) => {
                chat_state.scroll_offset =
                    (chat_state.scroll_offset + 1).min(chat_state.messages.len());
            }
            AppState::Sessions(sessions, sel) => {
                if !sessions.is_empty() {
                    *sel = (*sel + 1) % sessions.len();
                }
            }
            AppState::Tools(sel) => *sel = (*sel + 1) % 4,
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
        if let Some(approval) = &mut self.pending_approval {
            approval.scroll_offset = approval.scroll_offset.saturating_sub(1);
            return;
        }

        match &mut self.state {
            AppState::Menu(sel) => *sel = if *sel == 0 { 4 } else { *sel - 1 },
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
            AppState::Tools(sel) => *sel = if *sel == 0 { 3 } else { *sel - 1 },
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
