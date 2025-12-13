use super::app::{AppState, TuiApp};
use crossterm::cursor::MoveTo;
use crossterm::execute;
use crossterm::style::{Color, Print, ResetColor, SetForegroundColor};
use crossterm::terminal::{Clear, ClearType};
use std::io::Write;

pub fn draw(stdout: &mut impl Write, app: &TuiApp) -> Result<(), crate::core::error::HarperError> {
    execute!(stdout, Clear(ClearType::All), MoveTo(0, 0))?;

    // Title
    execute!(
        stdout,
        SetForegroundColor(Color::Cyan),
        Print("Harper AI Agent"),
        ResetColor,
        MoveTo(0, 1)
    )?;

    let start_y = 3;
    let (_width, height) = crossterm::terminal::size().unwrap_or((80, 24));
    let bottom_y = height.saturating_sub(1);
    let visible_items = (bottom_y - start_y) as usize;

    match &app.state {
        AppState::Menu(sel) => {
            execute!(stdout, MoveTo(0, 2), Print("Menu:"), MoveTo(0, start_y))?;
            for (i, item) in app.menu_items.iter().enumerate() {
                if i == *sel {
                    execute!(
                        stdout,
                        SetForegroundColor(Color::Yellow),
                        Print(format!(">> {}", item.text())),
                        ResetColor
                    )?;
                } else {
                    execute!(stdout, Print(format!("   {}", item.text())))?;
                }
                execute!(stdout, MoveTo(0, start_y + i as u16 + 1))?;
            }
        }
        AppState::ListSessions(sessions, sel, offset) => {
            execute!(stdout, MoveTo(0, 2), Print("Sessions:"), MoveTo(0, start_y))?;
            for (i, session) in sessions
                .iter()
                .enumerate()
                .skip(*offset)
                .take(visible_items)
            {
                let display_i = (i - *offset) as u16;
                let text = format!("{} ({})", session.id, session.created_at);
                if i == *sel {
                    execute!(
                        stdout,
                        SetForegroundColor(Color::Yellow),
                        Print(format!(">> {}", text)),
                        ResetColor
                    )?;
                } else {
                    execute!(stdout, Print(format!("   {}", text)))?;
                }
                execute!(stdout, MoveTo(0, start_y + display_i + 1))?;
            }
        }
        AppState::ViewSession(session_id, history, sel, offset) => {
            execute!(
                stdout,
                MoveTo(0, 2),
                Print(format!("Session: {}", session_id)),
                MoveTo(0, start_y)
            )?;
            for (i, msg) in history.iter().enumerate().skip(*offset).take(visible_items) {
                let display_i = (i - *offset) as u16;
                let color = match msg.role.as_str() {
                    "user" => Color::Blue,
                    "assistant" => Color::Green,
                    _ => Color::White,
                };
                let text = format!("{}: {}", msg.role, msg.content);
                if i == *sel {
                    execute!(
                        stdout,
                        SetForegroundColor(Color::Yellow),
                        Print(format!(">> {}", text)),
                        ResetColor
                    )?;
                } else {
                    execute!(
                        stdout,
                        SetForegroundColor(color),
                        Print(format!("   {}", text)),
                        ResetColor
                    )?;
                }
                execute!(stdout, MoveTo(0, start_y + display_i + 1))?;
            }
        }
        AppState::PromptWebSearch => {
            execute!(
                stdout,
                MoveTo(0, 2),
                Print("Enable web search for this session? (y/n)")
            )?;
        }
        AppState::Chat(history, offset, input, _, _) => {
            execute!(stdout, MoveTo(0, 2), Print("Chat:"), MoveTo(0, start_y))?;
            let mut display_i: u16 = 0;
            for msg in history.iter().skip(*offset) {
                if msg.role == "system" {
                    continue;
                }
                if display_i as usize >= visible_items {
                    break;
                }
                let color = match msg.role.as_str() {
                    "user" => Color::Blue,
                    "assistant" => Color::Green,
                    _ => Color::White,
                };
                let text = format!("{}: {}", msg.role, msg.content);
                execute!(stdout, SetForegroundColor(color), Print(text), ResetColor)?;
                execute!(stdout, MoveTo(0, start_y + display_i + 1))?;
                display_i += 1;
            }
            let input_y = bottom_y - 1;
            execute!(stdout, MoveTo(0, input_y), Print(format!("You: {}", input)))?;
        }
    }

    // Help or message
    execute!(stdout, MoveTo(0, bottom_y))?;
    if let Some(ref message) = app.message {
        execute!(
            stdout,
            SetForegroundColor(Color::Red),
            Print(format!("Message: {}", message)),
            ResetColor
        )?;
    } else {
        let help = match &app.state {
            AppState::Menu(_) => "Use ↑/↓ or j/k to navigate, Enter to select, q/5 to quit",
            AppState::PromptWebSearch => "Press y or n to enable/disable web search",
            AppState::Chat(_, _, _, _, _) => {
                "Type message, Enter to send, ↑/↓ to scroll, q to quit"
            }
            AppState::ListSessions(_, _, _) => {
                "Use ↑/↓ or j/k to navigate, Enter to view, Esc to back"
            }
            AppState::ViewSession(_, _, _, _) => "Use ↑/↓ or j/k to scroll, Esc to back",
        };
        execute!(
            stdout,
            SetForegroundColor(Color::Grey),
            Print(help),
            ResetColor
        )?;
    }

    stdout.flush()?;
    Ok(())
}
