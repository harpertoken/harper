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

use std::io::{self, Write};

use crossterm::{cursor, queue};
use ratatui::layout::Rect;

const LOGO_PNG: &[u8] = include_bytes!("../../../assets/harper-menu-logo.png");
const LOGO_PIXEL_WIDTH: u32 = 560;
const LOGO_PIXEL_HEIGHT: u32 = 338;
const KITTY_CHUNK_SIZE: usize = 4096;

#[derive(Debug, Clone, Copy, Eq, PartialEq)]
pub(crate) enum TerminalImageProtocol {
    Kitty,
    Iterm2,
}

pub(crate) fn supported_protocol() -> Option<TerminalImageProtocol> {
    let override_value = std::env::var("HARPER_TERMINAL_IMAGE_PROTOCOL").ok();
    match override_value
        .as_deref()
        .map(str::to_ascii_lowercase)
        .as_deref()
    {
        Some("kitty") | Some("wezterm") => return Some(TerminalImageProtocol::Kitty),
        Some("iterm") | Some("iterm2") => return Some(TerminalImageProtocol::Iterm2),
        Some("off") | Some("none") | Some("false") | Some("0") => return None,
        _ => {}
    }

    let term = std::env::var("TERM")
        .unwrap_or_default()
        .to_ascii_lowercase();
    let term_program = std::env::var("TERM_PROGRAM")
        .unwrap_or_default()
        .to_ascii_lowercase();

    if std::env::var_os("KITTY_WINDOW_ID").is_some()
        || term_program.contains("wezterm")
        || term.contains("kitty")
    {
        Some(TerminalImageProtocol::Kitty)
    } else if term_program.contains("iterm") {
        Some(TerminalImageProtocol::Iterm2)
    } else {
        None
    }
}

pub(crate) fn render_logo<W: Write>(
    writer: &mut W,
    area: Rect,
    protocol: TerminalImageProtocol,
) -> io::Result<()> {
    if area.width == 0 || area.height == 0 {
        return Ok(());
    }

    queue!(writer, cursor::MoveTo(area.x, area.y))?;
    match protocol {
        TerminalImageProtocol::Kitty => render_kitty_logo(writer, area),
        TerminalImageProtocol::Iterm2 => render_iterm2_logo(writer, area),
    }
}

pub(crate) fn clear_images<W: Write>(
    writer: &mut W,
    protocol: TerminalImageProtocol,
) -> io::Result<()> {
    match protocol {
        TerminalImageProtocol::Kitty => write!(writer, "\x1b_Ga=d,d=A\x1b\\"),
        TerminalImageProtocol::Iterm2 => Ok(()),
    }
}

fn render_kitty_logo<W: Write>(writer: &mut W, area: Rect) -> io::Result<()> {
    let encoded = encode_base64(LOGO_PNG);
    let mut chunks = encoded.as_bytes().chunks(KITTY_CHUNK_SIZE).peekable();
    let mut first = true;

    while let Some(chunk) = chunks.next() {
        let more = if chunks.peek().is_some() { 1 } else { 0 };
        if first {
            write!(
                writer,
                "\x1b_Ga=T,f=100,t=d,q=2,s={},v={},c={},r={},m={};",
                LOGO_PIXEL_WIDTH, LOGO_PIXEL_HEIGHT, area.width, area.height, more
            )?;
            first = false;
        } else {
            write!(writer, "\x1b_Gm={more};")?;
        }
        writer.write_all(chunk)?;
        write!(writer, "\x1b\\")?;
    }

    Ok(())
}

fn render_iterm2_logo<W: Write>(writer: &mut W, area: Rect) -> io::Result<()> {
    let encoded = encode_base64(LOGO_PNG);
    write!(
        writer,
        "\x1b]1337;File=inline=1;width={}ch;height={}ch;preserveAspectRatio=1:{}\x07",
        area.width, area.height, encoded
    )
}

fn encode_base64(bytes: &[u8]) -> String {
    const TABLE: &[u8; 64] = b"ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789+/";
    let mut encoded = String::with_capacity(bytes.len().div_ceil(3) * 4);

    for chunk in bytes.chunks(3) {
        let first = chunk[0];
        let second = chunk.get(1).copied().unwrap_or(0);
        let third = chunk.get(2).copied().unwrap_or(0);

        encoded.push(TABLE[(first >> 2) as usize] as char);
        encoded.push(TABLE[(((first & 0b0000_0011) << 4) | (second >> 4)) as usize] as char);
        if chunk.len() > 1 {
            encoded.push(TABLE[(((second & 0b0000_1111) << 2) | (third >> 6)) as usize] as char);
        } else {
            encoded.push('=');
        }
        if chunk.len() > 2 {
            encoded.push(TABLE[(third & 0b0011_1111) as usize] as char);
        } else {
            encoded.push('=');
        }
    }

    encoded
}

#[cfg(test)]
mod tests {
    use super::{render_logo, TerminalImageProtocol};
    use ratatui::layout::Rect;

    #[test]
    fn kitty_logo_render_uses_graphics_protocol() {
        let mut output = Vec::new();
        render_logo(
            &mut output,
            Rect {
                x: 2,
                y: 3,
                width: 40,
                height: 12,
            },
            TerminalImageProtocol::Kitty,
        )
        .expect("logo renders");

        let rendered = String::from_utf8_lossy(&output);
        assert!(rendered.contains("\x1b_Ga=T"));
        assert!(rendered.contains("f=100"));
        assert!(rendered.contains("c=40"));
        assert!(rendered.contains("r=12"));
    }

    #[test]
    fn iterm2_logo_render_uses_inline_file_protocol() {
        let mut output = Vec::new();
        render_logo(
            &mut output,
            Rect {
                x: 2,
                y: 3,
                width: 40,
                height: 12,
            },
            TerminalImageProtocol::Iterm2,
        )
        .expect("logo renders");

        let rendered = String::from_utf8_lossy(&output);
        assert!(rendered.contains("\x1b]1337;File=inline=1"));
        assert!(rendered.contains("width=40ch"));
        assert!(rendered.contains("height=12ch"));
    }

    #[test]
    fn base64_encoder_handles_padding() {
        assert_eq!(super::encode_base64(b""), "");
        assert_eq!(super::encode_base64(b"f"), "Zg==");
        assert_eq!(super::encode_base64(b"fo"), "Zm8=");
        assert_eq!(super::encode_base64(b"foo"), "Zm9v");
    }
}
