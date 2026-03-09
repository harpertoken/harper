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

use ratatui::prelude::*;
use ratatui::style::Modifier;
use ratatui::widgets::{Block, Borders, Clear, List, ListItem, Paragraph, Wrap};

use super::app::{AppState, ApprovalState, MessageType, SessionInfo, TuiApp, UiMessage};
use super::theme::Theme;

// Keyboard shortcut constants for the new footer
const FOOTER_SHORTCUTS: [[(&str, &str); 6]; 2] = [
    [
        ("^G", "Get Help"),
        ("^O", "Write Out"),
        ("^W", "Where Is"),
        ("^K", "Cut"),
        ("^T", "Execute"),
        ("^C", "Location"),
    ],
    [
        ("^X", "Exit"),
        ("^J", "Justify"),
        ("^R", "Read File"),
        ("^U", "Paste"),
        ("^Y", "Prev Page"),
        ("^V", "Next Page"),
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
        // Text before code block
        if start > 0 {
            spans.push(Span::styled(
                &remaining[..start],
                Style::default().fg(default_color),
            ));
        }

        // Find end of code block
        let after_start = &remaining[start + 3..];
        if let Some(end) = after_start.find("```") {
            let code_block = &after_start[..end];
            // Parse language and code
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
                // No language, treat as plain
                spans.push(Span::styled(code_block, Style::default().fg(default_color)));
            }
            remaining = &after_start[end + 3..];
        } else {
            // No closing ```, treat rest as text
            spans.push(Span::styled(
                &remaining[start..],
                Style::default().fg(default_color),
            ));
            remaining = "";
            break;
        }
    }

    // Remaining text
    if !remaining.is_empty() {
        spans.push(Span::styled(remaining, Style::default().fg(default_color)));
    }

    spans
}

pub fn draw(frame: &mut Frame, app: &TuiApp, theme: &Theme) {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Min(0),
            Constraint::Length(3), // Footer: 1 status line + 2 shortcut lines
        ])
        .split(frame.area());

    let main_area = chunks[0];
    let footer_area = chunks[1];

    match &app.state {
        AppState::Menu(selected) => draw_menu(frame, *selected, theme, main_area),
        AppState::Chat(chat_state) => {
            // Create layout for chat: messages area and input area
            let chunks = Layout::default()
                .direction(Direction::Vertical)
                .constraints([
                    Constraint::Min(5),    // Messages area
                    Constraint::Length(3), // Input area
                ])
                .split(main_area);

            // Messages area
            let safe_scroll_offset = chat_state.scroll_offset.min(chat_state.messages.len());
            let displayed_messages = &chat_state.messages[safe_scroll_offset..];
            let message_lines: Vec<Line> = displayed_messages
                .iter()
                .filter(|msg| msg.role != "system")
                .flat_map(|msg| {
                    let default_color = match msg.role.as_str() {
                        "user" => theme.input,
                        "assistant" => theme.output,
                        _ => theme.foreground,
                    };
                    if msg.content.contains("```") {
                        let spans = parse_content_with_code(
                            &theme.syntax_set,
                            &theme.theme_set,
                            &msg.content,
                            default_color,
                            &theme.syntax_theme,
                        );
                        vec![Line::from(spans)]
                    } else {
                        msg.content
                            .lines()
                            .map(|line| Line::styled(line, default_color))
                            .collect::<Vec<_>>()
                    }
                })
                .collect();

            let title = if chat_state.web_search_enabled {
                "Chat (Web Search Enabled)"
            } else {
                "Chat"
            };

            let messages_widget = Paragraph::new(message_lines)
                .block(
                    Block::default()
                        .borders(Borders::ALL)
                        .title(title)
                        .border_style(theme.border_style())
                        .title_style(theme.title_style()),
                )
                .wrap(Wrap { trim: false });

            frame.render_widget(messages_widget, chunks[0]);

            // Input area
            let input_widget = Paragraph::new(format!("> {}", chat_state.input))
                .block(
                    Block::default()
                        .borders(Borders::ALL)
                        .title("Input")
                        .border_style(theme.border_style())
                        .title_style(theme.title_style()),
                )
                .style(Style::default().bg(theme.background).fg(theme.input));

            frame.render_widget(input_widget, chunks[1]);
        }
        AppState::Sessions(sessions, selected) => {
            draw_sessions(frame, sessions, *selected, theme, main_area)
        }
        AppState::ExportSessions(sessions, selected) => {
            draw_export_sessions(frame, sessions, *selected, theme, main_area)
        }
        AppState::Tools(selected) => draw_tools(frame, *selected, theme, main_area),
        AppState::ViewSession(name, messages, selected) => {
            draw_view_session(frame, name, messages, *selected, theme, main_area)
        }
    }

    // Draw status bar
    draw_status_bar(frame, app, theme, footer_area);

    // Draw approval overlay if present
    if let Some(approval) = &app.pending_approval {
        draw_approval(frame, approval, theme);
    }

    // Draw message overlay if present
    if let Some(msg) = &app.message {
        draw_message_overlay(frame, msg, theme);
    }
}

fn draw_approval(frame: &mut Frame, state: &ApprovalState, theme: &Theme) {
    let content = format!(
        "{}\n\n{}\n\nPress 'y' to approve or 'n' to reject.\nUse ↑↓ to scroll.",
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
        .title("Security Approval Required")
        .border_style(theme.warning_style())
        .title_style(theme.title_style())
        .style(Style::default().bg(theme.background));

    let paragraph = Paragraph::new(content)
        .block(block)
        .alignment(Alignment::Center)
        .wrap(Wrap { trim: true })
        .scroll((state.scroll_offset, 0));

    frame.render_widget(Clear, overlay_area);
    frame.render_widget(paragraph, overlay_area);
}

fn draw_menu(frame: &mut Frame, selected: usize, theme: &Theme, area: Rect) {
    let menu_items = [
        "Start Chat",
        "Load Session",
        "Export Session",
        "Tools",
        "Exit",
    ];

    let items: Vec<ListItem> = menu_items
        .iter()
        .enumerate()
        .map(|(i, item)| {
            let style = if i == selected {
                Style::default().bg(theme.accent).fg(theme.foreground)
            } else {
                Style::default().fg(theme.foreground)
            };
            ListItem::new(*item).style(style)
        })
        .collect();

    let menu = List::new(items)
        .block(
            Block::default()
                .borders(Borders::ALL)
                .title("Harper")
                .border_style(theme.border_style())
                .title_style(theme.title_style()),
        )
        .highlight_style(Style::default().add_modifier(Modifier::BOLD));

    frame.render_widget(menu, area);
}

fn draw_sessions(
    frame: &mut Frame,
    sessions: &[SessionInfo],
    selected: usize,
    theme: &Theme,
    area: Rect,
) {
    let items: Vec<ListItem> = sessions
        .iter()
        .enumerate()
        .map(|(i, session)| {
            let style = if i == selected {
                Style::default().bg(theme.accent).fg(theme.foreground)
            } else {
                Style::default().fg(theme.foreground)
            };
            ListItem::new(format!("{} - {}", session.name, session.created_at)).style(style)
        })
        .collect();

    let sessions_list = List::new(items)
        .block(
            Block::default()
                .borders(Borders::ALL)
                .title("Sessions")
                .border_style(theme.border_style())
                .title_style(theme.title_style()),
        )
        .highlight_style(Style::default().add_modifier(Modifier::BOLD));

    frame.render_widget(sessions_list, area);
}

fn draw_export_sessions(
    frame: &mut Frame,
    sessions: &[SessionInfo],
    selected: usize,
    theme: &Theme,
    area: Rect,
) {
    let items: Vec<ListItem> = sessions
        .iter()
        .enumerate()
        .map(|(i, session)| {
            let style = if i == selected {
                Style::default().bg(theme.accent).fg(theme.foreground)
            } else {
                Style::default().fg(theme.foreground)
            };
            ListItem::new(format!("Export: {} - {}", session.name, session.created_at)).style(style)
        })
        .collect();

    let sessions_list = List::new(items)
        .block(
            Block::default()
                .borders(Borders::ALL)
                .title("Select Sessions to Export")
                .border_style(theme.border_style())
                .title_style(theme.title_style()),
        )
        .highlight_style(Style::default().add_modifier(Modifier::BOLD));

    frame.render_widget(sessions_list, area);
}

fn draw_tools(frame: &mut Frame, selected: usize, theme: &Theme, area: Rect) {
    let tools = ["Search", "System Info", "Process List", "Git Status"];

    let items: Vec<ListItem> = tools
        .iter()
        .enumerate()
        .map(|(i, tool)| {
            let style = if i == selected {
                Style::default().bg(theme.accent).fg(theme.foreground)
            } else {
                Style::default().fg(theme.foreground)
            };
            ListItem::new(*tool).style(style)
        })
        .collect();

    let tools_list = List::new(items)
        .block(
            Block::default()
                .borders(Borders::ALL)
                .title("Tools")
                .border_style(theme.border_style())
                .title_style(theme.title_style()),
        )
        .highlight_style(Style::default().add_modifier(Modifier::BOLD));

    frame.render_widget(tools_list, area);
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
    let message_lines: Vec<Line> = displayed_messages
        .iter()
        .flat_map(|msg| {
            let default_color = match msg.role.as_str() {
                "user" => theme.input,
                "assistant" => theme.output,
                _ => theme.foreground,
            };
            msg.content
                .lines()
                .map(|line| Line::styled(line, default_color))
                .collect::<Vec<_>>()
        })
        .collect();

    let view = Paragraph::new(message_lines)
        .block(
            Block::default()
                .borders(Borders::ALL)
                .title(format!("Session: {}", name))
                .border_style(theme.border_style())
                .title_style(theme.title_style()),
        )
        .wrap(Wrap { trim: false });

    frame.render_widget(view, area);
}

fn draw_status_bar(frame: &mut Frame, app: &TuiApp, theme: &Theme, area: Rect) {
    let status_text = if app.pending_approval.is_some() {
        " Approval Required "
    } else {
        match &app.state {
            AppState::Menu(_) => " Ready ",
            AppState::Chat(..) => " Chatting ",
            AppState::Sessions(_, _) => " Sessions ",
            AppState::ExportSessions(_, _) => " Export ",
            AppState::Tools(_) => " Tools ",
            AppState::ViewSession(_, _, _) => " Viewing ",
        }
    };

    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(1), // Status line
            Constraint::Length(2), // Shortcuts grid
        ])
        .split(area);

    // Draw status line (centered)
    let status_line = format!("[ {} ]", status_text.trim());
    let status_paragraph = Paragraph::new(status_line)
        .style(theme.title_style().add_modifier(Modifier::BOLD))
        .alignment(Alignment::Center);
    frame.render_widget(status_paragraph, chunks[0]);

    // Draw shortcuts grid
    let col_width = area.width / 6;
    for (row_idx, row) in FOOTER_SHORTCUTS.iter().enumerate() {
        for (col_idx, (key, label)) in row.iter().enumerate() {
            let shortcut_area = Rect {
                x: area.x + col_idx as u16 * col_width,
                y: chunks[1].y + row_idx as u16,
                width: col_width,
                height: 1,
            };

            let shortcut_text = Line::from(vec![
                Span::styled(
                    *key,
                    Style::default()
                        .bg(theme.foreground)
                        .fg(theme.background)
                        .add_modifier(Modifier::BOLD),
                ),
                Span::styled(format!(" {}", label), Style::default().fg(theme.foreground)),
            ]);

            frame.render_widget(Paragraph::new(shortcut_text), shortcut_area);
        }
    }
}

fn draw_message_overlay(frame: &mut Frame, message: &UiMessage, theme: &Theme) {
    let (title, style, border_style) = match message.message_type {
        MessageType::Error => ("⚠ Error", theme.error_style(), theme.error_style()),
        MessageType::Help => (
            "💡 Keyboard Shortcuts",
            theme.info_style(),
            theme.info_style(),
        ),
        MessageType::Status => ("ℹ Status", theme.info_style(), theme.info_style()),
        MessageType::Info => ("📢 Message", theme.warning_style(), theme.warning_style()),
    };

    let overlay = Paragraph::new(message.content.as_str())
        .block(
            Block::default()
                .borders(Borders::ALL)
                .title(title)
                .border_style(border_style)
                .title_style(theme.title_style()),
        )
        .style(
            Style::default()
                .bg(theme.background)
                .fg(style.fg.unwrap_or(theme.foreground)),
        )
        .alignment(Alignment::Center)
        .wrap(Wrap { trim: true });

    let area = frame.area();
    let message_lines = message.content.lines().count().max(1) as u16;
    let overlay_height = (message_lines + 2).min(area.height / 2);
    let overlay_width = (message.content.len() as u16 + 4).min(area.width * 3 / 4);

    let overlay_area = Rect {
        x: (area.width - overlay_width) / 2,
        y: (area.height - overlay_height) / 2,
        width: overlay_width,
        height: overlay_height,
    };

    frame.render_widget(Clear, overlay_area);
    frame.render_widget(overlay, overlay_area);
}
