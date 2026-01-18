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

//! Chat interaction module
//!
//! This module handles user input, chat loops, and message processing.

use crate::agent::prompt::PromptBuilder;
use crate::core::cache::{ApiCacheKey, ApiResponseCache};
use crate::core::error::HarperError;
use crate::core::{ApiConfig, Message};
use crate::runtime::config::ExecPolicyConfig;
use crate::tools::ToolService;

use colored::Colorize;
use reqwest::Client;
use rusqlite::Connection;
use rustyline::completion::{Completer, Pair};
use rustyline::error::ReadlineError;
use rustyline::highlight::Highlighter;
use rustyline::hint::Hinter;
use rustyline::validate::Validator;

use rustyline::Editor;
use rustyline::Helper;
use std::collections::HashMap;
use std::fs;
use std::path::Path;
use turul_mcp_client::{McpClient, ResourceContent};

#[derive(Debug)]
enum CommandAction {
    Clear,
    Exit,
    Custom(String),
    Unknown(String),
}

struct FileCompleter;

impl Helper for FileCompleter {}

impl Hinter for FileCompleter {
    type Hint = String;

    fn hint(&self, _line: &str, _pos: usize, _ctx: &rustyline::Context<'_>) -> Option<String> {
        None
    }
}

impl Highlighter for FileCompleter {
    fn highlight<'l>(&self, line: &'l str, _pos: usize) -> std::borrow::Cow<'l, str> {
        std::borrow::Cow::Borrowed(line)
    }
}

impl Validator for FileCompleter {
    fn validate(
        &self,
        _ctx: &mut rustyline::validate::ValidationContext,
    ) -> rustyline::Result<rustyline::validate::ValidationResult> {
        Ok(rustyline::validate::ValidationResult::Valid(None))
    }
}

impl Completer for FileCompleter {
    type Candidate = Pair;

    fn complete(
        &self,
        line: &str,
        pos: usize,
        _ctx: &rustyline::Context<'_>,
    ) -> rustyline::Result<(usize, Vec<Self::Candidate>)> {
        // Find the last @ before the cursor position
        let at_pos = match line[..pos].rfind('@') {
            Some(pos) => pos,
            None => return Ok((pos, Vec::new())), // No @ found, no completion
        };

        let prefix = &line[at_pos + 1..pos];
        let path = Path::new(prefix);

        // Determine directory to read and file prefix to match
        let (dir_to_read, file_prefix) = if prefix.is_empty() {
            (Path::new("."), "")
        } else if prefix.ends_with('/') {
            (path, "")
        } else if prefix == "." {
            (Path::new("."), ".")
        } else if prefix.starts_with('.') && !prefix.contains('/') {
            (Path::new("."), prefix)
        } else {
            let parent = path.parent().unwrap_or_else(|| Path::new("."));
            let file_part = path.file_name().and_then(|n| n.to_str()).unwrap_or("");
            (parent, file_part)
        };

        let mut candidates = Vec::new();

        if let Ok(entries) = fs::read_dir(dir_to_read) {
            for entry in entries.flatten() {
                if let Some(name) = entry.file_name().to_str() {
                    if name.starts_with(file_prefix) {
                        let mut completion = prefix.to_string();
                        if prefix.is_empty() || prefix.ends_with('/') {
                            completion.push_str(name);
                        } else {
                            // Replace the file part
                            if let Some(parent) = path.parent() {
                                let parent_str = parent.to_string_lossy();
                                if parent_str.is_empty() {
                                    completion = name.to_string();
                                } else {
                                    completion = format!("{}/{}", parent_str, name);
                                }
                            } else {
                                completion = name.to_string();
                            }
                        }
                        if entry.file_type().map(|ft| ft.is_dir()).unwrap_or(false) {
                            completion.push('/');
                        }
                        candidates.push(Pair {
                            display: completion.clone(),
                            replacement: completion,
                        });
                    }
                }
            }
        }

        let start = at_pos + 1;
        Ok((start, candidates))
    }
}

/// Chat service for handling conversations
pub struct ChatService<'a> {
    conn: &'a Connection,
    config: &'a ApiConfig,
    api_cache: Option<&'a mut ApiResponseCache>,
    #[allow(dead_code)]
    mcp_client: Option<&'a McpClient>,
    #[allow(dead_code)]
    todos: Vec<String>,
    prompt_id: Option<String>,
    custom_commands: HashMap<String, String>,
    exec_policy: ExecPolicyConfig,
}

#[allow(dead_code)]
impl<'a> ChatService<'a> {
    /// Create a new chat service
    pub fn new(
        conn: &'a Connection,
        config: &'a ApiConfig,
        mcp_client: Option<&'a McpClient>,
        api_cache: Option<&'a mut ApiResponseCache>,
        prompt_id: Option<String>,
        custom_commands: HashMap<String, String>,
        exec_policy: ExecPolicyConfig,
    ) -> Self {
        Self {
            conn,
            config,
            mcp_client,
            api_cache,
            todos: Vec::new(),
            prompt_id,
            custom_commands,
            exec_policy,
        }
    }

    /// Create a new chat service for testing
    #[cfg(test)]
    #[allow(dead_code)]
    pub fn new_test(conn: &'a Connection, config: &'a ApiConfig) -> Self {
        Self {
            conn,
            config,
            mcp_client: None,
            api_cache: None,
            todos: Vec::new(),
            prompt_id: None,
            custom_commands: HashMap::new(),
            exec_policy: ExecPolicyConfig {
                allowed_commands: None,
                blocked_commands: None,
            },
        }
    }

    /// Get available MCP tools as formatted text for system prompt
    /// Start a new interactive chat session
    pub async fn create_session(
        &self,
        web_search_enabled: bool,
    ) -> Result<(Vec<Message>, String), HarperError> {
        let session_id = uuid::Uuid::new_v4().to_string();
        self.save_session(&session_id)?;

        let system_prompt = self.build_system_prompt(web_search_enabled).await;

        let history = vec![Message {
            role: "system".to_string(),
            content: system_prompt,
        }];

        Ok((history, session_id))
    }

    /// Send a message and get response
    pub async fn send_message(
        &mut self,
        user_msg: &str,
        history: &mut Vec<Message>,
        web_search_enabled: bool,
        session_id: &str,
    ) -> Result<(), HarperError> {
        // Handle slash commands locally
        if let Some(command) = user_msg.strip_prefix('/') {
            let response = match command {
                "help" => self.generate_help_text(),
                _ => match self.handle_command(command) {
                    CommandAction::Exit => "Session ended. Type /help for commands.".to_string(),
                    CommandAction::Clear => {
                        history.clear();
                        "Chat history cleared.".to_string()
                    }
                    CommandAction::Custom(desc) => {
                        self.add_user_message(history, session_id, &desc)?;
                        let response = self.process_message(history, web_search_enabled).await?;
                        self.add_assistant_message(history, session_id, &response)?;
                        self.trim_history(history);
                        return Ok(());
                    }
                    CommandAction::Unknown(s) => s,
                },
            };
            // Add as assistant message
            self.add_assistant_message(history, session_id, &response)?;
            return Ok(());
        }

        // Preprocess @file and @mcp_resource references
        let processed_msg = self.preprocess_file_references(user_msg);
        let processed_msg = self
            .preprocess_mcp_resource_references(&processed_msg)
            .await;

        // Normal message processing
        self.add_user_message(history, session_id, &processed_msg)?;
        let response = self.process_message(history, web_search_enabled).await?;
        self.add_assistant_message(history, session_id, &response)?;
        self.trim_history(history);
        Ok(())
    }

    /// Preprocess @mcp_resource references into content
    pub(crate) async fn preprocess_mcp_resource_references(&self, user_msg: &str) -> String {
        if self.mcp_client.is_none() {
            return user_msg.to_string();
        }

        let mut processed = user_msg.to_string();
        let mut search_start = 0;

        // Process all @mcp_resource references in the message
        while let Some(at_pos) = processed[search_start..].find('@') {
            let absolute_at_pos = search_start + at_pos;

            // Find the end of the resource URI (next space or end of string)
            let after_at = &processed[absolute_at_pos + 1..];
            let resource_end = after_at.find(' ').unwrap_or(after_at.len());
            let resource_part = &after_at[..resource_end];

            // Check if it starts with "mcp:" to identify MCP resources
            if let Some(resource_uri) = resource_part.strip_prefix("mcp:") {
                // Try to read the MCP resource
                match self.read_mcp_resource(resource_uri).await {
                    Ok(content) => {
                        // Replace @mcp:uri with the resource content
                        let replacement =
                            format!("\n[READ MCP RESOURCE: {}]\n{}\n", resource_uri, content);
                        let end_of_pattern = absolute_at_pos + 1 + resource_part.len();
                        processed.replace_range(absolute_at_pos..end_of_pattern, &replacement);

                        // Update search position to continue after this replacement
                        search_start = absolute_at_pos + replacement.len();
                    }
                    Err(e) => {
                        eprintln!(
                            "Warning: Failed to read MCP resource {}: {}",
                            resource_uri, e
                        );
                        search_start = absolute_at_pos + 1;
                    }
                }
            } else {
                // Not an MCP resource, skip this @
                search_start = absolute_at_pos + 1;
            }
        }

        processed
    }

    /// Read an MCP resource by URI
    async fn read_mcp_resource(&self, uri: &str) -> Result<String, HarperError> {
        let Some(client) = self.mcp_client else {
            return Err(HarperError::Config("MCP client not available".to_string()));
        };

        match client.read_resource(uri).await {
            Ok(contents) => {
                let mut content_parts = Vec::new();
                for item in &contents {
                    match item {
                        ResourceContent::Text(text_content) => {
                            content_parts.push(text_content.text.clone());
                        }
                        ResourceContent::Blob(blob_content) => {
                            content_parts.push(format!(
                                "[Binary content: {} bytes, type: {}]",
                                blob_content.blob.len(),
                                blob_content
                                    .mime_type
                                    .clone()
                                    .unwrap_or("unknown".to_string())
                            ));
                        }
                    }
                }
                Ok(content_parts.join("\n"))
            }
            Err(e) => Err(HarperError::Mcp(format!("MCP resource read failed: {}", e))),
        }
    }

    /// Preprocess @file references into \[READ_FILE\] commands
    pub(crate) fn preprocess_file_references(&self, user_msg: &str) -> String {
        let mut processed = user_msg.to_string();
        let mut search_start = 0;

        // Process all @file references in the message
        while let Some(at_pos) = processed[search_start..].find('@') {
            let absolute_at_pos = search_start + at_pos;

            // Find the end of the file path (next space or end of string)
            let after_at = &processed[absolute_at_pos + 1..];
            let path_end = after_at.find(' ').unwrap_or(after_at.len());
            let file_path = &after_at[..path_end];

            // Skip if empty or looks like a command (starts with / or !)
            if !file_path.is_empty() && !file_path.starts_with('/') && !file_path.starts_with('!') {
                // Replace @filepath with [READ_FILE filepath]
                let replacement = format!("[READ_FILE {}]", file_path);
                let old_pattern = format!("@{}", file_path);

                // Find the position of the old pattern in the current processed string
                if let Some(replace_pos) = processed[absolute_at_pos..].find(&old_pattern) {
                    let actual_replace_pos = absolute_at_pos + replace_pos;
                    processed.replace_range(
                        actual_replace_pos..actual_replace_pos + old_pattern.len(),
                        &replacement,
                    );

                    // Update search position to continue after this replacement
                    search_start = actual_replace_pos + replacement.len();
                } else {
                    // Pattern not found, skip this @ and continue
                    search_start = absolute_at_pos + 1;
                }
            } else {
                // Not a valid file reference, skip this @ and continue
                search_start = absolute_at_pos + 1;
            }
        }

        processed
    }

    /// Generate help text for commands
    fn generate_help_text(&self) -> String {
        let mut help_text = "Available commands:\n".to_string();
        help_text.push_str("  /help - Show this help\n");
        help_text.push_str("  /exit - Exit the session\n");
        help_text.push_str("  /clear - Clear chat history\n");
        help_text.push_str("  !command - Execute shell command directly\n");
        help_text.push_str("  @file - Reference and read files (with Tab completion)\n");
        for (cmd, desc) in &self.custom_commands {
            help_text.push_str(&format!("  /{} - {}\n", cmd, desc));
        }
        help_text
    }

    fn handle_command(&self, command: &str) -> CommandAction {
        match command {
            "exit" => CommandAction::Exit,
            "clear" => CommandAction::Clear,
            cmd => {
                if let Some(desc) = self.custom_commands.get(cmd) {
                    CommandAction::Custom(desc.clone())
                } else {
                    CommandAction::Unknown(format!(
                        "Unknown command '{}'. Type /help for available commands.",
                        cmd
                    ))
                }
            }
        }
    }

    /// Start the chat session
    pub async fn start_session(&mut self, web_search_enabled: bool) -> Result<(), HarperError> {
        let (mut history, session_id) = self.create_session(web_search_enabled).await?;

        self.display_session_start();

        self.run_chat_loop(&session_id, &mut history, web_search_enabled)
            .await
    }

    /// Build system prompt
    pub async fn build_system_prompt(&self, web_search_enabled: bool) -> String {
        let prompt_builder =
            PromptBuilder::new(self.config, self.prompt_id.clone(), self.mcp_client);
        prompt_builder.build_system_prompt(web_search_enabled).await
    }

    /// Display session start
    fn display_session_start(&self) {
        println!(
            "{}
",
            "🤖 Harper AI Assistant - Type /help for commands"
                .bold()
                .yellow()
        );
        println!(
            "💡 Quick commands: /help, /exit, /clear, !shell, @file
"
        );
    }

    /// Run the chat loop
    async fn run_chat_loop(
        &mut self,
        session_id: &str,
        history: &mut Vec<Message>,
        web_search_enabled: bool,
    ) -> Result<(), HarperError> {
        loop {
            let user_input = self.get_user_input()?;
            if self.should_exit(&user_input) {
                self.display_session_end();
                break;
            }

            // Handle direct shell commands
            if let Some(command) = user_input.strip_prefix('!') {
                match crate::tools::shell::execute_command(
                    &format!("[RUN_COMMAND {}]", command),
                    self.config,
                    &self.exec_policy,
                ) {
                    Ok(result) => println!("{} {}", "Shell:".bold().cyan(), result.cyan()),
                    Err(e) => println!("{} {}", "Error:".bold().red(), e),
                }
                continue;
            }

            // Handle slash commands
            if let Some(command) = user_input.strip_prefix('/') {
                match command {
                    "help" => {
                        let help_text = self.generate_help_text();
                        let mut lines = help_text.lines();
                        if let Some(first_line) = lines.next() {
                            println!("{}", first_line.bold().yellow());
                            lines.for_each(|line| println!("{}", line));
                        }
                        continue;
                    }
                    _ => match self.handle_command(command) {
                        CommandAction::Exit => {
                            self.display_session_end();
                            break;
                        }
                        CommandAction::Clear => {
                            history.clear();
                            println!("{}", "Chat history cleared.".bold().green());
                            continue;
                        }
                        CommandAction::Custom(desc) => {
                            println!("Custom command: {}", desc);
                            self.add_user_message(history, session_id, &desc)?;
                            let response = self
                                .process_message(history, web_search_enabled)
                                .await
                                .map_err(|e| {
                                    HarperError::Api(format!(
                                        "Failed to process message in session {}: {}",
                                        session_id, e
                                    ))
                                })?;
                            self.display_response(&response);
                            self.add_assistant_message(history, session_id, &response)?;
                            self.trim_history(history);
                            continue;
                        }
                        CommandAction::Unknown(s) => {
                            println!("{} {}", "Error:".bold().red(), s);
                            continue;
                        }
                    },
                }
            }

            self.add_user_message(history, session_id, &user_input)?;
            let response = self
                .process_message(history, web_search_enabled)
                .await
                .map_err(|e| {
                    HarperError::Api(format!(
                        "Failed to process message in session {}: {}",
                        session_id, e
                    ))
                })?;
            self.display_response(&response);
            self.add_assistant_message(history, session_id, &response)?;
            self.trim_history(history);
        }
        Ok(())
    }

    /// Get user input with completion
    fn get_user_input(&self) -> Result<String, HarperError> {
        let config = rustyline::Config::builder().build();
        let mut rl: Editor<FileCompleter, _> = Editor::with_config(config)?;
        rl.set_helper(Some(FileCompleter));
        let prompt = format!("{} ", "You:".bold().blue());
        match rl.readline(&prompt) {
            Ok(line) => {
                let _ = rl.add_history_entry(&line);
                Ok(line.trim().to_string())
            }
            Err(ReadlineError::Interrupted) | Err(ReadlineError::Eof) => Ok("exit".to_string()),
            Err(e) => Err(e.into()),
        }
    }

    /// Check if should exit
    pub fn should_exit(&self, input: &str) -> bool {
        input.is_empty() || input.eq_ignore_ascii_case("exit") || input.eq_ignore_ascii_case("quit")
    }

    /// Display session end
    fn display_session_end(&self) {
        println!("{}", "Session ended.".bold().yellow());
    }

    /// Process message
    async fn process_message(
        &mut self,
        history: &mut Vec<Message>,
        web_search_enabled: bool,
    ) -> Result<String, HarperError> {
        let client = Client::builder()
            .timeout(std::time::Duration::from_secs(90))
            .build()?;
        let history_for_llm = history.clone();
        let mut response = self.call_llm(&client, &history_for_llm).await?;
        let trimmed_response = response
            .trim()
            .trim_matches(|c| c == '\'' || c == '\"' || c == '`');

        let mut tool_service =
            ToolService::new(self.conn, self.config, &self.exec_policy, self.mcp_client);
        let tool_option = tool_service
            .handle_tool_use(
                &client,
                &history_for_llm,
                trimmed_response,
                web_search_enabled,
            )
            .await?;

        if let Some((tool_result, tool_content)) = tool_option {
            history.push(Message {
                role: "system".to_string(),
                content: tool_content,
            });
            response = tool_result;
        }

        Ok(response)
    }

    /// Call LLM
    async fn call_llm(
        &mut self,
        client: &Client,
        history: &[Message],
    ) -> Result<String, HarperError> {
        // Check cache
        if let Some(cache) = &self.api_cache {
            let cache_key = ApiCacheKey::new(
                &format!("{}", self.config.provider),
                &self.config.model_name,
                history,
            );

            if let Some(cached_response) = cache.get(&cache_key) {
                return Ok(cached_response.clone());
            }
        }

        // Make API call
        let response = crate::core::llm_client::call_llm(client, self.config, history).await?;

        // Cache response
        if let Some(cache) = &mut self.api_cache {
            let cache_key = ApiCacheKey::new(
                &format!("{}", self.config.provider),
                &self.config.model_name,
                history,
            );
            cache.insert(cache_key, response.clone());
        }

        Ok(response)
    }

    /// Display response
    fn display_response(&self, response: &str) {
        println!(
            "{} {}
",
            "Assistant:".bold().green(),
            response.green()
        );
    }

    /// Add user message
    fn add_user_message(
        &self,
        history: &mut Vec<Message>,
        session_id: &str,
        content: &str,
    ) -> Result<(), HarperError> {
        // Ensure session exists
        self.save_session(session_id)?;
        history.push(Message {
            role: "user".to_string(),
            content: content.to_string(),
        });
        crate::memory::storage::save_message(self.conn, session_id, "user", content)
    }

    /// Add assistant message
    fn add_assistant_message(
        &self,
        history: &mut Vec<Message>,
        session_id: &str,
        content: &str,
    ) -> Result<(), HarperError> {
        history.push(Message {
            role: "assistant".to_string(),
            content: content.to_string(),
        });
        crate::memory::storage::save_message(self.conn, session_id, "assistant", content)
    }

    /// Trim history
    fn trim_history(&self, history: &mut Vec<Message>) {
        const MAX_HISTORY: usize = 50;
        if history.len() > MAX_HISTORY {
            let to_remove_count = history.len() - MAX_HISTORY;
            history.drain(1..1 + to_remove_count);
        }
    }

    /// Save session
    fn save_session(&self, session_id: &str) -> Result<(), HarperError> {
        crate::memory::storage::save_session(self.conn, session_id)
    }

    /// Load custom prompt
    fn load_custom_prompt(&self, prompt_id: &str) -> Result<String, HarperError> {
        let home = dirs::home_dir()
            .ok_or_else(|| HarperError::Config("Home directory not found".to_string()))?;
        let prompt_path = home
            .join(".harper")
            .join("prompts")
            .join(format!("{}.md", prompt_id));
        std::fs::read_to_string(&prompt_path).map_err(|e| {
            HarperError::Config(format!(
                "Failed to load custom prompt {}: {}",
                prompt_path.display(),
                e
            ))
        })
    }

    /// Get project context
    fn get_project_context(&self) -> Result<String, HarperError> {
        let mut context = String::new();

        let current_dir = std::env::current_dir()
            .map_err(|e| HarperError::Command(format!("Failed to get current dir: {}", e)))?;

        let entries = std::fs::read_dir(&current_dir)
            .map_err(|e| HarperError::Command(format!("Failed to read dir: {}", e)))?;

        let mut files = Vec::new();
        for entry in entries.flatten() {
            if let Ok(file_name) = entry.file_name().into_string() {
                if !file_name.starts_with('.')
                    && file_name != "target"
                    && file_name != "node_modules"
                {
                    if let Ok(metadata) = entry.metadata() {
                        if metadata.is_dir() {
                            files.push(format!("{}/", file_name));
                        } else {
                            files.push(file_name);
                        }
                    }
                }
            }
        }
        files.sort();

        context.push_str(&format!(
            "Current directory: {}
",
            current_dir.display()
        ));
        context.push_str(&format!(
            "Files in project root: {}
",
            files.join(", ")
        ));

        // Git status
        if let Ok(git_status) = std::process::Command::new("git")
            .args(["status", "--porcelain"])
            .output()
        {
            if git_status.status.success() {
                let status = String::from_utf8_lossy(&git_status.stdout);
                if !status.trim().is_empty() {
                    context.push_str(&format!(
                        "Git status:
{}",
                        status
                    ));
                } else {
                    context.push_str(
                        "Git status: clean
",
                    );
                }
            }
        }

        Ok(context)
    }
}
