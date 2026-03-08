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

//! Lightweight intent routing for deterministic actions.

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ChangedFilesIntent {
    pub ext: Option<String>,
    pub tracked_only: bool,
    pub since: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum DeterministicIntent {
    ListChangedFiles(ChangedFilesIntent),
}

pub fn route_intent(query: &str) -> Option<DeterministicIntent> {
    let normalized = normalize(query);
    let tokens: Vec<&str> = normalized.split_whitespace().collect();

    if is_changed_files_intent(&normalized, &tokens) {
        return Some(DeterministicIntent::ListChangedFiles(ChangedFilesIntent {
            ext: infer_extension_filter(&tokens),
            tracked_only: tokens.iter().any(|t| *t == "tracked"),
            since: infer_since_filter(&tokens),
        }));
    }

    None
}

fn normalize(input: &str) -> String {
    input
        .to_ascii_lowercase()
        .chars()
        .map(|c| if c.is_ascii_alphanumeric() || c.is_ascii_whitespace() { c } else { ' ' })
        .collect::<String>()
}

fn is_changed_files_intent(normalized: &str, tokens: &[&str]) -> bool {
    let has_file_noun = tokens.iter().any(|t| {
        matches!(
            *t,
            "file" | "files" | "codebase" | "repo" | "repository" | "worktree"
        )
    });
    let has_change_signal = tokens.iter().any(|t| {
        matches!(
            *t,
            "change"
                | "changed"
                | "changing"
                | "modified"
                | "edit"
                | "edited"
                | "editing"
                | "working"
                | "updated"
                | "update"
                | "touch"
                | "touched"
        )
    });

    if has_file_noun && has_change_signal {
        return true;
    }

    normalized.contains("what am i changing")
        || normalized.contains("what i am changing")
        || normalized.contains("files i am changing")
        || normalized.contains("files im changing")
}

fn infer_since_filter(tokens: &[&str]) -> Option<String> {
    if tokens.iter().any(|t| *t == "today") {
        return Some("today".to_string());
    }
    if tokens.iter().any(|t| *t == "yesterday") {
        return Some("yesterday".to_string());
    }
    if tokens.iter().any(|t| *t == "week") {
        return Some("1 week ago".to_string());
    }
    None
}

fn infer_extension_filter(tokens: &[&str]) -> Option<String> {
    if tokens.iter().any(|t| matches!(*t, "rust" | "rs")) {
        return Some("rs".to_string());
    }
    if tokens.iter().any(|t| matches!(*t, "python" | "py")) {
        return Some("py".to_string());
    }
    if tokens.iter().any(|t| matches!(*t, "javascript" | "js")) {
        return Some("js".to_string());
    }
    if tokens.iter().any(|t| matches!(*t, "typescript" | "ts")) {
        return Some("ts".to_string());
    }
    if tokens.iter().any(|t| matches!(*t, "markdown" | "md")) {
        return Some("md".to_string());
    }
    None
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn routes_am_changing_phrase() {
        let intent = route_intent("tell me the files am changing here in the codebase");
        assert!(matches!(
            intent,
            Some(DeterministicIntent::ListChangedFiles(_))
        ));
    }

    #[test]
    fn routes_working_on_phrase() {
        let intent = route_intent("show files i'm working on");
        assert!(matches!(
            intent,
            Some(DeterministicIntent::ListChangedFiles(_))
        ));
    }

    #[test]
    fn routes_new_files_changed_phrase() {
        let intent = route_intent("show me the new files changed");
        assert!(matches!(
            intent,
            Some(DeterministicIntent::ListChangedFiles(_))
        ));
    }

    #[test]
    fn does_not_route_plain_file_read_request() {
        let intent = route_intent("show me the new file gemini.md");
        assert!(intent.is_none());
    }

    #[test]
    fn does_not_route_open_file_request() {
        let intent = route_intent("open gemini.md");
        assert!(intent.is_none());
    }

    #[test]
    fn infers_rust_today_filters() {
        let intent = route_intent("tell me the rust files changed today");
        match intent {
            Some(DeterministicIntent::ListChangedFiles(filters)) => {
                assert_eq!(filters.ext.as_deref(), Some("rs"));
                assert_eq!(filters.since.as_deref(), Some("today"));
            }
            _ => panic!("intent not routed"),
        }
    }
}
