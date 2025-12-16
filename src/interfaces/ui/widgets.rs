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
use ratatui::widgets::{Block, Borders, Clear, List, ListItem, Paragraph, Wrap};

use super::app::{AppState, SessionInfo, TuiApp};
use super::theme::Theme;

pub fn draw(frame: &mut Frame, app: &TuiApp, theme: &Theme) {
    match &app.state {
        AppState::Menu(selected) => draw_menu(frame, *selected, theme),
        AppState::Chat(_, messages, input, _, web_search_enabled, _, _) => {
            draw_chat(frame, messages, input, *web_search_enabled, theme)
        }
        AppState::Sessions(sessions, selected) => draw_sessions(frame, sessions, *selected, theme),
        AppState::Tools(selected) => draw_tools(frame, *selected, theme),
        AppState::ViewSession(name, messages, selected) => {
            draw_view_session(frame, name, messages, *selected, theme)
        }
    }

    // Draw status bar
    draw_status_bar(frame, app, theme);

    // Draw message overlay if any
    if let Some(msg) = &app.message {
        draw_message_overlay(frame, msg, theme);
    }
}

fn draw_menu(frame: &mut Frame, selected: usize, theme: &Theme) {
    let menu_items = [
        "Start Chat",
        "List Sessions",
        "Resume Session",
        "Tools",
        "Export Session",
        "Quit",
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
                .title("Harper AI Agent")
                .border_style(theme.border_style())
                .title_style(theme.title_style()),
        )
        .highlight_style(Style::default().add_modifier(Modifier::BOLD));

    frame.render_widget(menu, frame.area());
}

fn draw_chat(
    frame: &mut Frame,
    messages: &[crate::core::Message],
    input: &str,
    web_search_enabled: bool,
    theme: &Theme,
) {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Min(1), Constraint::Length(3)])
        .split(frame.area());

    // Messages area
    let message_lines: Vec<Line> = messages
        .iter()
        .flat_map(|msg| {
            let color = match msg.role.as_str() {
                "user" => theme.input,
                "assistant" => theme.output,
                _ => theme.foreground,
            };
            let prefix = format!("[{}] ", msg.role.to_uppercase());
            vec![Line::from(Span::styled(
                format!("{}{}", prefix, msg.content),
                Style::default().fg(color),
            ))]
        })
        .collect();

    let title = format!(
        "Chat (Web Search: {})",
        if web_search_enabled { "ON" } else { "OFF" }
    );
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
    let input_widget = Paragraph::new(format!("> {}", input))
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
    messages: &[crate::core::Message],
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
        AppState::Tools(_) => "TOOLS",
        AppState::ViewSession(_, _, _) => "VIEW",
    };

    let status = format!(" {} | Harper AI Agent ", mode);
    let status_bar = Paragraph::new(status)
        .style(Style::default().bg(theme.accent).fg(theme.foreground))
        .alignment(Alignment::Center);

    let area = frame.area();
    let status_area = Rect {
        x: 0,
        y: area.height - 1,
        width: area.width,
        height: 1,
    };

    frame.render_widget(status_bar, status_area);
}

fn draw_message_overlay(frame: &mut Frame, message: &str, theme: &Theme) {
    let overlay = Paragraph::new(message)
        .block(
            Block::default()
                .borders(Borders::ALL)
                .title("Message")
                .border_style(theme.border_style())
                .title_style(theme.title_style()),
        )
        .style(Style::default().bg(theme.background).fg(theme.error))
        .alignment(Alignment::Center);

    let area = frame.area();
    let overlay_area = Rect {
        x: area.width / 4,
        y: area.height / 2 - 2,
        width: area.width / 2,
        height: 5,
    };

    frame.render_widget(Clear, overlay_area);
    frame.render_widget(overlay, overlay_area);
}
