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

#[allow(dead_code)]
use ratatui::style::{Color, Modifier, Style};

#[derive(Debug, Clone)]
pub struct Theme {
    pub background: Color,
    pub foreground: Color,
    pub accent: Color,
    pub border: Color,
    pub title: Color,
    pub input: Color,
    pub output: Color,
    pub error: Color,
    pub success: Color,
    pub warning: Color,
    pub info: Color,
    pub muted: Color,
    pub highlight: Color,
    pub selection: Color,
    pub syntax_theme: String,
}

impl Default for Theme {
    fn default() -> Self {
        Self {
            background: Color::Black,
            foreground: Color::White,
            accent: Color::Rgb(88, 166, 255),    // Modern blue
            border: Color::Rgb(64, 64, 64),      // Subtle gray
            title: Color::Rgb(255, 215, 0),      // Gold
            input: Color::Rgb(0, 255, 255),      // Cyan
            output: Color::Rgb(144, 238, 144),   // Light green
            error: Color::Rgb(255, 99, 71),      // Tomato red
            success: Color::Rgb(50, 205, 50),    // Lime green
            warning: Color::Rgb(255, 165, 0),    // Orange
            info: Color::Rgb(135, 206, 235),     // Sky blue
            muted: Color::Rgb(128, 128, 128),    // Gray
            highlight: Color::Rgb(255, 255, 0),  // Yellow
            selection: Color::Rgb(70, 130, 180), // Steel blue
            syntax_theme: "base16-ocean.dark".to_string(),
        }
    }
}

impl Theme {
    pub fn dark() -> Self {
        Self { ..Self::default() }
    }

    pub fn light() -> Self {
        Self {
            background: Color::White,
            foreground: Color::Black,
            accent: Color::Rgb(0, 122, 255),     // iOS blue
            border: Color::Rgb(200, 200, 200),   // Light gray
            title: Color::Rgb(88, 86, 214),      // Purple
            input: Color::Rgb(0, 122, 255),      // Blue
            output: Color::Rgb(40, 167, 69),     // Green
            error: Color::Rgb(220, 53, 69),      // Red
            success: Color::Rgb(40, 167, 69),    // Green
            warning: Color::Rgb(255, 193, 7),    // Amber
            info: Color::Rgb(23, 162, 184),      // Teal
            muted: Color::Rgb(108, 117, 125),    // Muted gray
            highlight: Color::Rgb(255, 235, 59), // Light yellow
            selection: Color::Rgb(0, 123, 255),  // Primary blue
            syntax_theme: "base16-ocean.light".to_string(),
        }
    }

    pub fn from_name(name: &str) -> Self {
        match name {
            "dark" => Self::dark(),
            "light" => Self::light(),
            "github" => Self::github(),
            _ => Self::default(),
        }
    }

    pub fn github() -> Self {
        Self {
            background: Color::Rgb(13, 17, 23),    // GitHub dark bg
            foreground: Color::Rgb(230, 237, 243), // GitHub text
            accent: Color::Rgb(33, 136, 255),      // GitHub blue
            border: Color::Rgb(48, 54, 61),        // GitHub border
            title: Color::Rgb(125, 196, 228),      // GitHub cyan
            input: Color::Rgb(33, 136, 255),       // GitHub blue
            output: Color::Rgb(63, 185, 80),       // GitHub green
            error: Color::Rgb(248, 81, 73),        // GitHub red
            success: Color::Rgb(63, 185, 80),      // GitHub green
            warning: Color::Rgb(219, 154, 4),      // GitHub yellow
            info: Color::Rgb(125, 196, 228),       // GitHub cyan
            muted: Color::Rgb(139, 148, 158),      // GitHub muted
            highlight: Color::Rgb(255, 223, 93),   // GitHub highlight
            selection: Color::Rgb(58, 117, 215),   // GitHub selection
            syntax_theme: "base16-ocean.dark".to_string(),
        }
    }

    #[allow(dead_code)]
    pub fn input_style(&self) -> Style {
        Style::default().fg(self.input)
    }

    #[allow(dead_code)]
    pub fn output_style(&self) -> Style {
        Style::default().fg(self.output)
    }

    #[allow(dead_code)]
    pub fn error_style(&self) -> Style {
        Style::default().fg(self.error)
    }

    pub fn title_style(&self) -> Style {
        Style::default().fg(self.title)
    }

    pub fn border_style(&self) -> Style {
        Style::default().fg(self.border)
    }

    #[allow(dead_code)]
    pub fn accent_style(&self) -> Style {
        Style::default().fg(self.accent)
    }

    pub fn warning_style(&self) -> Style {
        Style::default().fg(self.warning)
    }

    pub fn info_style(&self) -> Style {
        Style::default().fg(self.info)
    }

    pub fn muted_style(&self) -> Style {
        Style::default().fg(self.muted)
    }

    pub fn highlight_style(&self) -> Style {
        Style::default()
            .fg(self.highlight)
            .add_modifier(Modifier::BOLD)
    }

    pub fn selection_style(&self) -> Style {
        Style::default().bg(self.selection).fg(self.background)
    }
}
