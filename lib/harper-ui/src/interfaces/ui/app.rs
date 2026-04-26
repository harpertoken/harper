// Copyright 2026 harpertoken
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
use harper_core::memory::session_service::GlobalStats;
use harper_core::PlanState;
use harper_core::ResolvedAgents;
use serde::Deserialize;
use std::fs;
use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};
use tokio::sync::oneshot;

// Constants for sidebar entry limits
const MAX_SIDEBAR_PROBE_ENTRIES: usize = 5;
const MAX_SIDEBAR_GIT_ENTRIES: usize = 10;
const MAX_SIDEBAR_FILE_ENTRIES: usize = 12;

#[derive(Clone, Debug, Deserialize)]
pub struct ReviewFindingState {
    pub title: String,
    pub severity: String,
    pub message: String,
}

#[derive(Clone, Debug, Deserialize)]
pub struct ReviewState {
    pub summary: String,
    pub findings: Vec<ReviewFindingState>,
    pub model: Option<String>,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum NavigationFocus {
    Messages,
    Review,
    Agents,
}

#[derive(Clone)]
pub struct ChatState {
    pub session_id: String,
    pub messages: Vec<Message>,
    pub awaiting_response: bool,
    pub active_plan: Option<PlanState>,
    pub active_agents: Option<ResolvedAgents>,
    pub active_review: Option<ReviewState>,
    pub review_selected: usize,
    pub navigation_focus: NavigationFocus,
    pub command_output: Option<CommandOutputState>,
    pub agents_panel_expanded: bool,
    pub agents_scroll_offset: usize,
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

#[derive(Clone)]
pub struct CommandOutputState {
    pub command: String,
    pub content: String,
    pub has_error: bool,
    pub done: bool,
}

pub fn gather_sidebar_entries(chat_state: Option<&ChatState>) -> Vec<String> {
    let mut entries = Vec::new();
    if let Some(chat) = chat_state {
        for msg in chat.messages.iter().rev() {
            for line in msg.content.lines().rev() {
                let trimmed = line.trim();
                if trimmed.starts_with("$ ") {
                    entries.push(trimmed.trim_start_matches("$ ").to_string());
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
            entries.push(trimmed.to_string());
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
                entries.push(name);
                if entries.len() >= 12 {
                    break;
                }
            }
        }
    }

    if entries.is_empty() {
        entries.push("Empty context".to_string());
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
                entries.push(trimmed.trim_start_matches("$ ").to_string());
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
            entries.push(trimmed.to_string());
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
                    entries.push("Empty context".to_string());
                }
                return entries;
            }
        };
        while let Ok(Some(entry)) = dir.next_entry().await {
            let name = entry.file_name().to_string_lossy().into_owned();
            if name.starts_with('.') {
                continue;
            }
            entries.push(name);
            if entries.len() >= MAX_SIDEBAR_FILE_ENTRIES {
                break;
            }
        }
    }

    if entries.is_empty() {
        entries.push("Empty context".to_string());
    }
    entries
}

impl ChatState {
    pub fn reset_completion(&mut self) {
        self.completion_candidates.clear();
        self.completion_index = 0;
        self.completion_prefix = None;
    }

    pub fn set_navigation_focus(&mut self, focus: NavigationFocus) {
        self.navigation_focus = focus;
        self.normalize_navigation_focus();
    }

    pub fn normalize_navigation_focus(&mut self) {
        let has_review = self
            .active_review
            .as_ref()
            .is_some_and(|review| !review.findings.is_empty());
        let has_agents = self.agents_panel_expanded
            && self
                .active_agents
                .as_ref()
                .is_some_and(|agents| !agents.effective_rule_sections.is_empty());

        self.navigation_focus = match self.navigation_focus {
            NavigationFocus::Review if has_review => NavigationFocus::Review,
            NavigationFocus::Agents if has_agents => NavigationFocus::Agents,
            _ => NavigationFocus::Messages,
        };
    }

    pub fn navigation_focus_label(&self) -> &'static str {
        match self.navigation_focus {
            NavigationFocus::Messages => "messages",
            NavigationFocus::Review => "findings",
            NavigationFocus::Agents => "agents",
        }
    }

    pub fn refresh_review_state(&mut self) {
        self.active_review = derive_review_state(&self.messages);
        if let Some(review) = &self.active_review {
            if review.findings.is_empty() {
                self.review_selected = 0;
            } else {
                self.review_selected = self.review_selected.min(review.findings.len() - 1);
            }
        } else {
            self.review_selected = 0;
        }
        self.normalize_navigation_focus();
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
    Chat(Box<ChatState>),
    Sessions(Vec<SessionInfo>, usize),        // sessions, selected
    ExportSessions(Vec<SessionInfo>, usize),  // sessions, selected for export
    Tools(usize),                             // selected tool
    ViewSession(String, Vec<Message>, usize), // name, messages, selected
    Stats(GlobalStats),
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
    pub activity_status: Option<String>,
    pub activity_started_at: Option<Instant>,
    pub activity_clear_pending: bool,
    pub cut_buffer: String,
    pub approval_history: Vec<String>,
    pub model_label: String,
}

impl Default for TuiApp {
    fn default() -> Self {
        Self {
            state: AppState::Menu(0),
            message: None,
            pending_approval: None,
            activity_status: None,
            activity_started_at: None,
            activity_clear_pending: false,
            cut_buffer: String::new(),
            approval_history: Vec::new(),
            model_label: String::new(),
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

    pub fn set_activity_status(&mut self, status: Option<String>) {
        const MIN_ACTIVITY_VISIBLE: Duration = Duration::from_millis(700);

        match status {
            Some(status) => {
                self.activity_status = Some(status);
                self.activity_started_at = Some(Instant::now());
                self.activity_clear_pending = false;
            }
            None => {
                let should_clear_now = self
                    .activity_started_at
                    .map(|started| started.elapsed() >= MIN_ACTIVITY_VISIBLE)
                    .unwrap_or(true);
                if should_clear_now {
                    self.activity_status = None;
                    self.activity_started_at = None;
                    self.activity_clear_pending = false;
                } else {
                    self.activity_clear_pending = true;
                }
            }
        }
    }

    pub fn refresh_activity_status(&mut self) {
        if !self.activity_clear_pending {
            return;
        }
        if self
            .activity_started_at
            .map(|started| started.elapsed() >= Duration::from_millis(700))
            .unwrap_or(true)
        {
            self.activity_status = None;
            self.activity_started_at = None;
            self.activity_clear_pending = false;
        }
    }

    pub fn next(&mut self) {
        if let Some(approval) = &mut self.pending_approval {
            approval.scroll_offset = approval.scroll_offset.saturating_add(1);
            return;
        }

        match &mut self.state {
            AppState::Menu(sel) => *sel = (*sel + 1) % 6,
            AppState::Chat(chat_state) => {
                chat_state.normalize_navigation_focus();
                match chat_state.navigation_focus {
                    NavigationFocus::Review => {
                        if let Some(review) = &chat_state.active_review {
                            if !review.findings.is_empty() {
                                chat_state.review_selected =
                                    (chat_state.review_selected + 1).min(review.findings.len() - 1);
                            }
                        }
                    }
                    NavigationFocus::Agents => {
                        chat_state.agents_scroll_offset =
                            chat_state.agents_scroll_offset.saturating_add(1);
                    }
                    NavigationFocus::Messages => {
                        chat_state.scroll_offset =
                            (chat_state.scroll_offset + 1).min(chat_state.messages.len());
                    }
                }
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
            AppState::Stats(_) => {}
        }
    }

    pub fn previous(&mut self) {
        if let Some(approval) = &mut self.pending_approval {
            approval.scroll_offset = approval.scroll_offset.saturating_sub(1);
            return;
        }

        match &mut self.state {
            AppState::Menu(sel) => *sel = if *sel == 0 { 5 } else { *sel - 1 },
            AppState::Chat(chat_state) => {
                chat_state.normalize_navigation_focus();
                match chat_state.navigation_focus {
                    NavigationFocus::Review => {
                        chat_state.review_selected = chat_state.review_selected.saturating_sub(1);
                    }
                    NavigationFocus::Agents => {
                        chat_state.agents_scroll_offset =
                            chat_state.agents_scroll_offset.saturating_sub(1);
                    }
                    NavigationFocus::Messages => {
                        chat_state.scroll_offset = chat_state.scroll_offset.saturating_sub(1);
                    }
                }
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
            AppState::Stats(_) => {}
        }
    }
}

fn derive_review_state(messages: &[Message]) -> Option<ReviewState> {
    messages
        .iter()
        .rev()
        .find_map(|message| parse_review_state(&message.content))
}

fn parse_review_state(content: &str) -> Option<ReviewState> {
    let trimmed = content.trim();
    let cleaned = if let Some(stripped) = trimmed.strip_prefix("```") {
        let without_lang = stripped
            .split_once('\n')
            .map(|(_, rest)| rest)
            .unwrap_or(stripped);
        without_lang
            .strip_suffix("```")
            .unwrap_or(without_lang)
            .trim()
    } else {
        trimmed
    };

    let value: serde_json::Value = serde_json::from_str(cleaned).ok()?;
    if value.get("summary").is_none() || value.get("findings").is_none() {
        return None;
    }
    serde_json::from_value(value).ok()
}
