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
use harper_core::ResolvedAgents;
use harper_core::{ApprovalProfile, AuthSession, PlanState, SandboxProfile};
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
    PlanJobs,
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
    pub plan_job_selected: usize,
    pub plan_jobs_expanded: bool,
    pub plan_job_output_scroll: usize,
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
    pub fn refresh_plan_state(&mut self) {
        if let Some(plan) = &self.active_plan {
            let max_jobs = plan
                .runtime
                .as_ref()
                .map(|runtime| runtime.jobs.len().saturating_sub(1))
                .unwrap_or(0);
            self.plan_job_selected = self.plan_job_selected.min(max_jobs);
        } else {
            self.plan_job_selected = 0;
            self.plan_jobs_expanded = false;
            self.plan_job_output_scroll = 0;
        }
        self.normalize_navigation_focus();
    }

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
        let has_jobs = self
            .active_plan
            .as_ref()
            .and_then(|plan| plan.runtime.as_ref())
            .is_some_and(|runtime| !runtime.jobs.is_empty());
        let has_agents = self.agents_panel_expanded
            && self
                .active_agents
                .as_ref()
                .is_some_and(|agents| !agents.effective_rule_sections.is_empty());

        self.navigation_focus = match self.navigation_focus {
            NavigationFocus::Review if has_review => NavigationFocus::Review,
            NavigationFocus::PlanJobs if has_jobs => NavigationFocus::PlanJobs,
            NavigationFocus::Agents if has_agents => NavigationFocus::Agents,
            _ => NavigationFocus::Messages,
        };
    }

    pub fn navigation_focus_label(&self) -> &'static str {
        match self.navigation_focus {
            NavigationFocus::Messages => "messages",
            NavigationFocus::Review => "findings",
            NavigationFocus::PlanJobs => "jobs",
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
        self.refresh_plan_state();
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
    Sessions(Vec<SessionInfo>, usize),       // sessions, selected
    ExportSessions(Vec<SessionInfo>, usize), // sessions, selected for export
    Tools(usize),                            // selected tool
    Profile(usize),
    ExecutionPolicy(usize),
    ViewSession(String, Vec<Message>, usize), // name, messages, selected
    Stats(GlobalStats),
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ExecutionPolicyListField {
    AllowedCommands,
    BlockedCommands,
}

#[derive(Clone, Debug)]
pub struct ExecutionPolicyEditorState {
    pub field: ExecutionPolicyListField,
    pub input: String,
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
    pub expires_at: Option<Instant>,
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
    pub auth_session: Option<AuthSession>,
    pub auth_flow_id: Option<String>,
    pub auth_server_base_url: Option<String>,
    pub auth_last_poll_at: Option<Instant>,
    pub approval_profile: ApprovalProfile,
    pub sandbox_profile: SandboxProfile,
    pub allowed_commands: Vec<String>,
    pub blocked_commands: Vec<String>,
    pub execution_policy_editor: Option<ExecutionPolicyEditorState>,
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
            auth_session: None,
            auth_flow_id: None,
            auth_server_base_url: None,
            auth_last_poll_at: None,
            approval_profile: ApprovalProfile::AllowListed,
            sandbox_profile: SandboxProfile::Disabled,
            allowed_commands: Vec::new(),
            blocked_commands: Vec::new(),
            execution_policy_editor: None,
        }
    }
}

impl TuiApp {
    fn set_message(
        &mut self,
        content: String,
        message_type: MessageType,
        expires_after: Option<Duration>,
    ) {
        self.message = Some(UiMessage {
            content,
            message_type,
            expires_at: expires_after.map(|duration| Instant::now() + duration),
        });
    }

    pub fn new() -> Self {
        Self::default()
    }

    pub fn profile_action_count(&self) -> usize {
        if self.auth_session.is_some() {
            2
        } else {
            3
        }
    }

    pub fn execution_policy_row_count(&self) -> usize {
        5
    }

    pub fn set_error_message(&mut self, content: String) {
        self.set_message(content, MessageType::Error, None);
    }

    pub fn set_help_message(&mut self, content: String) {
        self.set_message(content, MessageType::Help, None);
    }

    pub fn set_status_message(&mut self, content: String) {
        self.set_message(content, MessageType::Status, Some(Duration::from_secs(2)));
    }

    pub fn set_info_message(&mut self, content: String) {
        self.set_message(content, MessageType::Info, Some(Duration::from_secs(3)));
    }

    pub fn clear_message(&mut self) {
        self.message = None;
    }

    pub fn refresh_message(&mut self) {
        if self
            .message
            .as_ref()
            .and_then(|message| message.expires_at)
            .is_some_and(|expires_at| Instant::now() >= expires_at)
        {
            self.message = None;
        }
    }

    pub fn auth_status_label(&self) -> String {
        match &self.auth_session {
            Some(session) => session
                .user
                .email
                .clone()
                .unwrap_or_else(|| format!("signed in: {}", session.user.user_id)),
            None => "auth: signed out".to_string(),
        }
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

        let profile_action_count = self.profile_action_count();
        let execution_policy_row_count = self.execution_policy_row_count();
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
                    NavigationFocus::PlanJobs => {
                        if chat_state.plan_jobs_expanded {
                            chat_state.plan_job_output_scroll =
                                chat_state.plan_job_output_scroll.saturating_add(1);
                        } else if let Some(runtime) = chat_state
                            .active_plan
                            .as_ref()
                            .and_then(|plan| plan.runtime.as_ref())
                        {
                            if !runtime.jobs.is_empty() {
                                chat_state.plan_job_selected =
                                    (chat_state.plan_job_selected + 1).min(runtime.jobs.len() - 1);
                                chat_state.plan_job_output_scroll = 0;
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
            AppState::Tools(sel) => *sel = (*sel + 1) % 5,
            AppState::Profile(sel) => *sel = (*sel + 1) % profile_action_count,
            AppState::ExecutionPolicy(sel) => *sel = (*sel + 1) % execution_policy_row_count,
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

        let profile_action_count = self.profile_action_count();
        let execution_policy_row_count = self.execution_policy_row_count();
        match &mut self.state {
            AppState::Menu(sel) => *sel = if *sel == 0 { 5 } else { *sel - 1 },
            AppState::Chat(chat_state) => {
                chat_state.normalize_navigation_focus();
                match chat_state.navigation_focus {
                    NavigationFocus::Review => {
                        chat_state.review_selected = chat_state.review_selected.saturating_sub(1);
                    }
                    NavigationFocus::PlanJobs => {
                        if chat_state.plan_jobs_expanded {
                            chat_state.plan_job_output_scroll =
                                chat_state.plan_job_output_scroll.saturating_sub(1);
                        } else {
                            chat_state.plan_job_selected =
                                chat_state.plan_job_selected.saturating_sub(1);
                            chat_state.plan_job_output_scroll = 0;
                        }
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
            AppState::Tools(sel) => *sel = if *sel == 0 { 4 } else { *sel - 1 },
            AppState::Profile(sel) => {
                *sel = if *sel == 0 {
                    profile_action_count - 1
                } else {
                    *sel - 1
                };
            }
            AppState::ExecutionPolicy(sel) => {
                *sel = if *sel == 0 {
                    execution_policy_row_count - 1
                } else {
                    *sel - 1
                }
            }
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

#[cfg(test)]
mod tests {
    use super::{MessageType, TuiApp, UiMessage};
    use std::time::{Duration, Instant};

    #[test]
    fn status_messages_expire() {
        let mut app = TuiApp::new();
        app.set_status_message("Signed in".to_string());

        assert!(matches!(
            app.message.as_ref().map(|message| &message.message_type),
            Some(MessageType::Status)
        ));
        assert!(app
            .message
            .as_ref()
            .and_then(|message| message.expires_at)
            .is_some());

        if let Some(message) = &mut app.message {
            message.expires_at = Some(Instant::now() - Duration::from_millis(1));
        }

        app.refresh_message();
        assert!(app.message.is_none());
    }

    #[test]
    fn help_messages_do_not_expire() {
        let mut app = TuiApp::new();
        app.message = Some(UiMessage {
            content: "Help".to_string(),
            message_type: MessageType::Help,
            expires_at: None,
        });

        app.refresh_message();
        assert!(app.message.is_some());
    }
}
