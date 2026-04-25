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

use super::app::{AppState, ApprovalState, SessionInfo, TuiApp, UiMessage};
use super::theme::Theme;

// Refined shortcuts for a cleaner footer
const FOOTER_SHORTCUTS: [[(&str, &str); 6]; 2] = [
    [
        ("G", "Help"),
        ("O", "Export"),
        ("W", "Search"),
        ("K", "Cut"),
        ("B", "Sidebar"),
        ("C", "ID"),
    ],
    [
        ("X", "Exit"),
        ("R", "Load"),
        ("U", "Paste"),
        ("T", "Enter"),
        ("Y", "Prev"),
        ("V", "Next"),
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

            let chunks = Layout::default()
                .direction(Direction::Vertical)
                .constraints([Constraint::Min(5), Constraint::Length(3)])
                .split(chat_area);

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

            let chat_block = Block::default()
                .borders(Borders::NONE) // No noise
                .padding(Padding::uniform(1));

            let messages_widget = Paragraph::new(message_lines)
                .block(chat_block)
                .wrap(Wrap { trim: false });

            frame.render_widget(messages_widget, chunks[0]);

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

    draw_zen_footer(frame, app, theme, footer_area);

    if let Some(approval) = &app.pending_approval {
        draw_approval(frame, approval, theme);
    }

    if let Some(msg) = &app.message {
        draw_message_overlay(frame, msg, theme);
    }
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
    let menu_items = ["New Conversation", "History", "Export", "Settings", "Quit"];

    let area = centered_rect(40, 30, area);

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

    let col_width = area.width / 6;
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
        "{}\n\n{}\n\n[Y] Approve  [N] Reject",
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
        .title(" Security ")
        .title_style(theme.warning_style())
        .style(Style::default().bg(theme.background));

    let paragraph = Paragraph::new(content)
        .block(block)
        .alignment(Alignment::Center)
        .wrap(Wrap { trim: true })
        .scroll((state.scroll_offset, 0));

    frame.render_widget(Clear, overlay_area);
    frame.render_widget(paragraph, overlay_area);
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
                Style::default()
                    .fg(theme.accent)
                    .add_modifier(Modifier::BOLD)
            } else {
                Style::default().fg(theme.foreground)
            };
            ListItem::new(format!("{} › {}", session.name, session.created_at)).style(style)
        })
        .collect();

    let sessions_list = List::new(items).block(
        Block::default()
            .title(" Sessions ")
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
            ListItem::new(format!("Export › {}", session.name)).style(style)
        })
        .collect();

    let sessions_list = List::new(items).block(
        Block::default()
            .title(" Export ")
            .padding(Padding::uniform(1)),
    );

    frame.render_widget(sessions_list, area);
}

fn draw_tools(frame: &mut Frame, selected: usize, theme: &Theme, area: Rect) {
    let tools = ["Search", "System", "Processes", "Git"];
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
            .title(" Tools ")
            .padding(Padding::uniform(1)),
    );

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
            let default_color = if msg.role == "user" {
                theme.input
            } else {
                theme.output
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
                .title(format!(" {} ", name))
                .padding(Padding::uniform(1)),
        )
        .wrap(Wrap { trim: false });

    frame.render_widget(view, area);
}

fn draw_message_overlay(frame: &mut Frame, message: &UiMessage, theme: &Theme) {
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

    let area = frame.area();
    let message_lines = message.content.lines().count().max(1) as u16;
    let overlay_height = (message_lines + 4).min(area.height / 2);
    let overlay_width = (message.content.len() as u16 + 8).min(area.width * 3 / 4);

    let overlay_area = Rect {
        x: (area.width - overlay_width) / 2,
        y: (area.height - overlay_height) / 2,
        width: overlay_width,
        height: overlay_height,
    };

    frame.render_widget(Clear, overlay_area);
    frame.render_widget(overlay, overlay_area);
}

#[cfg(test)]
mod tests {
    use super::*;
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
}
