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

use ratatui::prelude::*;
use ratatui::style::Modifier;
use ratatui::widgets::{Block, Borders, Clear, List, ListItem, Padding, Paragraph, Wrap};

use super::app::{
    AppState, ApprovalState, CommandOutputState, ReviewState, SessionInfo, TuiApp, UiMessage,
};
use super::theme::Theme;
use harper_core::core::plan::{PlanJobRecord, PlanJobStatus};
use harper_core::{PlanRuntime, PlanState, PlanStepStatus, ResolvedAgents};

// Refined shortcuts for a cleaner footer
const FOOTER_SHORTCUTS: [[(&str, &str); 9]; 2] = [
    [
        ("G", "Help"),
        ("W", "Search"),
        ("B", "Sidebar"),
        ("A", "Agents"),
        ("F", "Findings"),
        ("M", "Msgs"),
        ("C", "ID"),
        ("O", "Export"),
        ("D", "Delete"),
    ],
    [
        ("X", "Exit"),
        ("R", "Load"),
        ("L", "Preview"),
        ("U", "Paste"),
        ("T", "Enter"),
        ("Y", "Prev"),
        ("V", "Next"),
        ("Esc", "Back"),
        ("P", "Jobs"),
    ],
];

use crate::plugins::syntax::highlight_code;
use syntect::highlighting::ThemeSet;
use syntect::parsing::SyntaxSet;

pub fn parse_content_with_code<'a>(
    syntax_set: &SyntaxSet,
    theme_set: &ThemeSet,
    content: &'a str,
    default_color: Color,
    syntax_theme: &str,
) -> Vec<Span<'a>> {
    let mut spans = Vec::new();
    let mut remaining = content;

    while let Some(start) = remaining.find("```") {
        if start > 0 {
            spans.push(Span::styled(
                &remaining[..start],
                Style::default().fg(default_color),
            ));
        }

        let after_start = &remaining[start + 3..];
        if let Some(end) = after_start.find("```") {
            let code_block = &after_start[..end];
            if let Some(newline_pos) = code_block.find('\n') {
                let language = &code_block[..newline_pos].trim();
                let code = &code_block[newline_pos + 1..];
                spans.extend(highlight_code(
                    syntax_set,
                    theme_set,
                    language,
                    code,
                    syntax_theme,
                ));
            } else {
                spans.push(Span::styled(code_block, Style::default().fg(default_color)));
            }
            remaining = &after_start[end + 3..];
        } else {
            spans.push(Span::styled(
                &remaining[start..],
                Style::default().fg(default_color),
            ));
            remaining = "";
            break;
        }
    }

    if !remaining.is_empty() {
        spans.push(Span::styled(remaining, Style::default().fg(default_color)));
    }

    spans
}

pub fn draw(frame: &mut Frame, app: &TuiApp, theme: &Theme) {
    let area = frame.area();
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Min(0),
            Constraint::Length(2), // Slimmer footer
        ])
        .split(area);

    let main_area = chunks[0];
    let footer_area = chunks[1];

    match &app.state {
        AppState::Menu(selected) => draw_zen_menu(frame, *selected, theme, main_area),
        AppState::Chat(chat_state) => {
            let chat_area = if chat_state.sidebar_visible {
                let chunks = Layout::default()
                    .direction(Direction::Horizontal)
                    .constraints([
                        Constraint::Percentage(20), // Slimmer sidebar
                        Constraint::Percentage(80),
                    ])
                    .split(main_area);

                draw_zen_sidebar(frame, &chat_state.sidebar_entries, theme, chunks[0]);
                chunks[1]
            } else {
                main_area
            };

            let has_plan = chat_state
                .active_plan
                .as_ref()
                .is_some_and(|plan| !plan.items.is_empty() || plan.explanation.is_some());
            let has_agents = chat_state.agents_panel_expanded
                || chat_state
                    .active_agents
                    .as_ref()
                    .is_some_and(|agents| !agents.sources.is_empty());
            let has_review = chat_state.active_review.is_some();
            let has_command_output = chat_state.command_output.is_some();
            let plan_height = chat_state
                .active_plan
                .as_ref()
                .map(plan_panel_height)
                .unwrap_or(0);
            let command_output_height = chat_state
                .command_output
                .as_ref()
                .map(command_output_panel_height)
                .unwrap_or(0);
            let agents_height = chat_state
                .active_agents
                .as_ref()
                .map(|agents| agents_panel_height(agents, chat_state.agents_panel_expanded))
                .unwrap_or(6);
            let review_height = chat_state
                .active_review
                .as_ref()
                .map(|review| review_panel_height(review, chat_state.review_selected))
                .unwrap_or(0);
            let mut constraints = vec![Constraint::Length(3), Constraint::Min(5)];
            if has_review {
                constraints.push(Constraint::Length(review_height));
            }
            if has_command_output {
                constraints.push(Constraint::Length(command_output_height));
            }
            if has_plan {
                constraints.push(Constraint::Length(plan_height));
            }
            if has_agents {
                constraints.push(Constraint::Length(agents_height));
            }
            constraints.push(Constraint::Length(3));
            let chunks = Layout::default()
                .direction(Direction::Vertical)
                .constraints(constraints)
                .split(chat_area);

            draw_chat_summary(frame, app, chat_state, theme, chunks[0]);

            // Messages area - Typography focused
            let safe_scroll_offset = chat_state.scroll_offset.min(chat_state.messages.len());
            let displayed_messages = &chat_state.messages[safe_scroll_offset..];
            let mut message_lines: Vec<Line> = Vec::new();

            for msg in displayed_messages.iter().filter(|msg| msg.role != "system") {
                let label = match msg.role.as_str() {
                    "user" => "User ›",
                    "assistant" => "Harper ›",
                    _ => "System ›",
                };

                let label_style = match msg.role.as_str() {
                    "user" => Style::default()
                        .fg(theme.accent)
                        .add_modifier(Modifier::BOLD),
                    "assistant" => Style::default()
                        .fg(theme.title)
                        .add_modifier(Modifier::BOLD),
                    _ => theme.muted_style(),
                };

                message_lines.push(Line::from(vec![Span::styled(label, label_style)]));

                let content_color = if msg.role == "user" {
                    theme.foreground
                } else {
                    theme.output
                };

                if msg.content.contains("```") {
                    let spans = parse_content_with_code(
                        &theme.syntax_set,
                        &theme.theme_set,
                        &msg.content,
                        content_color,
                        &theme.syntax_theme,
                    );
                    message_lines.push(Line::from(spans));
                } else {
                    for line in msg.content.lines() {
                        message_lines.push(Line::styled(line, content_color));
                    }
                }
                message_lines.push(Line::raw("")); // Breathing room
            }

            if chat_state.awaiting_response {
                message_lines.push(Line::from(vec![Span::styled(
                    "Harper ›",
                    Style::default()
                        .fg(theme.title)
                        .add_modifier(Modifier::BOLD),
                )]));
                message_lines.push(Line::from(vec![
                    Span::styled(
                        activity_spinner_frame(app),
                        Style::default().fg(theme.accent),
                    ),
                    Span::raw(" "),
                    Span::styled(
                        "Thinking...",
                        theme.muted_style().add_modifier(Modifier::ITALIC),
                    ),
                ]));
                message_lines.push(Line::raw(""));
            }

            let chat_block = Block::default()
                .borders(Borders::NONE) // No noise
                .padding(Padding::uniform(1));

            let messages_widget = Paragraph::new(message_lines)
                .block(chat_block)
                .wrap(Wrap { trim: false });

            frame.render_widget(messages_widget, chunks[1]);

            let mut next_panel_index = 2;
            if has_review {
                if let Some(review) = &chat_state.active_review {
                    draw_review_panel(
                        frame,
                        review,
                        chat_state.review_selected,
                        matches!(
                            chat_state.navigation_focus,
                            super::app::NavigationFocus::Review
                        ),
                        theme,
                        chunks[next_panel_index],
                    );
                }
                next_panel_index += 1;
            }
            if has_command_output {
                if let Some(output) = &chat_state.command_output {
                    draw_command_output_panel(frame, output, theme, chunks[next_panel_index]);
                }
                next_panel_index += 1;
            }

            if has_plan {
                if let Some(plan) = &chat_state.active_plan {
                    draw_plan_panel(
                        frame,
                        plan,
                        chat_state.plan_job_selected,
                        matches!(
                            chat_state.navigation_focus,
                            super::app::NavigationFocus::PlanJobs
                        ),
                        theme,
                        chunks[next_panel_index],
                    );
                }
                next_panel_index += 1;
            }
            if has_agents {
                draw_agents_panel(
                    frame,
                    chat_state.active_agents.as_ref(),
                    theme,
                    chunks[next_panel_index],
                    chat_state.agents_panel_expanded,
                    chat_state.agents_scroll_offset,
                    matches!(
                        chat_state.navigation_focus,
                        super::app::NavigationFocus::Agents
                    ),
                );
            }

            // Input area - Minimalist
            let input_block = Block::default()
                .borders(Borders::TOP)
                .border_style(theme.muted_style())
                .padding(Padding::horizontal(1));

            let mut input_text = vec![Line::from(vec![
                Span::styled("› ", Style::default().fg(theme.accent)),
                Span::styled(&chat_state.input, Style::default().fg(theme.input)),
            ])];

            if chat_state.input.trim().is_empty() {
                input_text.push(Line::from(Span::styled(
                    "Type a message... (Ctrl+G for help)",
                    theme.muted_style().add_modifier(Modifier::ITALIC),
                )));
            }

            let input_widget = Paragraph::new(input_text).block(input_block);

            let input_index = 2
                + usize::from(has_review)
                + usize::from(has_command_output)
                + usize::from(has_plan)
                + usize::from(has_agents);
            frame.render_widget(input_widget, chunks[input_index]);
        }
        AppState::Sessions(sessions, selected) => draw_sessions(
            frame,
            sessions,
            *selected,
            app.auth_session.is_some() && app.auth_server_base_url.is_some(),
            theme,
            main_area,
        ),
        AppState::ExportSessions(sessions, selected) => {
            draw_export_sessions(frame, sessions, *selected, theme, main_area)
        }
        AppState::Tools(selected) => draw_tools(frame, *selected, theme, main_area),
        AppState::Profile(selected) => draw_profile(frame, app, *selected, theme, main_area),
        AppState::ViewSession(name, messages, selected) => {
            draw_view_session(frame, name, messages, *selected, theme, main_area)
        }
        AppState::Stats(stats) => draw_stats(frame, stats, theme, main_area),
    }

    draw_zen_footer(frame, app, theme, footer_area);

    if let Some(approval) = &app.pending_approval {
        draw_approval(frame, approval, theme);
    }

    if let AppState::Chat(chat_state) = &app.state {
        if chat_state.plan_jobs_expanded {
            draw_plan_jobs_browser(frame, chat_state, theme);
        }
    }

    if let Some(msg) = &app.message {
        draw_message_overlay(frame, msg, theme);
    }
}

fn review_panel_height(review: &ReviewState, selected: usize) -> u16 {
    let findings = review.findings.len().min(3);
    let model_line = usize::from(review.model.is_some());
    let detail_lines = review
        .findings
        .get(selected)
        .map(|finding| 2 + finding.message.lines().count().min(3))
        .unwrap_or(0);
    (findings + model_line + detail_lines + 4) as u16
}

fn draw_chat_summary(
    frame: &mut Frame,
    app: &TuiApp,
    chat_state: &super::app::ChatState,
    theme: &Theme,
    area: Rect,
) {
    let plan_status = chat_state
        .active_plan
        .as_ref()
        .map_or("plan: none".to_string(), |plan| {
            let total = plan.items.len();
            let completed = plan
                .items
                .iter()
                .filter(|item| matches!(item.status, PlanStepStatus::Completed))
                .count();
            let blocked = plan
                .items
                .iter()
                .filter(|item| matches!(item.status, PlanStepStatus::Blocked))
                .count();
            let in_progress = plan
                .items
                .iter()
                .any(|item| matches!(item.status, PlanStepStatus::InProgress));
            if total == 0 {
                "plan: empty".to_string()
            } else if blocked > 0 {
                format!("plan: {}/{} done, {} blocked", completed, total, blocked)
            } else if in_progress {
                format!("plan: {}/{} done, active", completed, total)
            } else {
                format!("plan: {}/{} done", completed, total)
            }
        });
    let agents_status = chat_state
        .active_agents
        .as_ref()
        .map_or("agents: none".to_string(), |agents| {
            format!("agents: {} sections", agents.effective_rule_sections.len())
        });
    let web_status = if chat_state.web_search_enabled {
        "web: on"
    } else {
        "web: off"
    };
    let auth_status = app.auth_status_label();
    let focus_status = format!("focus: {}", chat_state.navigation_focus_label());
    let model_status = if app.model_label.is_empty() {
        None
    } else {
        Some(format!("model: {}", app.model_label))
    };
    let approval_status = if app.pending_approval.is_some() {
        Some("approval: pending")
    } else {
        None
    };
    let activity_status = app.activity_status.as_ref().map(|status| {
        let spinner = activity_spinner_frame(app);
        format!(
            "activity: {} {}",
            spinner,
            truncate_chat_summary(status, 32)
        )
    });
    let agents_panel_status = if chat_state.agents_panel_expanded {
        Some("agents panel: open")
    } else {
        None
    };
    let last_action = latest_action_summary(app, chat_state);
    let last_rule_source = latest_rule_source(chat_state);

    let mut spans = vec![
        Span::styled("session ", theme.muted_style()),
        Span::styled(
            truncate_chat_summary(&chat_state.session_id, 12),
            Style::default()
                .fg(theme.foreground)
                .add_modifier(Modifier::BOLD),
        ),
        Span::raw("  "),
        Span::styled(plan_status, theme.muted_style()),
        Span::raw("  "),
        Span::styled(agents_status, theme.muted_style()),
        Span::raw("  "),
        Span::styled(web_status, theme.muted_style()),
        Span::raw("  "),
        Span::styled(auth_status, theme.muted_style()),
        Span::raw("  "),
        Span::styled(focus_status, theme.muted_style()),
    ];
    if let Some(status) = approval_status {
        spans.push(Span::raw("  "));
        spans.push(Span::styled(
            status,
            Style::default()
                .fg(theme.accent)
                .add_modifier(Modifier::BOLD),
        ));
    }
    if let Some(status) = activity_status {
        spans.push(Span::raw("  "));
        spans.push(Span::styled(
            status,
            Style::default()
                .fg(theme.output)
                .add_modifier(Modifier::BOLD),
        ));
    }
    if let Some(status) = model_status {
        spans.push(Span::raw("  "));
        spans.push(Span::styled(status, theme.muted_style()));
    }
    if let Some(status) = agents_panel_status {
        spans.push(Span::raw("  "));
        spans.push(Span::styled(status, theme.muted_style()));
    }
    let line = Line::from(spans);
    let mut detail_spans = Vec::new();
    if let Some(action) = last_action {
        detail_spans.push(Span::styled("last ", theme.muted_style()));
        detail_spans.push(Span::styled(
            truncate_chat_summary(&action, 36),
            Style::default().fg(theme.foreground),
        ));
    }
    if let Some(source) = last_rule_source {
        if !detail_spans.is_empty() {
            detail_spans.push(Span::raw("  "));
        }
        detail_spans.push(Span::styled("rule src ", theme.muted_style()));
        detail_spans.push(Span::styled(
            truncate_chat_summary(&source, 24),
            theme.muted_style().add_modifier(Modifier::ITALIC),
        ));
    }

    let block = Block::default()
        .borders(Borders::BOTTOM)
        .border_style(theme.muted_style())
        .padding(Padding::horizontal(1));
    let mut lines = vec![line];
    if !detail_spans.is_empty() {
        lines.push(Line::from(detail_spans));
    }
    let widget = Paragraph::new(lines).block(block).wrap(Wrap { trim: true });
    frame.render_widget(widget, area);
}

fn truncate_chat_summary(value: &str, max_len: usize) -> String {
    if value.len() <= max_len {
        value.to_string()
    } else {
        format!("{}...", &value[..max_len])
    }
}

fn activity_spinner_frame(app: &TuiApp) -> &'static str {
    const FRAMES: [&str; 4] = ["·", "◜", "◝", "◞"];
    let elapsed = app
        .activity_started_at
        .map(|started| started.elapsed().as_millis() / 120)
        .unwrap_or(0);
    FRAMES[(elapsed as usize) % FRAMES.len()]
}

fn latest_action_summary(app: &TuiApp, chat_state: &super::app::ChatState) -> Option<String> {
    if let Some(approval) = &app.pending_approval {
        return Some(approval.command.clone());
    }

    for msg in chat_state.messages.iter().rev() {
        for line in msg.content.lines().rev() {
            let trimmed = line.trim();
            if let Some(command) = trimmed.strip_prefix("$ ") {
                return Some(command.to_string());
            }
        }
    }

    chat_state
        .messages
        .iter()
        .rev()
        .find(|msg| msg.role != "system" && !msg.content.trim().is_empty())
        .map(|msg| {
            msg.content
                .lines()
                .next()
                .unwrap_or_default()
                .trim()
                .to_string()
        })
        .filter(|line| !line.is_empty())
}

fn latest_rule_source(chat_state: &super::app::ChatState) -> Option<String> {
    chat_state.active_agents.as_ref().and_then(|agents| {
        agents
            .effective_rule_sections
            .iter()
            .rev()
            .find_map(|section| {
                section
                    .rules
                    .last()
                    .map(|rule| rule.source_path.display().to_string())
            })
    })
}

fn plan_panel_height(plan: &PlanState) -> u16 {
    let mut lines = plan.items.len().min(4) + 2;
    if plan.explanation.is_some() {
        lines += 1;
    }
    if plan
        .runtime
        .as_ref()
        .is_some_and(PlanRuntime::has_active_state)
    {
        lines += 1;
    }
    if let Some(runtime) = &plan.runtime {
        lines += runtime.jobs.len().min(3);
        if runtime.jobs.len() > 3 {
            lines += 1;
        }
        if !runtime.jobs.is_empty() {
            lines += 1;
            if runtime.jobs.iter().any(|job| {
                job.output_preview
                    .as_ref()
                    .is_some_and(|preview| !preview.is_empty())
            }) {
                lines += 2;
            }
        }
    }
    lines as u16
}

fn command_output_panel_height(output: &CommandOutputState) -> u16 {
    let line_count = output.content.lines().count().clamp(1, 5);
    (line_count + 3) as u16
}

fn draw_command_output_panel(
    frame: &mut Frame,
    output: &CommandOutputState,
    theme: &Theme,
    area: Rect,
) {
    let status = if output.done { "done" } else { "live" };
    let title = format!(" Command Output ({}) ", status);
    let color = if output.has_error {
        Color::Rgb(245, 158, 11)
    } else {
        theme.output
    };
    let mut lines = vec![Line::styled(
        truncate_chat_summary(&output.command, 96),
        theme.muted_style().add_modifier(Modifier::ITALIC),
    )];
    let content = if output.content.trim().is_empty() {
        "No output yet"
    } else {
        output.content.trim_end()
    };
    for line in content
        .lines()
        .rev()
        .take(4)
        .collect::<Vec<_>>()
        .into_iter()
        .rev()
    {
        lines.push(Line::styled(line.to_string(), Style::default().fg(color)));
    }
    let block = Block::default()
        .title(title)
        .borders(Borders::TOP)
        .border_style(theme.muted_style())
        .padding(Padding::horizontal(1));
    let widget = Paragraph::new(lines).block(block).wrap(Wrap { trim: true });
    frame.render_widget(widget, area);
}

fn draw_review_panel(
    frame: &mut Frame,
    review: &ReviewState,
    selected: usize,
    focused: bool,
    theme: &Theme,
    area: Rect,
) {
    let mut lines = Vec::new();
    lines.push(Line::styled(review.summary.as_str(), theme.muted_style()));
    if let Some(model) = &review.model {
        lines.push(Line::styled(
            format!("model {}", model),
            theme.muted_style().add_modifier(Modifier::ITALIC),
        ));
    }
    for (index, finding) in review.findings.iter().take(3).enumerate() {
        let severity_color = match finding.severity.as_str() {
            "error" => Color::Rgb(239, 68, 68),
            "warning" => Color::Rgb(245, 158, 11),
            _ => theme.accent,
        };
        let marker = if index == selected { "▸" } else { "•" };
        lines.push(Line::from(vec![
            Span::styled(marker, Style::default().fg(severity_color)),
            Span::raw(" "),
            Span::styled(
                format!("{}: {}", finding.severity, finding.title),
                if index == selected {
                    Style::default()
                        .fg(theme.foreground)
                        .add_modifier(Modifier::BOLD)
                } else {
                    Style::default().fg(theme.foreground)
                },
            ),
        ]));
        if index == selected {
            for detail_line in finding.message.lines().take(3) {
                lines.push(Line::styled(
                    format!("    {}", truncate_chat_summary(detail_line, 88)),
                    theme.muted_style(),
                ));
            }
        } else {
            lines.push(Line::styled(
                format!("  {}", truncate_chat_summary(&finding.message, 88)),
                theme.muted_style(),
            ));
        }
    }
    if review.findings.len() > 3 {
        lines.push(Line::styled(
            format!("{} more findings", review.findings.len() - 3),
            theme.muted_style(),
        ));
    }

    let title = if focused {
        " Review Findings (focused • Y/V or ↑/↓) "
    } else {
        " Review Findings (Ctrl+F focus) "
    };
    let block = Block::default()
        .title(title)
        .borders(Borders::TOP)
        .border_style(theme.muted_style())
        .padding(Padding::horizontal(1));
    let widget = Paragraph::new(lines).block(block).wrap(Wrap { trim: true });
    frame.render_widget(widget, area);
}

fn draw_plan_panel(
    frame: &mut Frame,
    plan: &PlanState,
    selected_job: usize,
    focused: bool,
    theme: &Theme,
    area: Rect,
) {
    let mut lines = Vec::new();

    if let Some(explanation) = &plan.explanation {
        lines.push(Line::styled(explanation.as_str(), theme.muted_style()));
    }
    if let Some(runtime) = plan
        .runtime
        .as_ref()
        .filter(|runtime| runtime.has_active_state())
    {
        lines.push(Line::styled(
            format_plan_runtime(runtime),
            theme.muted_style().add_modifier(Modifier::ITALIC),
        ));
    }
    if let Some(runtime) = &plan.runtime {
        lines.extend(plan_job_lines(runtime, selected_job, focused, theme));
        if let Some(selected) = runtime
            .jobs
            .get(selected_job.min(runtime.jobs.len().saturating_sub(1)))
        {
            lines.push(Line::styled(
                format!(
                    "job {} • {}",
                    &selected.job_id[..selected.job_id.len().min(8)],
                    selected.tool
                ),
                theme.muted_style(),
            ));
            if let Some(preview) = selected
                .output_preview
                .as_ref()
                .filter(|preview| !preview.is_empty())
            {
                let preview_color = if selected.has_error_output {
                    Color::Rgb(245, 158, 11)
                } else {
                    theme.output
                };
                for line in preview.lines().take(2) {
                    lines.push(Line::styled(
                        format!("  {}", truncate_chat_summary(line, 88)),
                        Style::default().fg(preview_color),
                    ));
                }
            }
        }
    }

    for item in plan.items.iter().take(4) {
        let (marker, color) = match item.status {
            PlanStepStatus::Pending => ("○", theme.muted),
            PlanStepStatus::InProgress => ("◐", theme.accent),
            PlanStepStatus::Completed => ("●", theme.output),
            PlanStepStatus::Blocked => ("■", Color::Rgb(245, 158, 11)),
        };
        lines.push(Line::from(vec![
            Span::styled(marker, Style::default().fg(color)),
            Span::raw(" "),
            Span::styled(item.step.as_str(), Style::default().fg(theme.foreground)),
        ]));
    }

    if plan.items.len() > 4 {
        lines.push(Line::styled(
            format!("{} more steps", plan.items.len() - 4),
            theme.muted_style(),
        ));
    }

    let title = if focused {
        " Plan (focused • Y/V or ↑/↓) "
    } else {
        " Plan (Ctrl+P jobs) "
    };
    let block = Block::default()
        .title(title)
        .borders(Borders::TOP)
        .border_style(theme.muted_style())
        .padding(Padding::horizontal(1));
    let widget = Paragraph::new(lines).block(block).wrap(Wrap { trim: true });
    frame.render_widget(widget, area);
}

fn agents_panel_height(agents: &ResolvedAgents, expanded: bool) -> u16 {
    if expanded {
        return 10;
    }

    let section_lines: usize = agents
        .effective_rule_sections
        .iter()
        .take(2)
        .map(|section| 2 + usize::from(!section.rules.is_empty()))
        .sum();
    let overflow_lines = usize::from(agents.effective_rule_sections.len() > 2);
    (section_lines + overflow_lines + 2) as u16
}

fn draw_agents_panel(
    frame: &mut Frame,
    agents: Option<&ResolvedAgents>,
    theme: &Theme,
    area: Rect,
    expanded: bool,
    scroll_offset: usize,
    focused: bool,
) {
    let lines = if let Some(agents) = agents {
        if expanded {
            expanded_agents_lines(agents, theme, scroll_offset, area.height)
        } else {
            compact_agents_lines(agents, theme)
        }
    } else {
        vec![
            Line::styled(
                "No active AGENTS sources yet.",
                Style::default().fg(theme.foreground),
            ),
            Line::styled(
                "Send a message or open a session with resolved AGENTS context.",
                theme.muted_style(),
            ),
        ]
    };

    let title = if expanded && focused {
        " AGENTS (focused • Y/V or ↑/↓) "
    } else if expanded {
        " AGENTS (expanded • Ctrl+A focus) "
    } else {
        " AGENTS (Ctrl+A open) "
    };
    let block = Block::default()
        .title(title)
        .borders(Borders::TOP)
        .border_style(theme.muted_style())
        .padding(Padding::horizontal(1));
    let widget = Paragraph::new(lines).block(block).wrap(Wrap { trim: true });
    frame.render_widget(widget, area);
}

fn compact_agents_lines(agents: &ResolvedAgents, theme: &Theme) -> Vec<Line<'static>> {
    let mut lines = Vec::new();
    for section in agents.effective_rule_sections.iter().take(2) {
        lines.push(Line::from(vec![
            Span::styled("•", Style::default().fg(theme.accent)),
            Span::raw(" "),
            Span::styled(
                section
                    .heading
                    .clone()
                    .unwrap_or_else(|| "General".to_string()),
                Style::default().fg(theme.foreground),
            ),
        ]));
        if let Some(preview) = section.rules.first() {
            lines.push(Line::styled(
                format!("  {}", truncate_agents_rule(&preview.text)),
                theme.muted_style(),
            ));
        }
    }
    if agents.effective_rule_sections.len() > 2 {
        lines.push(Line::styled(
            format!("{} more sections", agents.effective_rule_sections.len() - 2),
            theme.muted_style(),
        ));
    }
    lines
}

fn expanded_agents_lines(
    agents: &ResolvedAgents,
    theme: &Theme,
    scroll_offset: usize,
    panel_height: u16,
) -> Vec<Line<'static>> {
    let mut full_lines = Vec::new();
    for section in &agents.effective_rule_sections {
        full_lines.push(Line::from(vec![
            Span::styled("•", Style::default().fg(theme.accent)),
            Span::raw(" "),
            Span::styled(
                section
                    .heading
                    .clone()
                    .unwrap_or_else(|| "General".to_string()),
                Style::default()
                    .fg(theme.foreground)
                    .add_modifier(Modifier::BOLD),
            ),
        ]));
        for rule in section.rules.iter().take(5) {
            full_lines.push(Line::styled(
                format!("  {}", rule.text),
                theme.muted_style(),
            ));
            full_lines.push(Line::styled(
                format!("    source: {}", rule.source_path.display()),
                theme.muted_style().add_modifier(Modifier::ITALIC),
            ));
        }
        full_lines.push(Line::raw(""));
    }

    let visible_height = panel_height.saturating_sub(2) as usize;
    let max_offset = full_lines.len().saturating_sub(visible_height.max(1));
    let offset = scroll_offset.min(max_offset);
    let end = (offset + visible_height.max(1)).min(full_lines.len());
    full_lines[offset..end].to_vec()
}

fn truncate_agents_rule(rule: &str) -> String {
    const MAX_LEN: usize = 72;
    if rule.len() <= MAX_LEN {
        rule.to_string()
    } else {
        format!("{}...", &rule[..MAX_LEN])
    }
}

fn format_plan_runtime(runtime: &PlanRuntime) -> String {
    let status = runtime
        .active_status
        .as_deref()
        .unwrap_or("running")
        .replace('_', " ");
    let target = runtime
        .active_command
        .as_deref()
        .or(runtime.active_tool.as_deref())
        .unwrap_or("task");
    format!("{}: {}", status, target)
}

fn plan_job_lines(
    runtime: &PlanRuntime,
    selected_job: usize,
    focused: bool,
    theme: &Theme,
) -> Vec<Line<'static>> {
    let mut lines = Vec::new();
    let jobs = &runtime.jobs;
    let selected_job = selected_job.min(jobs.len().saturating_sub(1));
    let window_end = (selected_job + 1).max(3).min(jobs.len());
    let window_start = window_end.saturating_sub(3);

    for (index, job) in jobs
        .iter()
        .enumerate()
        .skip(window_start)
        .take(window_end.saturating_sub(window_start))
    {
        let (marker, color) = match job.status {
            PlanJobStatus::WaitingApproval => ("◌", theme.muted),
            PlanJobStatus::Running => ("◐", theme.accent),
            PlanJobStatus::Blocked => ("■", Color::Rgb(245, 158, 11)),
            PlanJobStatus::Succeeded => ("●", theme.output),
            PlanJobStatus::Failed => ("✕", Color::Rgb(239, 68, 68)),
        };
        let is_selected = focused && index == selected_job;
        lines.push(Line::from(vec![
            Span::styled(marker, Style::default().fg(color)),
            Span::raw(" "),
            Span::styled(
                format_plan_job(job),
                if is_selected {
                    Style::default()
                        .fg(theme.foreground)
                        .add_modifier(Modifier::BOLD)
                } else {
                    theme.muted_style().add_modifier(Modifier::ITALIC)
                },
            ),
        ]));
    }

    if runtime.jobs.len() > window_end {
        lines.push(Line::styled(
            format!("{} more jobs", runtime.jobs.len() - window_end),
            theme.muted_style(),
        ));
    }

    lines
}

fn format_plan_job(job: &PlanJobRecord) -> String {
    let target = job.command.as_deref().unwrap_or(&job.tool);
    let status = match job.status {
        PlanJobStatus::WaitingApproval => "waiting approval",
        PlanJobStatus::Running => "running",
        PlanJobStatus::Blocked => "blocked",
        PlanJobStatus::Succeeded => "succeeded",
        PlanJobStatus::Failed => "failed",
    };
    format!("{}: {}", status, truncate_chat_summary(target, 64))
}

fn draw_stats(
    frame: &mut Frame,
    stats: &harper_core::memory::session_service::GlobalStats,
    theme: &Theme,
    area: Rect,
) {
    let stats_lines = vec![
        Line::from(vec![
            Span::styled("Total Sessions   ", theme.muted_style()),
            Span::styled(
                stats.total_sessions.to_string(),
                Style::default()
                    .fg(theme.accent)
                    .add_modifier(Modifier::BOLD),
            ),
        ]),
        Line::from(vec![
            Span::styled("Total Messages   ", theme.muted_style()),
            Span::styled(
                stats.total_messages.to_string(),
                Style::default()
                    .fg(theme.accent)
                    .add_modifier(Modifier::BOLD),
            ),
        ]),
        Line::from(vec![
            Span::styled("Total Commands   ", theme.muted_style()),
            Span::styled(
                stats.total_commands.to_string(),
                Style::default()
                    .fg(theme.accent)
                    .add_modifier(Modifier::BOLD),
            ),
        ]),
        Line::from(vec![
            Span::styled("Approved Commands ", theme.muted_style()),
            Span::styled(
                stats.approved_commands.to_string(),
                Style::default()
                    .fg(theme.accent)
                    .add_modifier(Modifier::BOLD),
            ),
        ]),
        Line::from(vec![
            Span::styled("Avg Duration     ", theme.muted_style()),
            Span::styled(
                format!("{:.2} ms", stats.avg_command_duration_ms),
                Style::default()
                    .fg(theme.accent)
                    .add_modifier(Modifier::BOLD),
            ),
        ]),
    ];

    let area = centered_rect(50, 40, area);
    let stats_widget = Paragraph::new(stats_lines).block(
        Block::default()
            .title(" Usage Statistics ")
            .title_style(
                Style::default()
                    .fg(theme.foreground)
                    .add_modifier(Modifier::BOLD),
            )
            .padding(Padding::uniform(2)),
    );

    frame.render_widget(stats_widget, area);
}

fn draw_zen_sidebar(frame: &mut Frame, entries: &[String], theme: &Theme, area: Rect) {
    let items: Vec<ListItem> = entries
        .iter()
        .map(|entry| {
            let content = if entry.starts_with("[git]") {
                entry.trim_start_matches("[git] ").to_string()
            } else if entry.starts_with("[dir]") {
                entry.trim_start_matches("[dir] ").to_string()
            } else if entry.starts_with("[file]") {
                entry.trim_start_matches("[file] ").to_string()
            } else if entry.starts_with("[probe]") {
                entry.trim_start_matches("[probe] ").to_string()
            } else {
                entry.clone()
            };

            ListItem::new(Line::from(vec![Span::styled(content, theme.muted_style())]))
        })
        .collect();

    let sidebar = List::new(items).block(
        Block::default()
            .borders(Borders::RIGHT)
            .border_style(theme.muted_style())
            .padding(Padding::horizontal(1)),
    );

    frame.render_widget(sidebar, area);
}

fn draw_zen_menu(frame: &mut Frame, selected: usize, theme: &Theme, area: Rect) {
    let menu_items = [
        "New Conversation",
        "History",
        "Export",
        "Statistics",
        "Settings",
        "Quit",
    ];

    let area = centered_rect(40, 35, area);

    let items: Vec<ListItem> = menu_items
        .iter()
        .enumerate()
        .map(|(i, label)| {
            let (prefix, style) = if i == selected {
                (
                    "› ",
                    Style::default()
                        .fg(theme.accent)
                        .add_modifier(Modifier::BOLD),
                )
            } else {
                ("  ", Style::default().fg(theme.muted))
            };

            ListItem::new(Line::from(vec![
                Span::styled(prefix, style),
                Span::styled(*label, style),
            ]))
        })
        .collect();

    let menu = List::new(items).block(
        Block::default()
            .title("Harper")
            .title_style(
                Style::default()
                    .fg(theme.foreground)
                    .add_modifier(Modifier::BOLD),
            )
            .padding(Padding::uniform(1)),
    );

    frame.render_widget(menu, area);
}

fn centered_rect(percent_x: u16, percent_y: u16, r: Rect) -> Rect {
    let popup_layout = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Percentage((100 - percent_y) / 2),
            Constraint::Percentage(percent_y),
            Constraint::Percentage((100 - percent_y) / 2),
        ])
        .split(r);

    Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Percentage((100 - percent_x) / 2),
            Constraint::Percentage(percent_x),
            Constraint::Percentage((100 - percent_x) / 2),
        ])
        .split(popup_layout[1])[1]
}

fn draw_zen_footer(frame: &mut Frame, _app: &TuiApp, theme: &Theme, area: Rect) {
    let block = Block::default()
        .borders(Borders::TOP)
        .border_style(theme.muted_style());
    let area = block.inner(area);
    frame.render_widget(block, frame.area()); // Render border on full area

    let num_cols = FOOTER_SHORTCUTS[0].len().max(1) as u16;
    let col_width = (area.width / num_cols).max(1);
    for (row_idx, row) in FOOTER_SHORTCUTS.iter().enumerate() {
        for (col_idx, (key, label)) in row.iter().enumerate() {
            let shortcut_area = Rect {
                x: area.x + col_idx as u16 * col_width,
                y: area.y + row_idx as u16,
                width: col_width,
                height: 1,
            };

            let shortcut_text = Line::from(vec![
                Span::styled(*key, Style::default().fg(theme.accent)),
                Span::styled(format!(" {}", label), theme.muted_style()),
            ]);

            frame.render_widget(Paragraph::new(shortcut_text), shortcut_area);
        }
    }
}

fn draw_approval(frame: &mut Frame, state: &ApprovalState, theme: &Theme) {
    let content = format!(
        "{}\n\nCommand:\n{}\n\nControls: Y approve • N reject • Esc reject • ↑/↓ or J/K scroll",
        state.prompt, state.command
    );
    let area = frame.area();
    let overlay_width = (content.len() as u16 + 4).min(area.width * 3 / 4).max(40);
    let overlay_height = (content.lines().count() as u16 + 4)
        .min(area.height / 2)
        .max(10);

    let overlay_area = Rect {
        x: (area.width - overlay_width) / 2,
        y: (area.height - overlay_height) / 2,
        width: overlay_width,
        height: overlay_height,
    };

    let block = Block::default()
        .borders(Borders::ALL)
        .border_style(theme.muted_style())
        .title(" Approval Required ")
        .title_style(theme.warning_style())
        .style(Style::default().bg(theme.background));

    let paragraph = Paragraph::new(content)
        .block(block)
        .alignment(Alignment::Left)
        .wrap(Wrap { trim: true })
        .scroll((state.scroll_offset, 0));

    frame.render_widget(Clear, overlay_area);
    frame.render_widget(paragraph, overlay_area);
}

fn draw_sessions(
    frame: &mut Frame,
    sessions: &[SessionInfo],
    selected: usize,
    remote_mode: bool,
    theme: &Theme,
    area: Rect,
) {
    if sessions.is_empty() {
        let detail = if remote_mode {
            "No remote sessions were found for the signed-in user. Local-only sessions remain available under Export."
        } else {
            "Start a conversation first, then return here to resume it."
        };
        let empty = Paragraph::new(vec![
            Line::from(vec![Span::styled(
                "No sessions found",
                Style::default()
                    .fg(theme.foreground)
                    .add_modifier(Modifier::BOLD),
            )]),
            Line::from(""),
            Line::from(vec![Span::styled(detail, theme.muted_style())]),
        ])
        .block(
            Block::default()
                .title(if remote_mode {
                    " Remote Sessions "
                } else {
                    " Sessions "
                })
                .padding(Padding::uniform(1)),
        )
        .wrap(Wrap { trim: true });
        frame.render_widget(empty, area);
        return;
    }

    let items: Vec<ListItem> = sessions
        .iter()
        .enumerate()
        .map(|(i, session)| {
            let style = if i == selected {
                Style::default()
                    .fg(theme.accent)
                    .add_modifier(Modifier::BOLD)
            } else {
                Style::default().fg(theme.foreground)
            };
            ListItem::new(vec![
                Line::from(vec![
                    Span::styled(session.name.clone(), style),
                    Span::styled("  ", theme.muted_style()),
                    Span::styled(session.created_at.clone(), theme.muted_style()),
                ]),
                Line::from(vec![Span::styled(
                    truncate_chat_summary(&session.id, 48),
                    theme.muted_style(),
                )]),
            ])
            .style(style)
        })
        .collect();

    let sessions_list = List::new(items).block(
        Block::default()
            .title(if remote_mode {
                " Remote Sessions (Enter resume • → preview) "
            } else {
                " Sessions (Enter resume • → preview) "
            })
            .padding(Padding::uniform(1)),
    );

    frame.render_widget(sessions_list, area);
}

fn draw_export_sessions(
    frame: &mut Frame,
    sessions: &[SessionInfo],
    selected: usize,
    theme: &Theme,
    area: Rect,
) {
    if sessions.is_empty() {
        let empty = Paragraph::new(vec![
            Line::from(vec![Span::styled(
                "No sessions available to export",
                Style::default()
                    .fg(theme.foreground)
                    .add_modifier(Modifier::BOLD),
            )]),
            Line::from(""),
            Line::from(vec![Span::styled(
                "Create a conversation first, then return here to export it.",
                theme.muted_style(),
            )]),
        ])
        .block(
            Block::default()
                .title(" Export Sessions ")
                .padding(Padding::uniform(1)),
        )
        .wrap(Wrap { trim: true });
        frame.render_widget(empty, area);
        return;
    }

    let items: Vec<ListItem> = sessions
        .iter()
        .enumerate()
        .map(|(i, session)| {
            let style = if i == selected {
                Style::default()
                    .fg(theme.accent)
                    .add_modifier(Modifier::BOLD)
            } else {
                Style::default().fg(theme.foreground)
            };
            ListItem::new(vec![
                Line::from(vec![
                    Span::styled(display_session_name(session), style),
                    Span::styled("  ", theme.muted_style()),
                    Span::styled(session.created_at.clone(), theme.muted_style()),
                ]),
                Line::from(vec![Span::styled(
                    truncate_chat_summary(&session.id, 48),
                    theme.muted_style(),
                )]),
            ])
            .style(style)
        })
        .collect();

    let sessions_list = List::new(items).block(
        Block::default()
            .title(" Export Local Sessions ")
            .padding(Padding::uniform(1)),
    );

    frame.render_widget(sessions_list, area);
}

fn display_session_name(session: &SessionInfo) -> String {
    let trimmed = session.name.trim();
    if !trimmed.is_empty() && trimmed != session.id {
        return trimmed.to_string();
    }

    if session.id.len() >= 8 {
        return format!("Session {}", &session.id[..8]);
    }

    "Untitled session".to_string()
}

fn draw_tools(frame: &mut Frame, selected: usize, theme: &Theme, area: Rect) {
    let tools = ["Profile", "Search", "System", "Processes", "Git"];
    let items: Vec<ListItem> = tools
        .iter()
        .enumerate()
        .map(|(i, tool)| {
            let style = if i == selected {
                Style::default()
                    .fg(theme.accent)
                    .add_modifier(Modifier::BOLD)
            } else {
                Style::default().fg(theme.foreground)
            };
            ListItem::new(*tool).style(style)
        })
        .collect();

    let tools_list = List::new(items).block(
        Block::default()
            .title(" Settings ")
            .padding(Padding::uniform(1)),
    );

    frame.render_widget(tools_list, area);
}

fn draw_profile(frame: &mut Frame, app: &TuiApp, selected: usize, theme: &Theme, area: Rect) {
    let info_lines = if let Some(session) = app.auth_session.as_ref() {
        vec![
            Line::from(vec![
                Span::styled("Status: ", Style::default().fg(theme.foreground)),
                Span::styled("Signed in", Style::default().fg(theme.accent)),
            ]),
            Line::from(vec![
                Span::styled("Email: ", Style::default().fg(theme.foreground)),
                Span::styled(
                    session
                        .user
                        .email
                        .clone()
                        .unwrap_or_else(|| "—".to_string()),
                    theme.muted_style(),
                ),
            ]),
            Line::from(vec![
                Span::styled("Name: ", Style::default().fg(theme.foreground)),
                Span::styled(
                    session
                        .user
                        .display_name
                        .clone()
                        .unwrap_or_else(|| "—".to_string()),
                    theme.muted_style(),
                ),
            ]),
            Line::from(vec![
                Span::styled("Provider: ", Style::default().fg(theme.foreground)),
                Span::styled(
                    session
                        .user
                        .provider
                        .as_ref()
                        .map(|provider| format!("{provider:?}"))
                        .unwrap_or_else(|| "—".to_string()),
                    theme.muted_style(),
                ),
            ]),
            Line::from(vec![
                Span::styled("User ID: ", Style::default().fg(theme.foreground)),
                Span::styled(session.user.user_id.clone(), theme.muted_style()),
            ]),
            Line::from(""),
            Line::styled(
                "Session storage uses the local OS keychain/keyring.",
                theme.muted_style(),
            ),
            Line::styled(
                "History, Export, and Statistics are scoped to this signed-in account.",
                theme.muted_style(),
            ),
        ]
    } else {
        vec![
            Line::from(vec![
                Span::styled("Status: ", Style::default().fg(theme.foreground)),
                Span::styled("Not signed in", theme.muted_style()),
            ]),
            Line::from(""),
            Line::styled(
                "Sign in with /auth login <provider> in chat to enable account-scoped History, Export, and Statistics.",
                theme.muted_style(),
            ),
            Line::styled(
                "Without sign-in, Harper stays in local-only mode on this machine.",
                theme.muted_style(),
            ),
        ]
    };

    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Min(8), Constraint::Length(6)])
        .split(area);

    let profile = Paragraph::new(info_lines)
        .block(
            Block::default()
                .title(" Profile ")
                .padding(Padding::uniform(1)),
        )
        .wrap(Wrap { trim: true });
    frame.render_widget(profile, chunks[0]);

    let actions: Vec<&str> = if app.auth_session.is_some() {
        vec!["Logout", "Refresh Session"]
    } else {
        vec!["Login with GitHub", "Login with Google", "Login with Apple"]
    };
    let items: Vec<ListItem> = actions
        .iter()
        .enumerate()
        .map(|(index, label)| {
            let style = if index == selected {
                Style::default()
                    .fg(theme.accent)
                    .add_modifier(Modifier::BOLD)
            } else {
                Style::default().fg(theme.foreground)
            };
            ListItem::new(*label).style(style)
        })
        .collect();

    let actions_list = List::new(items).block(
        Block::default()
            .title(" Actions ")
            .padding(Padding::uniform(1)),
    );
    frame.render_widget(actions_list, chunks[1]);
}

fn draw_view_session(
    frame: &mut Frame,
    name: &str,
    messages: &[harper_core::core::Message],
    selected: usize,
    theme: &Theme,
    area: Rect,
) {
    let safe_scroll_offset = selected.min(messages.len());
    let displayed_messages = &messages[safe_scroll_offset..];
    let mut message_lines: Vec<Line> = Vec::new();
    for msg in displayed_messages.iter().filter(|msg| msg.role != "system") {
        let label = match msg.role.as_str() {
            "user" => "User ›",
            "assistant" => "Harper ›",
            _ => "System ›",
        };
        let label_style = match msg.role.as_str() {
            "user" => Style::default()
                .fg(theme.accent)
                .add_modifier(Modifier::BOLD),
            "assistant" => Style::default()
                .fg(theme.title)
                .add_modifier(Modifier::BOLD),
            _ => theme.muted_style(),
        };
        message_lines.push(Line::from(vec![Span::styled(label, label_style)]));
        let default_color = if msg.role == "user" {
            theme.input
        } else {
            theme.output
        };
        if msg.content.contains("```") {
            let spans = parse_content_with_code(
                &theme.syntax_set,
                &theme.theme_set,
                &msg.content,
                default_color,
                &theme.syntax_theme,
            );
            message_lines.push(Line::from(spans));
        } else {
            for line in msg.content.lines() {
                message_lines.push(Line::styled(line, default_color));
            }
        }
        message_lines.push(Line::raw(""));
    }

    let view = Paragraph::new(message_lines)
        .block(
            Block::default()
                .title(format!(" Preview {} (Enter resume • Esc back) ", name))
                .padding(Padding::uniform(1)),
        )
        .wrap(Wrap { trim: false });

    frame.render_widget(view, area);
}

fn draw_message_overlay(frame: &mut Frame, message: &UiMessage, theme: &Theme) {
    let area = frame.area();
    let max_overlay_width = area.width.saturating_mul(3) / 4;
    let content_width = message
        .content
        .lines()
        .map(|line| line.chars().count() as u16)
        .max()
        .unwrap_or(0);
    let overlay_width = (content_width + 8).clamp(24, max_overlay_width.max(24));
    let text_width = overlay_width.saturating_sub(4).max(1) as usize;
    let wrapped_line_count = message
        .content
        .lines()
        .map(|line| {
            let len = line.chars().count();
            usize::max(1, len.div_ceil(text_width))
        })
        .sum::<usize>() as u16;
    let overlay_height = (wrapped_line_count + 4).clamp(5, area.height.saturating_mul(3) / 4);

    let overlay = Paragraph::new(message.content.as_str())
        .block(
            Block::default()
                .borders(Borders::ALL)
                .border_style(theme.muted_style())
                .padding(Padding::uniform(1)),
        )
        .style(Style::default().bg(theme.background).fg(theme.foreground))
        .alignment(Alignment::Center)
        .wrap(Wrap { trim: true });

    let overlay_area = Rect {
        x: (area.width - overlay_width) / 2,
        y: (area.height - overlay_height) / 2,
        width: overlay_width,
        height: overlay_height,
    };

    frame.render_widget(Clear, overlay_area);
    frame.render_widget(overlay, overlay_area);
}

fn draw_plan_jobs_browser(frame: &mut Frame, chat_state: &super::app::ChatState, theme: &Theme) {
    let Some(plan) = &chat_state.active_plan else {
        return;
    };
    let Some(runtime) = &plan.runtime else {
        return;
    };
    if runtime.jobs.is_empty() {
        return;
    }

    let selected_index = chat_state
        .plan_job_selected
        .min(runtime.jobs.len().saturating_sub(1));
    let selected = &runtime.jobs[selected_index];
    let overlay_area = centered_rect(80, 70, frame.area());
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(3),
            Constraint::Length(4),
            Constraint::Min(8),
        ])
        .split(overlay_area);

    frame.render_widget(Clear, overlay_area);

    let title = Paragraph::new(format!(
        "Planner Jobs • job {} • {} • Esc close • Y/V select • ↑/↓ scroll",
        &selected.job_id[..selected.job_id.len().min(8)],
        selected.tool
    ))
    .block(
        Block::default()
            .borders(Borders::ALL)
            .border_style(theme.muted_style())
            .style(Style::default().bg(theme.background)),
    )
    .style(Style::default().fg(theme.foreground));
    frame.render_widget(title, chunks[0]);

    let summary_lines = plan_job_lines(runtime, selected_index, true, theme);
    let summary = Paragraph::new(summary_lines)
        .block(
            Block::default()
                .title(" Recent Jobs ")
                .borders(Borders::LEFT | Borders::RIGHT | Borders::BOTTOM)
                .border_style(theme.muted_style())
                .style(Style::default().bg(theme.background)),
        )
        .wrap(Wrap { trim: true });
    frame.render_widget(summary, chunks[1]);

    let detail_lines = plan_job_transcript_lines(selected, theme);
    let detail = Paragraph::new(detail_lines)
        .block(
            Block::default()
                .title(" Output Transcript ")
                .borders(Borders::LEFT | Borders::RIGHT | Borders::BOTTOM)
                .border_style(theme.muted_style())
                .style(Style::default().bg(theme.background))
                .padding(Padding::horizontal(1)),
        )
        .scroll((chat_state.plan_job_output_scroll as u16, 0))
        .wrap(Wrap { trim: false });
    frame.render_widget(detail, chunks[2]);
}

fn plan_job_transcript_lines(job: &PlanJobRecord, theme: &Theme) -> Vec<Line<'static>> {
    let transcript = if job.output_transcript.trim().is_empty() {
        "No output recorded yet".to_string()
    } else {
        job.output_transcript.clone()
    };
    let detail_color = if job.has_error_output {
        Color::Rgb(245, 158, 11)
    } else {
        theme.output
    };
    transcript
        .lines()
        .map(|line| Line::styled(line.to_string(), Style::default().fg(detail_color)))
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;
    use harper_core::PlanItem;
    use ratatui::style::Color;

    fn setup() -> (SyntaxSet, ThemeSet) {
        (
            SyntaxSet::load_defaults_newlines(),
            ThemeSet::load_defaults(),
        )
    }

    #[test]
    fn test_parse_content_with_code_no_code() {
        let (syntax_set, theme_set) = setup();
        let spans = parse_content_with_code(
            &syntax_set,
            &theme_set,
            "Hello",
            Color::White,
            "base16-ocean.dark",
        );
        assert_eq!(spans.len(), 1);
    }

    #[test]
    fn plan_panel_height_accounts_for_recent_jobs() {
        let plan = PlanState {
            explanation: Some("Track execution".to_string()),
            items: vec![PlanItem {
                step: "Run command".to_string(),
                status: PlanStepStatus::InProgress,
                job_id: None,
            }],
            runtime: Some(PlanRuntime {
                active_tool: Some("run_command".to_string()),
                active_command: Some("echo hi".to_string()),
                active_status: Some("running".to_string()),
                active_job_id: Some("job-1".to_string()),
                jobs: vec![
                    PlanJobRecord {
                        job_id: "job-1".to_string(),
                        tool: "run_command".to_string(),
                        command: Some("echo hi".to_string()),
                        status: PlanJobStatus::Running,
                        output_transcript: "line one\nline two".to_string(),
                        output_preview: Some("line one\nline two".to_string()),
                        has_error_output: false,
                    },
                    PlanJobRecord {
                        job_id: "job-2".to_string(),
                        tool: "run_command".to_string(),
                        command: Some("ls".to_string()),
                        status: PlanJobStatus::Succeeded,
                        output_transcript: String::new(),
                        output_preview: None,
                        has_error_output: false,
                    },
                ],
            }),
            updated_at: None,
        };

        assert_eq!(plan_panel_height(&plan), 10);
    }

    #[test]
    fn plan_job_lines_show_recent_jobs_and_overflow() {
        let runtime = PlanRuntime {
            active_tool: None,
            active_command: None,
            active_status: None,
            active_job_id: None,
            jobs: vec![
                PlanJobRecord {
                    job_id: "job-1".to_string(),
                    tool: "run_command".to_string(),
                    command: Some("echo one".to_string()),
                    status: PlanJobStatus::Succeeded,
                    output_transcript: String::new(),
                    output_preview: None,
                    has_error_output: false,
                },
                PlanJobRecord {
                    job_id: "job-2".to_string(),
                    tool: "run_command".to_string(),
                    command: Some("echo two".to_string()),
                    status: PlanJobStatus::Failed,
                    output_transcript: "boom".to_string(),
                    output_preview: Some("boom".to_string()),
                    has_error_output: true,
                },
                PlanJobRecord {
                    job_id: "job-3".to_string(),
                    tool: "run_command".to_string(),
                    command: Some("echo three".to_string()),
                    status: PlanJobStatus::Running,
                    output_transcript: String::new(),
                    output_preview: None,
                    has_error_output: false,
                },
                PlanJobRecord {
                    job_id: "job-4".to_string(),
                    tool: "run_command".to_string(),
                    command: Some("echo four".to_string()),
                    status: PlanJobStatus::WaitingApproval,
                    output_transcript: String::new(),
                    output_preview: None,
                    has_error_output: false,
                },
            ],
        };

        let lines = plan_job_lines(&runtime, 0, true, &Theme::default());

        assert_eq!(lines.len(), 4);
        assert!(lines[0].to_string().contains("succeeded: echo one"));
        assert!(lines[1].to_string().contains("failed: echo two"));
        assert!(lines[2].to_string().contains("running: echo three"));
        assert!(lines[3].to_string().contains("1 more jobs"));
    }

    #[test]
    fn format_plan_job_prefers_command_text() {
        let job = PlanJobRecord {
            job_id: "job-1".to_string(),
            tool: "run_command".to_string(),
            command: Some("echo hi".to_string()),
            status: PlanJobStatus::Blocked,
            output_transcript: String::new(),
            output_preview: None,
            has_error_output: false,
        };

        assert_eq!(format_plan_job(&job), "blocked: echo hi");
    }

    #[test]
    fn plan_job_transcript_lines_use_full_transcript_and_placeholder() {
        let theme = Theme::default();
        let job = PlanJobRecord {
            job_id: "job-1".to_string(),
            tool: "run_command".to_string(),
            command: Some("echo hi".to_string()),
            status: PlanJobStatus::Succeeded,
            output_transcript: "line one\nline two\nline three".to_string(),
            output_preview: Some("line one\nline two".to_string()),
            has_error_output: false,
        };
        let placeholder_job = PlanJobRecord {
            output_transcript: String::new(),
            ..job.clone()
        };

        let transcript_lines = plan_job_transcript_lines(&job, &theme);
        let placeholder_lines = plan_job_transcript_lines(&placeholder_job, &theme);

        assert_eq!(transcript_lines.len(), 3);
        assert_eq!(transcript_lines[0].to_string(), "line one");
        assert_eq!(transcript_lines[2].to_string(), "line three");
        assert_eq!(placeholder_lines[0].to_string(), "No output recorded yet");
    }
}
