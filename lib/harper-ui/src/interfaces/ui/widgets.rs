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

use super::app::{AppState, SessionInfo, TuiApp};
use super::theme::Theme;
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
    match &app.state {
        AppState::Menu(selected) => draw_menu(frame, *selected, theme),
        AppState::Chat(chat_state) => {
            // Create layout for chat: messages area and input area
            let chunks = Layout::default()
                .direction(Direction::Vertical)
                .constraints([
                    Constraint::Min(5),    // Messages area
                    Constraint::Length(3), // Input area
                ])
                .split(frame.area());

            // Messages area
            let safe_scroll_offset = chat_state.scroll_offset.min(chat_state.messages.len());
            let displayed_messages = &chat_state.messages[safe_scroll_offset..];
            let message_lines: Vec<Line> = displayed_messages
                .iter()
                .flat_map(|msg| {
                    let color = match msg.role.as_str() {
                        "user" => theme.input,
                        "assistant" => theme.output,
                        _ => theme.foreground,
                    };
                    msg.content
                        .lines()
                        .map(|line| Line::styled(line, color))
                        .collect::<Vec<_>>()
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
        AppState::Sessions(sessions, selected) => draw_sessions(frame, sessions, *selected, theme),
        AppState::ExportSessions(sessions, selected) => {
            draw_export_sessions(frame, sessions, *selected, theme)
        }
        AppState::Tools(selected) => draw_tools(frame, *selected, theme),
        AppState::ViewSession(name, messages, selected) => {
            draw_view_session(frame, name, messages, *selected, theme)
        }
    }

    // Draw status bar
    draw_status_bar(frame, app, theme);

    // Draw message overlay if present
    if let Some(msg) = &app.message {
        draw_message_overlay(frame, msg, theme);
    }
}

fn draw_menu(frame: &mut Frame, selected: usize, theme: &Theme) {
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
                .title("Harper AI Assistant")
                .border_style(theme.border_style())
                .title_style(theme.title_style()),
        )
        .highlight_style(Style::default().add_modifier(Modifier::BOLD));

    frame.render_widget(menu, frame.area());
}

fn draw_sessions(frame: &mut Frame, sessions: &[SessionInfo], selected: usize, theme: &Theme) {
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

    frame.render_widget(sessions_list, frame.area());
}

fn draw_export_sessions(
    frame: &mut Frame,
    sessions: &[SessionInfo],
    selected: usize,
    theme: &Theme,
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

    let export_list = List::new(items)
        .block(
            Block::default()
                .borders(Borders::ALL)
                .title("Select Session to Export")
                .border_style(theme.border_style())
                .title_style(theme.title_style()),
        )
        .highlight_style(Style::default().add_modifier(Modifier::BOLD));

    frame.render_widget(export_list, frame.area());
}

fn draw_tools(frame: &mut Frame, selected: usize, theme: &Theme) {
    let tool_items = [
        "File Operations",
        "Git Commands",
        "Web Search",
        "Shell Commands",
        "Back to Menu",
    ];

    let items: Vec<ListItem> = tool_items
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

    let tools = List::new(items)
        .block(
            Block::default()
                .borders(Borders::ALL)
                .title("Available Tools")
                .border_style(theme.border_style())
                .title_style(theme.title_style()),
        )
        .highlight_style(Style::default().add_modifier(Modifier::BOLD));

    frame.render_widget(tools, frame.area());
}

fn draw_view_session(
    frame: &mut Frame,
    name: &str,
    messages: &[harper_core::core::Message],
    _selected: usize,
    theme: &Theme,
) {
    let message_lines: Vec<Line> = messages
        .iter()
        .map(|msg| {
            let color = match msg.role.as_str() {
                "user" => theme.input,
                "assistant" => theme.output,
                _ => theme.foreground,
            };
            Line::from(Span::styled(
                format!("[{}] {}", msg.role, msg.content),
                Style::default().fg(color),
            ))
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

    frame.render_widget(view, frame.area());
}

fn draw_status_bar(frame: &mut Frame, app: &TuiApp, theme: &Theme) {
    let mode = match &app.state {
        AppState::Menu(_) => "MENU",
        AppState::Chat(..) => "CHAT",
        AppState::Sessions(_, _) => "SESSIONS",
        AppState::ExportSessions(_, _) => "EXPORT",
        AppState::Tools(_) => "TOOLS",
        AppState::ViewSession(_, _, _) => "VIEW",
    };

    // Enhanced status with provider info and shortcuts
    let left_status = format!(" {} ", mode);
    let center_status = "Harper AI Agent";
    let right_status = " F1/Ctrl+H:Help | Ctrl+C:Quit ";

    let area = frame.area();
    let status_area = Rect {
        x: 0,
        y: area.height - 1,
        width: area.width,
        height: 1,
    };

    // Left section
    let left_width = left_status.len() as u16;
    let left_area = Rect {
        x: 0,
        y: status_area.y,
        width: left_width,
        height: 1,
    };

    let left_widget = Paragraph::new(left_status).style(theme.highlight_style().bg(theme.accent));
    frame.render_widget(left_widget, left_area);

    // Center section
    let center_width = center_status.len() as u16;
    let center_x = (area.width - center_width) / 2;
    let center_area = Rect {
        x: center_x,
        y: status_area.y,
        width: center_width,
        height: 1,
    };

    let center_widget = Paragraph::new(center_status).style(
        Style::default()
            .bg(theme.accent)
            .fg(theme.background)
            .add_modifier(Modifier::BOLD),
    );
    frame.render_widget(center_widget, center_area);

    // Right section
    let right_width = right_status.len() as u16;
    let right_area = Rect {
        x: area.width - right_width,
        y: status_area.y,
        width: right_width,
        height: 1,
    };

    let right_widget = Paragraph::new(right_status).style(theme.muted_style().bg(theme.accent));
    frame.render_widget(right_widget, right_area);

    // Fill remaining space
    let fill_start = left_width;
    let fill_end = center_x;
    if fill_end > fill_start {
        let fill_area = Rect {
            x: fill_start,
            y: status_area.y,
            width: fill_end - fill_start,
            height: 1,
        };
        let fill_widget = Paragraph::new("").style(Style::default().bg(theme.accent));
        frame.render_widget(fill_widget, fill_area);
    }

    let fill_start2 = center_x + center_width;
    let fill_end2 = area.width - right_width;
    if fill_end2 > fill_start2 {
        let fill_area2 = Rect {
            x: fill_start2,
            y: status_area.y,
            width: fill_end2 - fill_start2,
            height: 1,
        };
        let fill_widget2 = Paragraph::new("").style(Style::default().bg(theme.accent));
        frame.render_widget(fill_widget2, fill_area2);
    }
}

fn draw_message_overlay(frame: &mut Frame, message: &str, theme: &Theme) {
    // Determine message type and styling
    let (title, style, border_style) = if message.starts_with("Error") || message.contains("error")
    {
        ("⚠ Error", theme.error_style(), theme.error_style())
    } else if message.starts_with("F1:Help") || message.contains("Help") {
        (
            "💡 Keyboard Shortcuts",
            theme.info_style(),
            theme.info_style(),
        )
    } else if message.contains("enabled") || message.contains("disabled") {
        ("ℹ Status", theme.info_style(), theme.info_style())
    } else {
        ("📢 Message", theme.warning_style(), theme.warning_style())
    };

    let overlay = Paragraph::new(message)
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
    let message_lines = message.lines().count().max(1) as u16;
    let overlay_height = (message_lines + 2).min(area.height / 2);
    let overlay_width = (message.len() as u16 + 4).min(area.width * 3 / 4);

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
        let content = "Hello world";
        let spans = parse_content_with_code(
            &syntax_set,
            &theme_set,
            content,
            Color::White,
            "base16-ocean.dark",
        );
        assert_eq!(spans.len(), 1);
        assert_eq!(spans[0].content, "Hello world");
    }

    #[test]
    fn test_parse_content_with_code_with_code_block() {
        let (syntax_set, theme_set) = setup();
        let content = "Before ```rust\nfn main() {}\n``` After";
        let spans = parse_content_with_code(
            &syntax_set,
            &theme_set,
            content,
            Color::White,
            "base16-ocean.dark",
        );
        // Should have spans for "Before ", highlighted code, " After"
        assert!(spans.len() > 1);
        // First span plain
        assert_eq!(spans[0].content, "Before ");
        // Then highlighted spans
    }

    #[test]
    fn test_parse_content_with_code_unclosed_code_block() {
        let (syntax_set, theme_set) = setup();
        let content = "Text ```code";
        let spans = parse_content_with_code(
            &syntax_set,
            &theme_set,
            content,
            Color::White,
            "base16-ocean.dark",
        );
        // Should treat as plain text before and the unclosed block as plain
        assert_eq!(spans.len(), 2, "Should have two spans for unclosed block");
        assert_eq!(spans[0].content, "Text ");
        assert_eq!(spans[1].content, "```code");
    }

    #[test]
    fn test_parse_content_with_code_multiple_blocks() {
        let (syntax_set, theme_set) = setup();
        let content = "```js\nconsole.log()\n``` and ```python\nprint()\n```";
        let spans = parse_content_with_code(
            &syntax_set,
            &theme_set,
            content,
            Color::White,
            "base16-ocean.dark",
        );
        assert!(spans.len() > 2);
    }
}
