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

// Note: We use syntect for syntax highlighting due to its comprehensive language support
// and theme system. The yaml-rust dependency is unmaintained but poses no security risk
// as it's only used for YAML theme file parsing. syntect is actively maintained and
// provides the best syntax highlighting experience. This warning is acknowledged and
// documented in deny.toml and CI configuration.

use ratatui::style::{Color, Style};
use ratatui::text::Span;
use syntect::easy::HighlightLines;
use syntect::highlighting::{Style as SynStyle, ThemeSet};
use syntect::parsing::SyntaxSet;
use syntect::util::LinesWithEndings;

pub fn highlight_code(
    syntax_set: &SyntaxSet,
    theme_set: &ThemeSet,
    language: &str,
    code: &str,
    theme_name: &str,
) -> Vec<Span<'static>> {
    let syntax = syntax_set
        .find_syntax_by_extension(language)
        .unwrap_or_else(|| syntax_set.find_syntax_plain_text());

    let theme = theme_set
        .themes
        .get(theme_name)
        .unwrap_or(&theme_set.themes["base16-ocean.dark"]);
    let mut highlighter = HighlightLines::new(syntax, theme);

    LinesWithEndings::from(code)
        .flat_map(|line| {
            let ranges: Vec<(SynStyle, &str)> = highlighter
                .highlight_line(line, syntax_set)
                .unwrap_or_default();

            ranges
                .into_iter()
                .map(|(style, text)| {
                    let color = syntect_to_ratatui_color(style.foreground);
                    Span::styled(text.to_string(), Style::default().fg(color))
                })
                .collect::<Vec<Span>>()
        })
        .collect()
}

fn syntect_to_ratatui_color(color: syntect::highlighting::Color) -> Color {
    Color::Rgb(color.r, color.g, color.b)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn setup() -> (SyntaxSet, ThemeSet) {
        (
            SyntaxSet::load_defaults_newlines(),
            ThemeSet::load_defaults(),
        )
    }

    #[test]
    fn test_highlight_code_rust() {
        let (syntax_set, theme_set) = setup();
        let code = "fn main() {\n    println!(\"Hello\");\n}";
        let spans = highlight_code(&syntax_set, &theme_set, "rs", code, "base16-ocean.dark");
        assert!(!spans.is_empty());
        // Check that spans contain expected text
        let all_text: String = spans
            .iter()
            .map(|s| s.content.clone())
            .collect::<Vec<_>>()
            .join("");
        assert!(all_text.contains("fn main()"));
        assert!(all_text.contains("println!"));
    }

    #[test]
    fn test_highlight_code_unknown_language() {
        let (syntax_set, theme_set) = setup();
        let code = "print('hello')";
        let spans = highlight_code(
            &syntax_set,
            &theme_set,
            "unknown",
            code,
            "base16-ocean.dark",
        );
        assert!(!spans.is_empty());
        // Should fall back to plain text syntax
    }

    #[test]
    fn test_highlight_code_empty() {
        let (syntax_set, theme_set) = setup();
        let spans = highlight_code(&syntax_set, &theme_set, "rs", "", "base16-ocean.dark");
        assert!(spans.is_empty());
    }
}
