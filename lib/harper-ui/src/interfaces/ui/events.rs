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

use arboard::{Clipboard, ImageData};
use crossterm::event::{Event, KeyCode, KeyEvent, KeyModifiers};
use harper_core;
use harper_core::PlanStepStatus;
use std::fs;
use std::path::Path;
use std::path::PathBuf;
use uuid::Uuid;

// Keyboard shortcut constants
const HELP_MESSAGE: &str =
    "G:Help | Tab:Complete | Esc:Back | ↑↓:Navigate | Y/V:Prev/Next | Enter:Select/Approve | T:Send | L/→:Preview | D/Delete:Remove Session | X:Exit | W:Web | B:Sidebar | A:Agents | /agents on|off | F:Findings | Ctrl+S:Plan | R:Retry | U:Replan | K:Ack | P:Jobs | M:Msgs | C:ID";

use super::app::{
    AppState, ChatState, ExecutionPolicyEditorState, ExecutionPolicyListField, NavigationFocus,
    SessionInfo, TuiApp,
};
use super::settings;
use harper_core::memory::session_service::SessionService;

// Constants
const MAX_APPROVAL_HISTORY: usize = 6;

pub enum EventResult {
    Continue,
    SendMessage(String),
    LoadSessions,
    OpenSession {
        session_id: String,
        preview: bool,
    },
    RefreshAuthSession,
    StartProfileLogin {
        provider: String,
    },
    SaveExecutionPolicy,
    SetPlanStepStatus {
        session_id: String,
        step_index: usize,
        status: PlanStepStatus,
    },
    ClearPlan {
        session_id: String,
    },
    RetryPlanFollowup {
        session_id: String,
        command: String,
    },
    RequestPlanReplan {
        session_id: String,
        step_index: usize,
        step: String,
    },
    ClearPlanFollowup {
        session_id: String,
    },
    DeleteSession {
        session_id: String,
        remote: bool,
        export_view: bool,
        selected_index: usize,
    },
    GatherSidebarEntries,
    Quit,
}

pub(crate) fn create_chat_state(
    session_id: String,
    messages: Vec<harper_core::core::Message>,
    active_plan: Option<harper_core::PlanState>,
    active_agents: Option<harper_core::ResolvedAgents>,
    agents_context_enabled: bool,
) -> ChatState {
    let active_agents = if agents_context_enabled {
        active_agents.or_else(|| {
            std::env::current_dir()
                .ok()
                .and_then(|cwd| harper_core::core::agents::resolve_agents_for_dir(&cwd).ok())
        })
    } else {
        None
    };

    let mut chat_state = ChatState {
        session_id,
        messages,
        awaiting_response: false,
        active_plan,
        active_agents,
        active_review: None,
        review_selected: 0,
        plan_step_selected: 0,
        plan_steps_expanded: false,
        plan_job_selected: 0,
        plan_jobs_expanded: false,
        plan_job_output_scroll: 0,
        navigation_focus: NavigationFocus::Messages,
        command_output: None,
        agents_panel_expanded: false,
        agents_scroll_offset: 0,
        input: String::new(),
        web_search: false,
        web_search_enabled: false,
        completion_candidates: vec![],
        completion_index: 0,
        scroll_offset: 0,
        completion_prefix: None,
        sidebar_visible: false,
        sidebar_sections: Vec::new(),
        rendered_message_cache: Vec::new(),
        rendered_transcript_lines: Vec::new(),
        render_cache_theme_key: String::new(),
    };
    chat_state.refresh_review_state();
    chat_state.follow_latest_messages();
    chat_state
}

fn record_approval_history(app: &mut TuiApp, command: &str, approved: bool) {
    let marker = if approved { "[Y]" } else { "[N]" };
    app.approval_history.push(format!("{} {}", marker, command));
    if app.approval_history.len() > MAX_APPROVAL_HISTORY {
        let excess = app.approval_history.len() - MAX_APPROVAL_HISTORY;
        app.approval_history.drain(0..excess);
    }
}

fn load_export_sessions_into_state(app: &mut TuiApp, session_service: &SessionService) {
    let sessions_result = if let Some(auth_session) = app.auth_session.as_ref() {
        session_service.list_sessions_data_for_user(&auth_session.user.user_id)
    } else {
        session_service.list_sessions_data()
    };
    match sessions_result {
        Ok(sessions) => {
            let session_infos: Vec<SessionInfo> = sessions
                .into_iter()
                .map(|s| SessionInfo {
                    id: s.id.clone(),
                    name: s.title.unwrap_or(s.id),
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

fn parse_command_list(input: &str) -> Vec<String> {
    input
        .split(',')
        .map(str::trim)
        .filter(|value| !value.is_empty())
        .map(ToOwned::to_owned)
        .collect()
}

fn start_execution_policy_editor(app: &mut TuiApp, field: ExecutionPolicyListField) {
    let input = match field {
        ExecutionPolicyListField::HeaderWidgets => {
            settings::header_widgets_summary(&app.header_widgets)
        }
        ExecutionPolicyListField::AllowedCommands => app.allowed_commands.join(", "),
        ExecutionPolicyListField::BlockedCommands => app.blocked_commands.join(", "),
    };
    app.execution_policy_editor = Some(ExecutionPolicyEditorState {
        field,
        input,
        selected_index: 0,
        text_input_focused: false,
    });
}

fn handle_plan_step_action(key: KeyEvent, app: &mut TuiApp) -> Option<EventResult> {
    if key.modifiers.contains(KeyModifiers::CONTROL) {
        return None;
    }
    let AppState::Chat(chat_state) = &mut app.state else {
        return None;
    };
    if !matches!(chat_state.navigation_focus, NavigationFocus::PlanSteps) {
        return None;
    }
    let Some(plan) = &chat_state.active_plan else {
        app.set_status_message("No active plan".to_string());
        return Some(EventResult::Continue);
    };
    if plan.items.is_empty() {
        app.set_status_message("No active plan".to_string());
        return Some(EventResult::Continue);
    }

    let session_id = chat_state.session_id.clone();
    let step_index = chat_state.plan_step_selected.min(plan.items.len() - 1);
    match key.code {
        KeyCode::Char('c') => Some(EventResult::SetPlanStepStatus {
            session_id,
            step_index,
            status: PlanStepStatus::Completed,
        }),
        KeyCode::Char('i') => Some(EventResult::SetPlanStepStatus {
            session_id,
            step_index,
            status: PlanStepStatus::InProgress,
        }),
        KeyCode::Char('b') => Some(EventResult::SetPlanStepStatus {
            session_id,
            step_index,
            status: PlanStepStatus::Blocked,
        }),
        KeyCode::Char('r') => plan
            .runtime
            .as_ref()
            .and_then(|runtime| runtime.followup.as_ref())
            .and_then(|followup| match followup {
                harper_core::core::plan::PlanFollowup::RetryOrReplan {
                    command: Some(command),
                    ..
                } => Some(EventResult::RetryPlanFollowup {
                    session_id,
                    command: command.clone(),
                }),
                _ => None,
            })
            .or_else(|| {
                app.set_status_message("No retryable planner command".to_string());
                Some(EventResult::Continue)
            }),
        KeyCode::Char('u') => plan
            .runtime
            .as_ref()
            .and_then(|runtime| runtime.followup.as_ref())
            .and_then(|followup| match followup {
                harper_core::core::plan::PlanFollowup::RetryOrReplan { step, .. } => {
                    Some(EventResult::RequestPlanReplan {
                        session_id,
                        step_index,
                        step: step.clone(),
                    })
                }
                _ => None,
            })
            .or_else(|| {
                app.set_status_message("No planner retry followup to replan".to_string());
                Some(EventResult::Continue)
            }),
        KeyCode::Char('k') => {
            if plan
                .runtime
                .as_ref()
                .and_then(|runtime| runtime.followup.as_ref())
                .is_some()
            {
                Some(EventResult::ClearPlanFollowup { session_id })
            } else {
                app.set_status_message("No planner followup to acknowledge".to_string());
                Some(EventResult::Continue)
            }
        }
        KeyCode::Char('x') => Some(EventResult::ClearPlan { session_id }),
        _ => None,
    }
}

pub fn handle_event(
    event: Event,
    app: &mut TuiApp,
    session_service: &SessionService,
) -> EventResult {
    match event {
        Event::Key(key) => {
            if let Some(message) = app.message.as_ref() {
                if matches!(message.message_type, super::app::MessageType::Help) {
                    let help_row_count = message
                        .content
                        .split('|')
                        .map(str::trim)
                        .filter(|item| !item.is_empty())
                        .collect::<Vec<_>>()
                        .chunks(2)
                        .len();

                    match key.code {
                        KeyCode::Esc => app.clear_message(),
                        KeyCode::Down | KeyCode::Char('j') if help_row_count > 0 => {
                            app.help_selected = (app.help_selected + 1) % help_row_count;
                        }
                        KeyCode::Up | KeyCode::Char('k') if help_row_count > 0 => {
                            app.help_selected = if app.help_selected == 0 {
                                help_row_count - 1
                            } else {
                                app.help_selected - 1
                            };
                        }
                        _ => {}
                    }
                    return EventResult::Continue;
                }
                app.clear_message();
            }

            // PRIORITIZE: Handle input for the security approval modal if active
            if let Some(approval) = &mut app.pending_approval {
                match key.code {
                    KeyCode::Char('y') | KeyCode::Char('Y') | KeyCode::Enter => {
                        let command = approval.command.clone();
                        if let Some(tx) = approval
                            .tx
                            .lock()
                            .expect("Failed to lock approval channel")
                            .take()
                        {
                            let _ = tx.send(true);
                        }
                        app.pending_approval = None;
                        app.set_activity_status(Some(format!("resuming: {}", command)));
                        record_approval_history(app, &command, true);
                        return EventResult::Continue;
                    }
                    KeyCode::Char('n') | KeyCode::Char('N') | KeyCode::Esc => {
                        let command = approval.command.clone();
                        if let Some(tx) = approval
                            .tx
                            .lock()
                            .expect("Failed to lock approval channel")
                            .take()
                        {
                            let _ = tx.send(false);
                        }
                        app.pending_approval = None;
                        app.set_activity_status(None);
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

            if let Some(editor) = &mut app.execution_policy_editor {
                if matches!(editor.field, ExecutionPolicyListField::HeaderWidgets) {
                    let widgets = settings::available_header_widgets();
                    match key.code {
                        KeyCode::Esc => {
                            app.execution_policy_editor = None;
                            return EventResult::Continue;
                        }
                        KeyCode::Tab => {
                            editor.text_input_focused = !editor.text_input_focused;
                            return EventResult::Continue;
                        }
                        KeyCode::Down | KeyCode::Char('j') => {
                            if editor.text_input_focused {
                                return EventResult::Continue;
                            }
                            if !widgets.is_empty() {
                                editor.selected_index = (editor.selected_index + 1) % widgets.len();
                            }
                            return EventResult::Continue;
                        }
                        KeyCode::Up | KeyCode::Char('k') => {
                            if editor.text_input_focused {
                                return EventResult::Continue;
                            }
                            if !widgets.is_empty() {
                                editor.selected_index = if editor.selected_index == 0 {
                                    widgets.len() - 1
                                } else {
                                    editor.selected_index - 1
                                };
                            }
                            return EventResult::Continue;
                        }
                        KeyCode::Char(' ') | KeyCode::Enter => {
                            if editor.text_input_focused {
                                return EventResult::Continue;
                            }
                            if let Some(widget) = widgets.get(editor.selected_index).copied() {
                                if widget == super::app::HeaderWidget::Model {
                                } else if app.header_widgets.contains(&widget) {
                                    app.header_widgets.retain(|item| *item != widget);
                                    editor.input =
                                        settings::header_widgets_summary(&app.header_widgets);
                                } else {
                                    app.header_widgets.push(widget);
                                    editor.input =
                                        settings::header_widgets_summary(&app.header_widgets);
                                }
                            }
                            return EventResult::Continue;
                        }
                        KeyCode::Char('s') => {
                            app.execution_policy_editor = None;
                            app.set_status_message("Updated execution policy settings".to_string());
                            return EventResult::Continue;
                        }
                        KeyCode::Backspace => {
                            if editor.text_input_focused {
                                editor.input.pop();
                                let values = parse_command_list(&editor.input);
                                app.header_widgets = settings::parse_header_widgets(&values);
                            }
                            return EventResult::Continue;
                        }
                        KeyCode::Char(c) if !key.modifiers.contains(KeyModifiers::CONTROL) => {
                            if editor.text_input_focused {
                                editor.input.push(c);
                                let values = parse_command_list(&editor.input);
                                app.header_widgets = settings::parse_header_widgets(&values);
                            }
                            return EventResult::Continue;
                        }
                        _ => return EventResult::Continue,
                    }
                }

                match key.code {
                    KeyCode::Esc => {
                        app.execution_policy_editor = None;
                        return EventResult::Continue;
                    }
                    KeyCode::Enter => {
                        let values = parse_command_list(&editor.input);
                        match editor.field {
                            ExecutionPolicyListField::HeaderWidgets => {
                                app.header_widgets = settings::parse_header_widgets(&values);
                            }
                            ExecutionPolicyListField::AllowedCommands => {
                                app.allowed_commands = values;
                            }
                            ExecutionPolicyListField::BlockedCommands => {
                                app.blocked_commands = values;
                            }
                        }
                        app.execution_policy_editor = None;
                        app.set_status_message("Updated execution policy settings".to_string());
                        return EventResult::Continue;
                    }
                    KeyCode::Backspace => {
                        editor.input.pop();
                        return EventResult::Continue;
                    }
                    KeyCode::Char(c) if !key.modifiers.contains(KeyModifiers::CONTROL) => {
                        editor.input.push(c);
                        return EventResult::Continue;
                    }
                    _ => return EventResult::Continue,
                }
            }

            if let Some(result) = handle_plan_step_action(key, app) {
                return result;
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
                        return EventResult::LoadSessions;
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
                                app.set_status_message("Context sidebar shown".to_string());
                                return EventResult::GatherSidebarEntries;
                            } else {
                                app.set_status_message("Context sidebar hidden".to_string());
                            }
                        }
                        return EventResult::Continue;
                    }
                    KeyCode::Char('a') => {
                        let mut status_message = None;
                        if let AppState::Chat(chat_state) = &mut app.state {
                            if chat_state.active_agents.is_some() {
                                if !chat_state.agents_panel_expanded {
                                    chat_state.agents_panel_expanded = true;
                                    chat_state.agents_scroll_offset = 0;
                                    chat_state.set_navigation_focus(NavigationFocus::Agents);
                                    status_message =
                                        Some("AGENTS panel expanded; focus on agents".to_string());
                                } else if matches!(
                                    chat_state.navigation_focus,
                                    NavigationFocus::Agents
                                ) {
                                    chat_state.agents_panel_expanded = false;
                                    chat_state.agents_scroll_offset = 0;
                                    chat_state.set_navigation_focus(NavigationFocus::Messages);
                                    status_message = Some(
                                        "AGENTS panel collapsed; focus on messages".to_string(),
                                    );
                                } else {
                                    chat_state.set_navigation_focus(NavigationFocus::Agents);
                                    status_message = Some("Focus on AGENTS panel".to_string());
                                }
                            } else {
                                if !chat_state.agents_panel_expanded {
                                    chat_state.agents_panel_expanded = true;
                                    chat_state.agents_scroll_offset = 0;
                                    status_message = Some(
                                        "AGENTS panel opened; no active sources yet".to_string(),
                                    );
                                } else {
                                    chat_state.agents_panel_expanded = false;
                                    status_message = Some("AGENTS panel collapsed".to_string());
                                }
                            }
                        }
                        if let Some(message) = status_message {
                            app.set_status_message(message);
                        }
                        return EventResult::Continue;
                    }
                    KeyCode::Char('f') => {
                        let mut status_message = None;
                        if let AppState::Chat(chat_state) = &mut app.state {
                            if chat_state
                                .active_review
                                .as_ref()
                                .is_some_and(|review| !review.findings.is_empty())
                            {
                                chat_state.set_navigation_focus(NavigationFocus::Review);
                                status_message = Some("Focus on review findings".to_string());
                            } else {
                                status_message = Some("No review findings".to_string());
                            }
                        }
                        if let Some(message) = status_message {
                            app.set_status_message(message);
                        }
                        return EventResult::Continue;
                    }
                    KeyCode::Char('m') => {
                        if let AppState::Chat(chat_state) = &mut app.state {
                            chat_state.set_navigation_focus(NavigationFocus::Messages);
                            app.set_status_message("Focus on messages".to_string());
                        }
                        return EventResult::Continue;
                    }
                    KeyCode::Char('p') => {
                        let mut status_message = None;
                        if let AppState::Chat(chat_state) = &mut app.state {
                            if chat_state
                                .active_plan
                                .as_ref()
                                .and_then(|plan| plan.runtime.as_ref())
                                .is_some_and(|runtime| !runtime.jobs.is_empty())
                            {
                                if !chat_state.plan_jobs_expanded {
                                    chat_state.plan_jobs_expanded = true;
                                    chat_state.set_navigation_focus(NavigationFocus::PlanJobs);
                                    status_message =
                                        Some("Planner jobs browser expanded".to_string());
                                } else if matches!(
                                    chat_state.navigation_focus,
                                    NavigationFocus::PlanJobs
                                ) {
                                    chat_state.plan_jobs_expanded = false;
                                    chat_state.plan_job_output_scroll = 0;
                                    chat_state.set_navigation_focus(NavigationFocus::Messages);
                                    status_message =
                                        Some("Planner jobs browser closed".to_string());
                                } else {
                                    chat_state.set_navigation_focus(NavigationFocus::PlanJobs);
                                    status_message = Some("Focus on planner jobs".to_string());
                                }
                            } else {
                                status_message = Some("No planner jobs".to_string());
                            }
                        }
                        if let Some(message) = status_message {
                            app.set_status_message(message);
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
                    KeyCode::Char('s') => {
                        let mut status_message = None;
                        if let AppState::Chat(chat_state) = &mut app.state {
                            if chat_state
                                .active_plan
                                .as_ref()
                                .is_some_and(|plan| !plan.items.is_empty())
                            {
                                if !chat_state.plan_steps_expanded {
                                    chat_state.plan_steps_expanded = true;
                                    chat_state.set_navigation_focus(NavigationFocus::PlanSteps);
                                    status_message = Some("Plan browser expanded".to_string());
                                } else if matches!(
                                    chat_state.navigation_focus,
                                    NavigationFocus::PlanSteps
                                ) {
                                    chat_state.plan_steps_expanded = false;
                                    chat_state.set_navigation_focus(NavigationFocus::Messages);
                                    status_message = Some("Plan browser closed".to_string());
                                } else {
                                    chat_state.set_navigation_focus(NavigationFocus::PlanSteps);
                                    status_message = Some("Focus on plan steps".to_string());
                                }
                            } else {
                                status_message = Some("No active plan".to_string());
                            }
                        }
                        if let Some(message) = status_message {
                            app.set_status_message(message);
                        }
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
                KeyCode::Esc => match &mut app.state {
                    AppState::Menu(_) => {}
                    AppState::Chat(chat_state) => {
                        if chat_state.plan_jobs_expanded {
                            chat_state.plan_jobs_expanded = false;
                            chat_state.plan_job_output_scroll = 0;
                            chat_state.set_navigation_focus(NavigationFocus::Messages);
                            app.set_status_message("Planner jobs browser closed".to_string());
                        } else if chat_state.plan_steps_expanded {
                            chat_state.plan_steps_expanded = false;
                            chat_state.set_navigation_focus(NavigationFocus::Messages);
                            app.set_status_message("Plan browser closed".to_string());
                        } else {
                            app.state = AppState::Menu(0);
                        }
                    }
                    AppState::Sessions(_, _) => app.state = AppState::Menu(0),
                    AppState::ExportSessions(_, _) => app.state = AppState::Menu(0),
                    AppState::Settings(_) => app.state = AppState::Menu(0),
                    AppState::Profile(_) | AppState::ExecutionPolicy(_) => {
                        app.state = AppState::Menu(0)
                    }
                    AppState::ViewSession(_, _, _) => app.state = AppState::Menu(0),
                    AppState::Stats(_) => app.state = AppState::Menu(0),
                },
                KeyCode::Down | KeyCode::Char('j') => {
                    if handle_completion_down(app) {
                        return EventResult::Continue;
                    }
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
                    if handle_completion_up(app) {
                        return EventResult::Continue;
                    }
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
                KeyCode::Right => return preview_selected_session(app, session_service),
                KeyCode::Delete => return delete_selected_session(app),
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
                    if c == 'l' {
                        if matches!(app.state, AppState::Chat(_)) {
                            handle_char_input(app, 'l');
                            return EventResult::Continue;
                        }
                        return preview_selected_session(app, session_service);
                    }
                    if c == 'd' && !matches!(app.state, AppState::Chat(_)) {
                        return delete_selected_session(app);
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
                    app.state = AppState::Chat(Box::new(create_chat_state(
                        Uuid::new_v4().to_string(),
                        vec![],
                        None,
                        None,
                        app.agents_context_enabled,
                    )));
                    return EventResult::GatherSidebarEntries;
                } // Start Chat
                1 => return EventResult::LoadSessions,
                2 => load_export_sessions_into_state(app, session_service),
                3 => {
                    let stats_result = if let Some(auth_session) = app.auth_session.as_ref() {
                        session_service.get_global_stats_for_user(&auth_session.user.user_id)
                    } else {
                        session_service.get_global_stats()
                    };
                    match stats_result {
                        Ok(stats) => app.state = AppState::Stats(stats),
                        Err(e) => app.set_error_message(format!("Error loading stats: {}", e)),
                    }
                }
                4 => app.state = AppState::Settings(0), // Settings
                5 => return EventResult::Quit,          // Exit
                _ => {}
            }
        }
        AppState::Chat(chat_state) if !chat_state.input.is_empty() => {
            let message = chat_state.input.clone();
            chat_state.input = String::new();
            return EventResult::SendMessage(message);
        }
        AppState::Sessions(sessions, selected)
            if !sessions.is_empty() && *selected < sessions.len() =>
        {
            let session = &sessions[*selected];
            return EventResult::OpenSession {
                session_id: session.id.clone(),
                preview: false,
            };
        }
        AppState::ExportSessions(sessions, selected)
            if !sessions.is_empty() && *selected < sessions.len() =>
        {
            let session = &sessions[*selected];
            let export_result = if let Some(auth_session) = app.auth_session.as_ref() {
                session_service
                    .export_session_by_id_for_user(&session.id, &auth_session.user.user_id)
            } else {
                session_service.export_session_by_id(&session.id)
            };
            match export_result {
                Ok(path) => app.set_info_message(format!("Session exported to {}", path)),
                Err(e) => app.set_error_message(format!("Export failed: {}", e)),
            }
            app.state = AppState::Menu(0);
        }
        AppState::Settings(selected) => match *selected {
            0 => app.state = AppState::Profile(0),
            1 => app.state = AppState::ExecutionPolicy(0),
            2 => handle_web_search(app),
            3 => handle_system_info(app),
            4 => handle_process_list(app),
            _ => {}
        },
        AppState::Profile(selected) => match *selected {
            0 if app.auth_session.is_some() => {
                if let Err(err) = crate::interfaces::ui::auth::clear_auth_session() {
                    app.set_error_message(format!("Auth logout failed: {}", err));
                } else {
                    app.auth_session = None;
                    app.auth_flow_id = None;
                    app.auth_last_poll_at = None;
                    app.set_status_message("Cleared local TUI auth session".to_string());
                    app.state = AppState::Profile(0);
                }
            }
            1 if app.auth_session.is_some() => return EventResult::RefreshAuthSession,
            0 => {
                return EventResult::StartProfileLogin {
                    provider: "github".to_string(),
                }
            }
            1 => {
                return EventResult::StartProfileLogin {
                    provider: "google".to_string(),
                }
            }
            2 => {
                return EventResult::StartProfileLogin {
                    provider: "apple".to_string(),
                }
            }
            _ => {}
        },
        AppState::ExecutionPolicy(selected) => match *selected {
            0 => {
                app.approval_profile = settings::next_approval_profile(app.approval_profile);
            }
            1 => {
                app.execution_strategy = settings::next_execution_strategy(app.execution_strategy);
            }
            2 => {
                app.sandbox_profile = settings::next_sandbox_profile(app.sandbox_profile);
            }
            3 => {
                app.retry_max_attempts = settings::next_retry_max_attempts(app.retry_max_attempts);
            }
            4 => start_execution_policy_editor(app, ExecutionPolicyListField::HeaderWidgets),
            5 => start_execution_policy_editor(app, ExecutionPolicyListField::AllowedCommands),
            6 => start_execution_policy_editor(app, ExecutionPolicyListField::BlockedCommands),
            7 => return EventResult::SaveExecutionPolicy,
            _ => {}
        },
        AppState::ViewSession(session_id, _, _) => {
            match session_service.load_session_state_view(session_id) {
                Ok(session_view) => {
                    app.state = AppState::Chat(Box::new(create_chat_state(
                        session_view.session_id,
                        session_view.messages,
                        session_view.plan,
                        session_view.agents,
                        app.agents_context_enabled,
                    )));
                    return EventResult::GatherSidebarEntries;
                }
                Err(e) => app.set_error_message(format!("Error loading session: {}", e)),
            }
        }
        _ => {}
    }
    EventResult::Continue
}

fn preview_selected_session(app: &mut TuiApp, session_service: &SessionService) -> EventResult {
    let AppState::Sessions(sessions, selected) = &app.state else {
        return EventResult::Continue;
    };
    if sessions.is_empty() || *selected >= sessions.len() {
        return EventResult::Continue;
    }
    let session = &sessions[*selected];
    if app.auth_session.is_some() && app.auth_server_base_url.is_some() {
        EventResult::OpenSession {
            session_id: session.id.clone(),
            preview: true,
        }
    } else {
        match session_service.view_session_data(&session.id) {
            Ok(messages) => {
                app.state = AppState::ViewSession(session.id.clone(), messages, 0);
                EventResult::Continue
            }
            Err(e) => {
                app.set_error_message(format!("Error loading session preview: {}", e));
                EventResult::Continue
            }
        }
    }
}

fn delete_selected_session(app: &mut TuiApp) -> EventResult {
    match &app.state {
        AppState::Sessions(sessions, selected)
            if !sessions.is_empty() && *selected < sessions.len() =>
        {
            let session = &sessions[*selected];
            EventResult::DeleteSession {
                session_id: session.id.clone(),
                remote: app.auth_session.is_some() && app.auth_server_base_url.is_some(),
                export_view: false,
                selected_index: *selected,
            }
        }
        AppState::ExportSessions(sessions, selected)
            if !sessions.is_empty() && *selected < sessions.len() =>
        {
            let session = &sessions[*selected];
            EventResult::DeleteSession {
                session_id: session.id.clone(),
                remote: false,
                export_view: true,
                selected_index: *selected,
            }
        }
        _ => EventResult::Continue,
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
        refresh_chat_completions(chat_state);
    }
}

fn handle_backspace(app: &mut TuiApp) {
    if let AppState::Chat(chat_state) = &mut app.state {
        chat_state.input.pop();
        refresh_chat_completions(chat_state);
    }
}
fn handle_paste(app: &mut TuiApp, content: String) {
    if let AppState::Chat(chat_state) = &mut app.state {
        chat_state.input.push_str(&content);
        refresh_chat_completions(chat_state);
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
                refresh_chat_completions(chat_state);
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

fn slash_command_candidates(input: &str) -> Vec<String> {
    let commands = vec![
        "/help",
        "/quit",
        "/clear",
        "/exit",
        "/audit",
        "/agents",
        "/agents on",
        "/agents off",
        "/agents status",
        "/strategy",
        "/strategy auto",
        "/strategy grounded",
        "/strategy deterministic",
        "/strategy model",
        "/auth login github",
        "/auth login google",
        "/auth login apple",
        "/auth logout",
        "/auth status",
    ];
    let mut matches = commands
        .into_iter()
        .filter(|cmd| cmd.starts_with(input))
        .map(ToOwned::to_owned)
        .collect::<Vec<_>>();
    matches.sort();
    matches
}

fn refresh_chat_completions(chat_state: &mut ChatState) {
    if chat_state.input.starts_with('/') {
        chat_state.completion_candidates = slash_command_candidates(&chat_state.input);
        chat_state.completion_index = 0;
        chat_state.completion_prefix = Some(chat_state.input.clone());
    } else {
        chat_state.reset_completion();
    }
}

fn current_completion_index(chat_state: &ChatState) -> Option<usize> {
    chat_state
        .completion_candidates
        .iter()
        .position(|candidate| candidate == &chat_state.input)
}

fn select_completion_candidate(chat_state: &mut ChatState, index: usize) {
    if let Some(candidate) = chat_state.completion_candidates.get(index) {
        chat_state.input = candidate.clone();
        chat_state.completion_index = index;
    }
}

fn handle_completion_down(app: &mut TuiApp) -> bool {
    let AppState::Chat(chat_state) = &mut app.state else {
        return false;
    };
    if !chat_state.input.starts_with('/') || chat_state.completion_candidates.is_empty() {
        return false;
    }

    let next_index = current_completion_index(chat_state)
        .map(|index| (index + 1) % chat_state.completion_candidates.len())
        .unwrap_or(0);
    select_completion_candidate(chat_state, next_index);
    true
}

fn handle_completion_up(app: &mut TuiApp) -> bool {
    let AppState::Chat(chat_state) = &mut app.state else {
        return false;
    };
    if !chat_state.input.starts_with('/') || chat_state.completion_candidates.is_empty() {
        return false;
    }

    let last_index = chat_state.completion_candidates.len().saturating_sub(1);
    let prev_index = current_completion_index(chat_state)
        .map(|index| if index == 0 { last_index } else { index - 1 })
        .unwrap_or(last_index);
    select_completion_candidate(chat_state, prev_index);
    true
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
                let next_index = current_completion_index(chat_state)
                    .map(|index| (index + 1) % chat_state.completion_candidates.len())
                    .unwrap_or(0);
                select_completion_candidate(chat_state, next_index);
            }
        } else if chat_state.input.starts_with('/') {
            if chat_state.completion_candidates.is_empty() {
                chat_state.completion_candidates = slash_command_candidates(&chat_state.input);
                chat_state.completion_index = 0;
            }
            if !chat_state.completion_candidates.is_empty() {
                let next_index = current_completion_index(chat_state)
                    .map(|index| (index + 1) % chat_state.completion_candidates.len())
                    .unwrap_or(0);
                select_completion_candidate(chat_state, next_index);
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
        assert!(matches!(result, EventResult::GatherSidebarEntries));
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
        assert!(matches!(result, EventResult::LoadSessions));
        assert!(matches!(app.state, AppState::Menu(1)));
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

    #[test]
    fn test_enter_settings_profile() {
        let mut app = TuiApp::new();
        app.state = AppState::Settings(0); // Profile
        let conn = rusqlite::Connection::open_in_memory().unwrap();
        harper_core::memory::storage::init_db(&conn).unwrap();
        let session_service = SessionService::new(&conn);

        let result = handle_event(
            Event::Key(KeyCode::Enter.into()),
            &mut app,
            &session_service,
        );
        assert!(matches!(result, EventResult::Continue));
        assert!(matches!(app.state, AppState::Profile(0)));
    }

    #[test]
    fn test_enter_settings_execution_policy() {
        let mut app = TuiApp::new();
        app.state = AppState::Settings(1);
        let conn = rusqlite::Connection::open_in_memory().unwrap();
        harper_core::memory::storage::init_db(&conn).unwrap();
        let session_service = SessionService::new(&conn);

        let result = handle_event(
            Event::Key(KeyCode::Enter.into()),
            &mut app,
            &session_service,
        );
        assert!(matches!(result, EventResult::Continue));
        assert!(matches!(app.state, AppState::ExecutionPolicy(0)));
    }

    #[test]
    fn test_enter_profile_refresh_returns_async_event() {
        let mut app = TuiApp::new();
        app.auth_session = Some(harper_core::AuthSession {
            access_token: "access".to_string(),
            refresh_token: Some("refresh".to_string()),
            expires_at: None,
            user: harper_core::AuthenticatedUser {
                user_id: "user-1".to_string(),
                email: Some("user@example.com".to_string()),
                display_name: Some("Example User".to_string()),
                provider: Some(harper_core::UserAuthProvider::Github),
            },
        });
        app.state = AppState::Profile(1); // Refresh Session
        let conn = rusqlite::Connection::open_in_memory().unwrap();
        harper_core::memory::storage::init_db(&conn).unwrap();
        let session_service = SessionService::new(&conn);

        let result = handle_event(
            Event::Key(KeyCode::Enter.into()),
            &mut app,
            &session_service,
        );
        assert!(matches!(result, EventResult::RefreshAuthSession));
    }

    #[test]
    fn test_enter_profile_login_returns_async_event() {
        let mut app = TuiApp::new();
        app.state = AppState::Profile(0); // Login with GitHub
        let conn = rusqlite::Connection::open_in_memory().unwrap();
        harper_core::memory::storage::init_db(&conn).unwrap();
        let session_service = SessionService::new(&conn);

        let result = handle_event(
            Event::Key(KeyCode::Enter.into()),
            &mut app,
            &session_service,
        );
        assert!(matches!(
            result,
            EventResult::StartProfileLogin { provider } if provider == "github"
        ));
    }

    #[test]
    fn test_enter_execution_policy_save_returns_async_event() {
        let mut app = TuiApp::new();
        app.state = AppState::ExecutionPolicy(7);
        let conn = rusqlite::Connection::open_in_memory().unwrap();
        harper_core::memory::storage::init_db(&conn).unwrap();
        let session_service = SessionService::new(&conn);

        let result = handle_event(
            Event::Key(KeyCode::Enter.into()),
            &mut app,
            &session_service,
        );
        assert!(matches!(result, EventResult::SaveExecutionPolicy));
    }

    #[test]
    fn test_enter_execution_policy_allowed_commands_opens_editor() {
        let mut app = TuiApp::new();
        app.allowed_commands = vec!["git".to_string()];
        app.state = AppState::ExecutionPolicy(5);
        let conn = rusqlite::Connection::open_in_memory().unwrap();
        harper_core::memory::storage::init_db(&conn).unwrap();
        let session_service = SessionService::new(&conn);

        let result = handle_event(
            Event::Key(KeyCode::Enter.into()),
            &mut app,
            &session_service,
        );
        assert!(matches!(result, EventResult::Continue));
        assert!(app.execution_policy_editor.is_some());
    }

    #[test]
    fn test_execution_policy_editor_commits_command_list() {
        let mut app = TuiApp::new();
        app.state = AppState::ExecutionPolicy(3);
        app.execution_policy_editor = Some(super::ExecutionPolicyEditorState {
            field: super::ExecutionPolicyListField::AllowedCommands,
            input: "git, ls, cargo".to_string(),
            selected_index: 0,
            text_input_focused: false,
        });
        let conn = rusqlite::Connection::open_in_memory().unwrap();
        harper_core::memory::storage::init_db(&conn).unwrap();
        let session_service = SessionService::new(&conn);

        let result = handle_event(
            Event::Key(KeyCode::Enter.into()),
            &mut app,
            &session_service,
        );
        assert!(matches!(result, EventResult::Continue));
        assert_eq!(app.allowed_commands, vec!["git", "ls", "cargo"]);
        assert!(app.execution_policy_editor.is_none());
    }

    #[test]
    fn test_enter_execution_policy_retry_attempts_cycles_value() {
        let mut app = TuiApp::new();
        app.retry_max_attempts = 1;
        app.state = AppState::ExecutionPolicy(3);
        let conn = rusqlite::Connection::open_in_memory().unwrap();
        harper_core::memory::storage::init_db(&conn).unwrap();
        let session_service = SessionService::new(&conn);

        let result = handle_event(
            Event::Key(KeyCode::Enter.into()),
            &mut app,
            &session_service,
        );
        assert!(matches!(result, EventResult::Continue));
        assert_eq!(app.retry_max_attempts, 2);
    }

    #[test]
    fn ctrl_s_opens_plan_steps_browser() {
        let mut app = TuiApp::new();
        app.state = AppState::Chat(Box::new(create_chat_state(
            "session".to_string(),
            vec![],
            Some(harper_core::PlanState {
                explanation: None,
                items: vec![harper_core::PlanItem {
                    step: "First".to_string(),
                    status: harper_core::PlanStepStatus::Pending,
                    job_id: None,
                }],
                runtime: None,
                updated_at: None,
            }),
            None,
            app.agents_context_enabled,
        )));
        let conn = rusqlite::Connection::open_in_memory().unwrap();
        harper_core::memory::storage::init_db(&conn).unwrap();
        let session_service = SessionService::new(&conn);

        let result = handle_event(
            Event::Key(KeyEvent::new(KeyCode::Char('s'), KeyModifiers::CONTROL)),
            &mut app,
            &session_service,
        );
        assert!(matches!(result, EventResult::Continue));
        let AppState::Chat(chat_state) = &app.state else {
            panic!("expected chat state");
        };
        assert!(chat_state.plan_steps_expanded);
        assert!(matches!(
            chat_state.navigation_focus,
            NavigationFocus::PlanSteps
        ));
    }

    #[test]
    fn plan_steps_focus_complete_shortcut_returns_status_update() {
        let mut app = TuiApp::new();
        let mut chat_state = create_chat_state(
            "session".to_string(),
            vec![],
            Some(harper_core::PlanState {
                explanation: None,
                items: vec![harper_core::PlanItem {
                    step: "First".to_string(),
                    status: harper_core::PlanStepStatus::Pending,
                    job_id: None,
                }],
                runtime: None,
                updated_at: None,
            }),
            None,
            true,
        );
        chat_state.plan_steps_expanded = true;
        chat_state.set_navigation_focus(NavigationFocus::PlanSteps);
        app.state = AppState::Chat(Box::new(chat_state));
        let conn = rusqlite::Connection::open_in_memory().unwrap();
        harper_core::memory::storage::init_db(&conn).unwrap();
        let session_service = SessionService::new(&conn);

        let result = handle_event(
            Event::Key(KeyCode::Char('c').into()),
            &mut app,
            &session_service,
        );
        assert!(matches!(
            result,
            EventResult::SetPlanStepStatus {
                step_index: 0,
                status: harper_core::PlanStepStatus::Completed,
                ..
            }
        ));
    }

    #[test]
    fn plan_steps_focus_retry_shortcut_returns_retry_event() {
        let mut app = TuiApp::new();
        let mut runtime = harper_core::core::plan::PlanRuntime::default();
        runtime
            .set_retry_or_replan_followup("Retry failing command", Some("cargo test".to_string()));
        let mut chat_state = create_chat_state(
            "session".to_string(),
            vec![],
            Some(harper_core::PlanState {
                explanation: None,
                items: vec![harper_core::PlanItem {
                    step: "Retry failing command".to_string(),
                    status: harper_core::PlanStepStatus::Blocked,
                    job_id: None,
                }],
                runtime: Some(runtime),
                updated_at: None,
            }),
            None,
            true,
        );
        chat_state.plan_steps_expanded = true;
        chat_state.set_navigation_focus(NavigationFocus::PlanSteps);
        app.state = AppState::Chat(Box::new(chat_state));
        let conn = rusqlite::Connection::open_in_memory().unwrap();
        harper_core::memory::storage::init_db(&conn).unwrap();
        let session_service = SessionService::new(&conn);

        let result = handle_event(
            Event::Key(KeyCode::Char('r').into()),
            &mut app,
            &session_service,
        );
        assert!(matches!(
            result,
            EventResult::RetryPlanFollowup {
                command,
                ..
            } if command == "cargo test"
        ));
    }

    #[test]
    fn plan_steps_focus_ack_shortcut_returns_followup_clear() {
        let mut app = TuiApp::new();
        let mut runtime = harper_core::core::plan::PlanRuntime::default();
        runtime.set_checkpoint_followup("Inspect output", Some("Patch".to_string()));
        let mut chat_state = create_chat_state(
            "session".to_string(),
            vec![],
            Some(harper_core::PlanState {
                explanation: None,
                items: vec![harper_core::PlanItem {
                    step: "Inspect output".to_string(),
                    status: harper_core::PlanStepStatus::InProgress,
                    job_id: None,
                }],
                runtime: Some(runtime),
                updated_at: None,
            }),
            None,
            true,
        );
        chat_state.plan_steps_expanded = true;
        chat_state.set_navigation_focus(NavigationFocus::PlanSteps);
        app.state = AppState::Chat(Box::new(chat_state));
        let conn = rusqlite::Connection::open_in_memory().unwrap();
        harper_core::memory::storage::init_db(&conn).unwrap();
        let session_service = SessionService::new(&conn);

        let result = handle_event(
            Event::Key(KeyCode::Char('k').into()),
            &mut app,
            &session_service,
        );
        assert!(matches!(result, EventResult::ClearPlanFollowup { .. }));
    }

    #[test]
    fn plan_steps_focus_replan_shortcut_returns_structured_event() {
        let mut app = TuiApp::new();
        let mut runtime = harper_core::core::plan::PlanRuntime::default();
        runtime
            .set_retry_or_replan_followup("Retry failing command", Some("cargo test".to_string()));
        let mut chat_state = create_chat_state(
            "session".to_string(),
            vec![],
            Some(harper_core::PlanState {
                explanation: None,
                items: vec![harper_core::PlanItem {
                    step: "Retry failing command".to_string(),
                    status: harper_core::PlanStepStatus::Blocked,
                    job_id: None,
                }],
                runtime: Some(runtime),
                updated_at: None,
            }),
            None,
            true,
        );
        chat_state.plan_steps_expanded = true;
        chat_state.set_navigation_focus(NavigationFocus::PlanSteps);
        app.state = AppState::Chat(Box::new(chat_state));
        let conn = rusqlite::Connection::open_in_memory().unwrap();
        harper_core::memory::storage::init_db(&conn).unwrap();
        let session_service = SessionService::new(&conn);

        let result = handle_event(
            Event::Key(KeyCode::Char('u').into()),
            &mut app,
            &session_service,
        );
        assert!(matches!(
            result,
            EventResult::RequestPlanReplan { step_index: 0, step, .. } if step == "Retry failing command"
        ));
    }

    #[test]
    fn chat_mode_treats_l_as_text_input() {
        let mut app = TuiApp::new();
        app.state = AppState::Chat(Box::new(create_chat_state(
            "session".to_string(),
            vec![],
            None,
            None,
            app.agents_context_enabled,
        )));
        let conn = rusqlite::Connection::open_in_memory().unwrap();
        harper_core::memory::storage::init_db(&conn).unwrap();
        let session_service = SessionService::new(&conn);

        let result = handle_event(
            Event::Key(KeyCode::Char('l').into()),
            &mut app,
            &session_service,
        );
        assert!(matches!(result, EventResult::Continue));
        match &app.state {
            AppState::Chat(chat_state) => assert_eq!(chat_state.input, "l"),
            _ => panic!("expected chat state"),
        }
    }

    #[test]
    fn test_enter_menu_load_sessions_returns_async_event() {
        let mut app = TuiApp::new();
        app.state = AppState::Menu(1); // Sessions
        let conn = rusqlite::Connection::open_in_memory().unwrap();
        harper_core::memory::storage::init_db(&conn).unwrap();
        let session_service = SessionService::new(&conn);

        let result = handle_event(
            Event::Key(KeyCode::Enter.into()),
            &mut app,
            &session_service,
        );
        assert!(matches!(result, EventResult::LoadSessions));
        assert!(matches!(app.state, AppState::Menu(1)));
    }

    #[test]
    fn delete_key_on_sessions_returns_delete_event() {
        let mut app = TuiApp::new();
        app.state = AppState::Sessions(
            vec![SessionInfo {
                id: "session-1".to_string(),
                name: "Example".to_string(),
                created_at: "2026-04-28".to_string(),
            }],
            0,
        );
        let conn = rusqlite::Connection::open_in_memory().unwrap();
        harper_core::memory::storage::init_db(&conn).unwrap();
        let session_service = SessionService::new(&conn);

        let result = handle_event(
            Event::Key(KeyCode::Delete.into()),
            &mut app,
            &session_service,
        );
        assert!(matches!(
            result,
            EventResult::DeleteSession {
                session_id,
                remote: false,
                export_view: false,
                ..
            } if session_id == "session-1"
        ));
    }

    #[test]
    fn delete_key_on_export_sessions_returns_delete_event() {
        let mut app = TuiApp::new();
        app.state = AppState::ExportSessions(
            vec![SessionInfo {
                id: "session-1".to_string(),
                name: "Example".to_string(),
                created_at: "2026-04-28".to_string(),
            }],
            0,
        );
        let conn = rusqlite::Connection::open_in_memory().unwrap();
        harper_core::memory::storage::init_db(&conn).unwrap();
        let session_service = SessionService::new(&conn);

        let result = handle_event(
            Event::Key(KeyCode::Delete.into()),
            &mut app,
            &session_service,
        );
        assert!(matches!(
            result,
            EventResult::DeleteSession {
                session_id,
                remote: false,
                export_view: true,
                ..
            } if session_id == "session-1"
        ));
    }

    #[test]
    fn slash_completion_down_selects_first_candidate() {
        let mut app = TuiApp::new();
        let mut chat_state = create_chat_state("session".to_string(), vec![], None, None, true);
        chat_state.input = "/".to_string();
        refresh_chat_completions(&mut chat_state);
        app.state = AppState::Chat(Box::new(chat_state));

        assert!(handle_completion_down(&mut app));

        let AppState::Chat(chat_state) = &app.state else {
            panic!("expected chat state");
        };
        assert_eq!(chat_state.input, "/agents");
    }

    #[test]
    fn slash_completion_up_wraps_to_last_candidate() {
        let mut app = TuiApp::new();
        let mut chat_state = create_chat_state("session".to_string(), vec![], None, None, true);
        chat_state.input = "/".to_string();
        refresh_chat_completions(&mut chat_state);
        app.state = AppState::Chat(Box::new(chat_state));

        assert!(handle_completion_up(&mut app));

        let AppState::Chat(chat_state) = &app.state else {
            panic!("expected chat state");
        };
        assert_eq!(chat_state.input, "/strategy model");
    }
}
