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
use ratatui::style::{Color, Style};

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
    #[allow(dead_code)]
    pub success: Color,
}

impl Default for Theme {
    fn default() -> Self {
        Self {
            background: Color::Black,
            foreground: Color::White,
            accent: Color::Blue,
            border: Color::Gray,
            title: Color::Yellow,
            input: Color::Cyan,
            output: Color::Green,
            error: Color::Red,
            success: Color::Green,
        }
    }
}

impl Theme {
    pub fn dark() -> Self {
        Self::default()
    }

    pub fn light() -> Self {
        Self {
            background: Color::White,
            foreground: Color::Black,
            accent: Color::Blue,
            border: Color::Gray,
            title: Color::DarkGray,
            input: Color::Blue,
            output: Color::Green,
            error: Color::Red,
            success: Color::Green,
        }
    }

    pub fn from_name(name: &str) -> Self {
        match name {
            "dark" => Self::dark(),
            "light" => Self::light(),
            _ => Self::default(),
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
}
