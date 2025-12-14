use ratatui::prelude::*;
use ratatui::widgets::{Block, Borders, Clear, List, ListItem, Paragraph, Wrap};

use super::app::{AppState, SessionInfo, TuiApp};

pub fn draw(frame: &mut Frame, app: &TuiApp) {
    match &app.state {
        AppState::Menu(selected) => draw_menu(frame, *selected),
        AppState::Chat(_, messages, input, _, web_search_enabled) => {
            draw_chat(frame, messages, input, *web_search_enabled)
        }
        AppState::Sessions(sessions, selected) => draw_sessions(frame, sessions, *selected),
        AppState::Tools(selected) => draw_tools(frame, *selected),
        AppState::ViewSession(name, messages, selected) => {
            draw_view_session(frame, name, messages, *selected)
        }
    }

    // Draw status bar
    draw_status_bar(frame, app);

    // Draw message overlay if any
    if let Some(msg) = &app.message {
        draw_message_overlay(frame, msg);
    }
}

fn draw_menu(frame: &mut Frame, selected: usize) {
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
                Style::default().bg(Color::Blue).fg(Color::White)
            } else {
                Style::default()
            };
            ListItem::new(*item).style(style)
        })
        .collect();

    let menu = List::new(items)
        .block(
            Block::default()
                .borders(Borders::ALL)
                .title("Harper AI Agent"),
        )
        .highlight_style(Style::default().add_modifier(Modifier::BOLD));

    frame.render_widget(menu, frame.area());
}

fn draw_chat(
    frame: &mut Frame,
    messages: &[crate::core::Message],
    input: &str,
    web_search_enabled: bool,
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
                "user" => Color::Blue,
                "assistant" => Color::Green,
                _ => Color::White,
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
        .block(Block::default().borders(Borders::ALL).title(title))
        .wrap(Wrap { trim: false });

    frame.render_widget(messages_widget, chunks[0]);

    // Input area
    let input_widget = Paragraph::new(format!("> {}", input))
        .block(Block::default().borders(Borders::ALL).title("Input"))
        .style(Style::default().bg(Color::DarkGray));

    frame.render_widget(input_widget, chunks[1]);
}

fn draw_sessions(frame: &mut Frame, sessions: &[SessionInfo], selected: usize) {
    let items: Vec<ListItem> = sessions
        .iter()
        .enumerate()
        .map(|(i, session)| {
            let style = if i == selected {
                Style::default().bg(Color::Blue).fg(Color::White)
            } else {
                Style::default()
            };
            ListItem::new(format!("{} - {}", session.name, session.created_at)).style(style)
        })
        .collect();

    let sessions_list = List::new(items)
        .block(Block::default().borders(Borders::ALL).title("Sessions"))
        .highlight_style(Style::default().add_modifier(Modifier::BOLD));

    frame.render_widget(sessions_list, frame.area());
}

fn draw_tools(frame: &mut Frame, selected: usize) {
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
                Style::default().bg(Color::Blue).fg(Color::White)
            } else {
                Style::default()
            };
            ListItem::new(*item).style(style)
        })
        .collect();

    let tools = List::new(items)
        .block(
            Block::default()
                .borders(Borders::ALL)
                .title("Available Tools"),
        )
        .highlight_style(Style::default().add_modifier(Modifier::BOLD));

    frame.render_widget(tools, frame.area());
}

fn draw_view_session(
    frame: &mut Frame,
    name: &str,
    messages: &[crate::core::Message],
    _selected: usize,
) {
    let message_lines: Vec<Line> = messages
        .iter()
        .map(|msg| {
            let color = match msg.role.as_str() {
                "user" => Color::Blue,
                "assistant" => Color::Green,
                _ => Color::White,
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
                .title(format!("Session: {}", name)),
        )
        .wrap(Wrap { trim: false });

    frame.render_widget(view, frame.area());
}

fn draw_status_bar(frame: &mut Frame, app: &TuiApp) {
    let mode = match &app.state {
        AppState::Menu(_) => "MENU",
        AppState::Chat(_, _, _, _, _) => "CHAT",
        AppState::Sessions(_, _) => "SESSIONS",
        AppState::Tools(_) => "TOOLS",
        AppState::ViewSession(_, _, _) => "VIEW",
    };

    let status = format!(" {} | Harper AI Agent ", mode);
    let status_bar = Paragraph::new(status)
        .style(Style::default().bg(Color::Blue).fg(Color::White))
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

fn draw_message_overlay(frame: &mut Frame, message: &str) {
    let overlay = Paragraph::new(message)
        .block(Block::default().borders(Borders::ALL).title("Message"))
        .style(Style::default().bg(Color::Black).fg(Color::Red))
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
