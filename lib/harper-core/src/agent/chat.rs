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

//! Chat interaction module
//!
//! This module handles user input, chat loops, and message processing.

use crate::agent::intent::{route_intent, DeterministicIntent};
use crate::agent::offline_shell::plan_offline_shell_commands;
use crate::agent::prompt::PromptBuilder;
use crate::core::cache::{ApiCacheKey, ApiResponseCache};
use crate::core::error::{HarperError, HarperResult};
use crate::core::{ApiConfig, Message};
use crate::memory::storage::CommandLogEntry;
use crate::runtime::config::ExecPolicyConfig;
use crate::runtime::scheduler::{TaskPriority, TaskScheduler};
use crate::tools::shell::CommandAuditContext;
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
use std::collections::{HashMap, HashSet};
use std::fs;
use std::path::{Path, PathBuf};
use std::time::{Duration, Instant};
use turul_mcp_client::{McpClient, ResourceContent};

#[derive(Debug)]
enum CommandAction {
    Clear,
    Exit,
    Audit(AuditParams),
    Custom(String),
    Unknown(String),
}

#[derive(Debug, Clone)]
enum ChatBackgroundTask {
    TodoReminder,
    AuditRefresh { session_id: String },
}

#[derive(Debug, Clone, Default)]
struct AuditParams {
    limit: Option<usize>,
    status: Option<String>,
    approval: Option<AuditApprovalFilter>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum AuditApprovalFilter {
    Approved,
    Rejected,
    Auto,
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

use crate::core::io_traits::{RuntimeEventSink, UserApproval};
use std::sync::Arc;

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
    approver: Option<Arc<dyn UserApproval>>,
    runtime_events: Option<Arc<dyn RuntimeEventSink>>,
    background_tasks: TaskScheduler<ChatBackgroundTask>,
    todo_reminder_armed: bool,
    last_audit_refresh: Option<Instant>,
}

#[allow(dead_code)]
impl<'a> ChatService<'a> {
    const AUDIT_LOG_LIMIT: usize = 10;
    const AUDIT_REFRESH_DEBOUNCE: Duration = Duration::from_secs(3);

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
            approver: None,
            runtime_events: None,
            background_tasks: TaskScheduler::new(),
            todo_reminder_armed: false,
            last_audit_refresh: None,
        }
    }

    /// Set a custom user approval provider
    pub fn with_approver(mut self, approver: Arc<dyn UserApproval>) -> Self {
        self.approver = Some(approver);
        self
    }

    pub fn with_runtime_events(mut self, runtime_events: Arc<dyn RuntimeEventSink>) -> Self {
        self.runtime_events = Some(runtime_events);
        self
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
                approval_profile: None,
                allowed_commands: None,
                blocked_commands: None,
                sandbox_profile: None,
                sandbox: None,
                retry_max_attempts: None,
                retry_network_commands: None,
                retry_write_commands: None,
            },
            approver: None,
            runtime_events: None,
            background_tasks: TaskScheduler::new(),
            todo_reminder_armed: false,
            last_audit_refresh: None,
        }
    }

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
        self.schedule_todo_reminder();
        self.poll_background_tasks(session_id);
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
                    CommandAction::Audit(params) => {
                        self.format_command_audit(session_id, &params)?
                    }
                    CommandAction::Custom(desc) => {
                        self.add_user_message(history, session_id, &desc)?;
                        let response = self
                            .process_message(history, web_search_enabled, session_id)
                            .await?;
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
        if let Some(local_response) = self
            .try_handle_deterministic_intent(&processed_msg, session_id)
            .await?
        {
            self.notify_command_activity(session_id);
            self.add_assistant_message(history, session_id, &local_response)?;
            self.trim_history(history);
            return Ok(());
        }
        if let Some(local_response) = self
            .try_handle_offline_shell_proxy(&processed_msg, session_id)
            .await?
        {
            self.add_assistant_message(history, session_id, &local_response)?;
            self.trim_history(history);
            return Ok(());
        }
        let response = self
            .process_message(history, web_search_enabled, session_id)
            .await?;
        self.add_assistant_message(history, session_id, &response)?;
        self.trim_history(history);
        self.poll_background_tasks(session_id);
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
        help_text.push_str("  /audit [limit] - Show recent command executions\n");
        help_text.push_str("  !command - Execute shell command directly\n");
        help_text.push_str("  @file - Reference and read files (with Tab completion)\n");
        for (cmd, desc) in &self.custom_commands {
            help_text.push_str(&format!("  /{} - {}\n", cmd, desc));
        }
        help_text
    }

    fn handle_command(&self, command: &str) -> CommandAction {
        let trimmed = command.trim();
        if trimmed.is_empty() {
            return CommandAction::Unknown("No command provided.".to_string());
        }

        let mut split_idx = None;
        for (idx, ch) in trimmed.char_indices() {
            if ch.is_whitespace() {
                split_idx = Some(idx);
                break;
            }
        }

        let (name, args) = if let Some(idx) = split_idx {
            let (left, right) = trimmed.split_at(idx);
            (left, Some(right.trim()))
        } else {
            (trimmed, None)
        };

        match name {
            "exit" => CommandAction::Exit,
            "clear" => CommandAction::Clear,
            "audit" => match parse_audit_params(args) {
                Ok(params) => CommandAction::Audit(params),
                Err(err) => CommandAction::Unknown(err.to_string()),
            },
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
        self.schedule_todo_reminder();

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

    fn schedule_todo_reminder(&mut self) {
        if self.todo_reminder_armed {
            return;
        }
        self.todo_reminder_armed = true;
        self.background_tasks.schedule_in(
            ChatBackgroundTask::TodoReminder,
            Duration::from_secs(45),
            TaskPriority::LOW,
        );
    }

    fn notify_command_activity(&mut self, session_id: &str) {
        let now = Instant::now();
        if let Some(last) = self.last_audit_refresh {
            if now.duration_since(last) < Self::AUDIT_REFRESH_DEBOUNCE {
                return;
            }
        }
        self.last_audit_refresh = Some(now);
        self.background_tasks.schedule_in(
            ChatBackgroundTask::AuditRefresh {
                session_id: session_id.to_string(),
            },
            Duration::from_millis(1200),
            TaskPriority::HIGH,
        );
    }

    fn poll_background_tasks(&mut self, default_session_id: &str) {
        let ready = self.background_tasks.drain_ready(Instant::now());
        for item in ready {
            match item.payload {
                ChatBackgroundTask::TodoReminder => {
                    self.todo_reminder_armed = false;
                    if let Err(err) = self.emit_todo_reminder() {
                        eprintln!("Warning: failed to load todos for reminder: {}", err);
                    }
                    self.schedule_todo_reminder();
                }
                ChatBackgroundTask::AuditRefresh { session_id } => {
                    let target = if session_id.is_empty() {
                        default_session_id.to_string()
                    } else {
                        session_id
                    };
                    self.emit_quick_audit_refresh(&target);
                }
            }
        }
    }

    fn emit_todo_reminder(&self) -> HarperResult<()> {
        let todos = crate::memory::storage::load_todos(self.conn)?;
        if todos.is_empty() {
            return Ok(());
        }

        println!(
            "
{} Pending todos ({} total):",
            "Reminder:".bold().cyan(),
            todos.len()
        );
        for (_, desc) in todos.iter().take(3) {
            println!("  - {}", desc);
        }
        if todos.len() > 3 {
            println!("  …and {} more", todos.len() - 3);
        }
        Ok(())
    }

    fn emit_quick_audit_refresh(&self, session_id: &str) {
        match self.fetch_command_logs(session_id, 3) {
            Ok(entries) if !entries.is_empty() => {
                println!(
                    "
{} Latest commands:",
                    "Audit:".bold().cyan()
                );
                for entry in entries {
                    let approval_state = if entry.requires_approval {
                        if entry.approved {
                            "approved"
                        } else {
                            "rejected"
                        }
                    } else {
                        "auto"
                    };
                    println!(
                        "  [{} | {} | {:?}] {}",
                        entry.status, approval_state, entry.exit_code, entry.command
                    );
                }
            }
            Ok(_) => {}
            Err(err) => eprintln!("Warning: failed to refresh audit summary: {}", err),
        }
    }

    /// Run the chat loop
    async fn run_chat_loop(
        &mut self,
        session_id: &str,
        history: &mut Vec<Message>,
        web_search_enabled: bool,
    ) -> Result<(), HarperError> {
        loop {
            self.poll_background_tasks(session_id);
            let user_input = self.get_user_input()?;
            if self.should_exit(&user_input) {
                self.display_session_end();
                break;
            }

            // Handle direct shell commands
            if let Some(command) = user_input.strip_prefix('!') {
                let audit_ctx = CommandAuditContext {
                    conn: self.conn,
                    session_id: Some(session_id),
                    source: "user_shell",
                };
                match crate::tools::shell::execute_command(
                    &format!("[RUN_COMMAND {}]", command),
                    self.config,
                    &self.exec_policy,
                    None,
                    Some(&audit_ctx),
                    self.approver.clone(),
                    self.runtime_events.clone(),
                )
                .await
                {
                    Ok(result) => println!("{} {}", "Shell:".bold().cyan(), result.cyan()),
                    Err(e) => println!("{} {}", "Error:".bold().red(), e),
                }
                self.notify_command_activity(session_id);
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
                        CommandAction::Audit(params) => {
                            let report = self.format_command_audit(session_id, &params)?;
                            for line in report.lines() {
                                println!("{}", line);
                            }
                            continue;
                        }
                        CommandAction::Custom(desc) => {
                            println!("Custom command: {}", desc);
                            self.add_user_message(history, session_id, &desc)?;
                            let response = self
                                .process_message(history, web_search_enabled, session_id)
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
                .process_message(history, web_search_enabled, session_id)
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
        session_id: &str,
    ) -> Result<String, HarperError> {
        const MAX_TOOL_ROUNDS: usize = 4;

        let client = Client::builder()
            .timeout(std::time::Duration::from_secs(90))
            .build()?;
        let mut history_for_llm = history.clone();
        if let Some(plan_prompt) = self.plan_prompt_for_request(history, session_id)? {
            history_for_llm.push(Message {
                role: "system".to_string(),
                content: plan_prompt,
            });
        }
        self.emit_activity_update(session_id, Some("thinking".to_string()));
        let mut response = self.call_llm(&client, &history_for_llm).await?;
        let mut executed_tool_calls: HashSet<String> = HashSet::new();
        let mut injected_agents_guidance: HashSet<String> = HashSet::new();
        let mut last_tool_content: Option<String> = None;
        let mut forced_tool_retry = false;
        let last_user_msg = history
            .iter()
            .rev()
            .find(|message| message.role == "user")
            .map(|message| message.content.clone())
            .unwrap_or_default();

        for _ in 0..MAX_TOOL_ROUNDS {
            let clean_response = Self::sanitize_model_response(&response);
            let tool_signature = Self::tool_call_signature(&clean_response);
            let dedupe_key = tool_signature
                .clone()
                .unwrap_or_else(|| clean_response.clone());
            if let Some(required_tool) =
                Self::forced_tool_retry_target(&last_user_msg, &clean_response, forced_tool_retry)
            {
                forced_tool_retry = true;
                history_for_llm.push(Message {
                    role: "system".to_string(),
                    content: format!(
                        "The user request requires an actual tool call. Do not answer with prose. Respond now with exactly one JSON tool call using `{}`.",
                        required_tool
                    ),
                });
                self.emit_activity_update(session_id, Some("thinking".to_string()));
                response = self.call_llm(&client, &history_for_llm).await?;
                continue;
            }
            if let Some(agents_prompt) = self.agents_guidance_for_tool_call(
                &clean_response,
                session_id,
                &injected_agents_guidance,
            )? {
                injected_agents_guidance.insert(dedupe_key.clone());
                history_for_llm.push(Message {
                    role: "system".to_string(),
                    content: agents_prompt,
                });
                self.emit_activity_update(session_id, Some("thinking".to_string()));
                response = self.call_llm(&client, &history_for_llm).await?;
                continue;
            }
            if executed_tool_calls.contains(&dedupe_key) {
                if let Some(content) = last_tool_content {
                    return Ok(format!("Tool result:\n{}", content));
                }
                break;
            }
            let tool_option = {
                let mut tool_service = ToolService::new(
                    self.conn,
                    self.config,
                    &self.exec_policy,
                    self.mcp_client,
                    Some(session_id),
                );
                if let Some(approver) = &self.approver {
                    tool_service = tool_service.with_approver(approver.clone());
                }
                if let Some(runtime_events) = &self.runtime_events {
                    tool_service = tool_service.with_runtime_events(runtime_events.clone());
                }
                tool_service
                    .handle_tool_use(
                        &client,
                        &history_for_llm,
                        &clean_response,
                        web_search_enabled,
                    )
                    .await?
            };

            if let Some((tool_result, tool_content)) = tool_option {
                self.notify_command_activity(session_id);
                executed_tool_calls.insert(dedupe_key);
                last_tool_content = Some(tool_content.clone());
                let tool_message = Message {
                    role: "system".to_string(),
                    content: tool_content,
                };
                history.push(tool_message.clone());
                history_for_llm.push(tool_message);
                response = tool_result;
                continue;
            }

            break;
        }

        Ok(response)
    }

    fn agents_guidance_for_tool_call(
        &self,
        tool_call: &str,
        session_id: &str,
        injected_agents_guidance: &HashSet<String>,
    ) -> Result<Option<String>, HarperError> {
        let Some(tool_signature) = Self::tool_call_signature(tool_call) else {
            return Ok(None);
        };
        if injected_agents_guidance.contains(&tool_signature) {
            return Ok(None);
        }

        let target_paths = ToolService::target_paths_for_tool_call(tool_call);
        if target_paths.is_empty() {
            crate::memory::storage::save_active_agents(self.conn, session_id, None)?;
            self.emit_agents_update(session_id, None);
            return Ok(None);
        }

        let cwd = std::env::current_dir().map_err(|err| {
            HarperError::Io(format!(
                "Failed to resolve current working directory: {}",
                err
            ))
        })?;
        let target_refs: Vec<&Path> = target_paths.iter().map(PathBuf::as_path).collect();
        let resolved_agents = crate::core::agents::resolve_agents_for_targets(&cwd, target_refs)?;
        crate::memory::storage::save_active_agents(self.conn, session_id, Some(&resolved_agents))?;
        self.emit_agents_update(session_id, Some(resolved_agents.clone()));
        let Some(rendered) = resolved_agents.render_for_prompt() else {
            return Ok(None);
        };

        Ok(Some(format!(
            "Before executing this path-targeting tool, apply these scoped AGENTS.md instructions for the affected files:\n{}",
            rendered
        )))
    }

    fn emit_agents_update(
        &self,
        session_id: &str,
        agents: Option<crate::core::agents::ResolvedAgents>,
    ) {
        let Some(runtime_events) = &self.runtime_events else {
            return;
        };
        if let Ok(handle) = tokio::runtime::Handle::try_current() {
            let runtime_events = runtime_events.clone();
            let session_id = session_id.to_string();
            handle.spawn(async move {
                let _ = runtime_events.agents_updated(&session_id, agents).await;
            });
        }
    }

    fn emit_activity_update(&self, session_id: &str, status: Option<String>) {
        let Some(runtime_events) = &self.runtime_events else {
            return;
        };
        if let Ok(handle) = tokio::runtime::Handle::try_current() {
            let runtime_events = runtime_events.clone();
            let session_id = session_id.to_string();
            handle.spawn(async move {
                let _ = runtime_events.activity_updated(&session_id, status).await;
            });
        }
    }

    fn plan_prompt_for_request(
        &self,
        history: &[Message],
        session_id: &str,
    ) -> Result<Option<String>, HarperError> {
        let Some(last_user_msg) = history
            .iter()
            .rev()
            .find(|message| message.role == "user")
            .map(|message| message.content.trim())
        else {
            return Ok(None);
        };

        if !Self::request_needs_plan(last_user_msg) {
            return Ok(None);
        }

        let existing_plan = crate::memory::storage::load_plan_state(self.conn, session_id)?;
        let has_active_plan = existing_plan.as_ref().is_some_and(|plan| {
            !plan.items.is_empty()
                && plan.items.iter().any(|item| {
                    !matches!(item.status, crate::core::plan::PlanStepStatus::Completed)
                })
        });

        if has_active_plan {
            if let Some(followup) = existing_plan
                .as_ref()
                .and_then(|plan| plan.runtime.as_ref())
                .and_then(|runtime| runtime.followup.as_ref())
            {
                match followup {
                    crate::core::plan::PlanFollowup::RetryOrReplan {
                        step,
                        command,
                        retry_count,
                    } => {
                        let command = command.as_deref().unwrap_or(step.as_str());
                        if *retry_count <= 1 {
                            return Ok(Some(format!(
                                "This multi-step task hit a failure at '{}' while running '{}'. Retry once if the issue looks transient; otherwise call update_plan to revise the remaining steps before more tool work.",
                                step, command
                            )));
                        }
                        return Ok(Some(format!(
                            "This multi-step task has already failed {} times at '{}' while running '{}'. Do not keep retrying blindly; call update_plan to revise the plan before more tool work.",
                            retry_count, step, command
                        )));
                    }
                    crate::core::plan::PlanFollowup::Checkpoint { step, next_step } => {
                        let next_clause = next_step
                            .as_deref()
                            .map(|next_step| {
                                format!(
                                    " Continue with '{}' only after a short checkpoint summary.",
                                    next_step
                                )
                            })
                            .unwrap_or_else(|| {
                                " If the task is done, confirm completion instead of repeating the plan."
                                    .to_string()
                            });
                        return Ok(Some(format!(
                            "This multi-step task has an open checkpoint on '{}'. Briefly summarize what changed before more tool work.{}",
                            step, next_clause
                        )));
                    }
                }
            }
            if let Some(blocked_step) = existing_plan.as_ref().and_then(|plan| {
                plan.items
                    .iter()
                    .find(|item| matches!(item.status, crate::core::plan::PlanStepStatus::Blocked))
            }) {
                return Ok(Some(format!(
                    "This multi-step task is currently blocked at '{}'. Before more tool work, call update_plan to explain the blocker, retry, or revise the remaining steps.",
                    blocked_step.step
                )));
            }
            return Ok(Some(
                "This is an active multi-step task. Keep the plan current with update_plan, and add a brief checkpoint summary whenever a step finishes or the next step changes."
                    .to_string(),
            ));
        }

        Ok(Some(
            "This request looks multi-step. Before doing substantial work, call update_plan with concise steps and exactly one in_progress item."
                .to_string(),
        ))
    }

    fn request_needs_plan(user_msg: &str) -> bool {
        let lower = user_msg.to_ascii_lowercase();
        let word_count = lower.split_whitespace().count();
        if word_count < 8 {
            return false;
        }

        let action_markers = [
            "fix",
            "implement",
            "update",
            "refactor",
            "debug",
            "investigate",
            "wire",
            "integrate",
            "add",
            "build",
            "make",
        ];
        let sequencing_markers = [
            " then ", " and ", " also ", " after ", " before ", " next ", " while ",
        ];

        let has_action = action_markers.iter().any(|marker| lower.contains(marker));
        let has_sequence = sequencing_markers
            .iter()
            .any(|marker| lower.contains(marker))
            || lower.contains(',')
            || lower.contains(':');

        has_action && (has_sequence || word_count >= 14)
    }

    fn request_requires_tool(user_msg: &str) -> Option<&'static str> {
        let lower = user_msg.to_ascii_lowercase();
        if lower.contains("git diff")
            || lower.contains("what changed")
            || lower.contains("show diff")
        {
            return Some("git_diff");
        }
        if lower.contains("git status") || lower.contains("status of repo") {
            return Some("git_status");
        }
        if (lower.contains("read ")
            || lower.contains("show ")
            || lower.contains("open ")
            || lower.contains("view ")
            || lower.contains("look at "))
            && (lower.contains(".rs")
                || lower.contains(".md")
                || lower.contains(".toml")
                || lower.contains("file"))
        {
            return Some("read_file");
        }
        if (lower.contains("fix ")
            || lower.contains("edit ")
            || lower.contains("change ")
            || lower.contains("update "))
            && (lower.contains("file") || lower.contains(".rs") || lower.contains(".md"))
        {
            return Some("search_replace");
        }
        if lower.contains("run ")
            || lower.contains("execute ")
            || lower.contains("make ")
            || lower.contains("cargo ")
            || lower.contains("grep ")
            || lower.contains("rg ")
        {
            return Some("run_command");
        }
        None
    }

    fn forced_tool_retry_target<'b>(
        user_msg: &'b str,
        model_response: &str,
        forced_tool_retry: bool,
    ) -> Option<&'b str> {
        if forced_tool_retry || Self::tool_call_signature(model_response).is_some() {
            return None;
        }
        Self::request_requires_tool(user_msg)
    }

    fn sanitize_model_response(response: &str) -> String {
        let trimmed_response = response.trim();

        if trimmed_response.starts_with("```") {
            let lines: Vec<&str> = trimmed_response.lines().collect();
            if lines.len() >= 2 {
                lines[1..lines.len() - 1].join("\n").trim().to_string()
            } else {
                trimmed_response.trim_matches('`').trim().to_string()
            }
        } else {
            trimmed_response
                .trim_matches(|c| c == '\'' || c == '\"' || c == '`')
                .trim()
                .to_string()
        }
    }

    fn tool_call_signature(response: &str) -> Option<String> {
        if let Ok(json_value) = serde_json::from_str::<serde_json::Value>(response) {
            // OpenAI tool_calls array format
            if let Some(arr) = json_value.as_array() {
                if let Some(first) = arr.first() {
                    let name = first
                        .get("function")
                        .and_then(|f| f.get("name"))
                        .and_then(|v| v.as_str())?;
                    let args = first
                        .get("function")
                        .and_then(|f| f.get("arguments"))
                        .cloned()
                        .unwrap_or(serde_json::Value::Null);
                    return Some(format!("json:{}:{}", name, args));
                }
            }

            // Gemini / regular JSON format
            if let Some(name) = json_value
                .get("tool")
                .and_then(|v| v.as_str())
                .or_else(|| json_value.get("name").and_then(|v| v.as_str()))
                .or_else(|| json_value.get("mcp_tool").and_then(|v| v.as_str()))
            {
                let args = json_value
                    .get("args")
                    .cloned()
                    .or_else(|| json_value.get("arguments").cloned())
                    .unwrap_or(serde_json::Value::Null);
                return Some(format!("json:{}:{}", name, args));
            }
        }

        let upper = response.to_uppercase();
        if upper.starts_with("[READ_FILE") {
            return Some(response.trim().to_string());
        }
        if upper.starts_with("[RUN_COMMAND") {
            return Some(response.trim().to_string());
        }
        if upper.starts_with("[WRITE_FILE")
            || upper.starts_with("[SEARCH_REPLACE")
            || upper.starts_with("[GIT_STATUS")
            || upper.starts_with("[GIT_DIFF")
            || upper.starts_with("[GIT_ADD")
            || upper.starts_with("[GIT_COMMIT")
        {
            return Some(response.trim().to_string());
        }
        None
    }

    async fn try_handle_deterministic_intent(
        &self,
        user_msg: &str,
        session_id: &str,
    ) -> Result<Option<String>, HarperError> {
        match route_intent(user_msg) {
            Some(DeterministicIntent::ListChangedFiles(filters)) => {
                let audit_ctx = CommandAuditContext {
                    conn: self.conn,
                    session_id: Some(session_id),
                    source: "intent_list_changed_files",
                };
                let response = crate::tools::git::list_changed_files_with_policy(
                    self.config,
                    &self.exec_policy,
                    Some(&audit_ctx),
                    self.approver.clone(),
                    filters.ext.as_deref(),
                    filters.tracked_only,
                    filters.since.as_deref(),
                )
                .await?;
                Ok(Some(response))
            }
            None => Ok(None),
        }
    }

    async fn try_handle_offline_shell_proxy(
        &mut self,
        user_msg: &str,
        session_id: &str,
    ) -> Result<Option<String>, HarperError> {
        let commands = plan_offline_shell_commands(user_msg);
        if commands.is_empty() {
            return Ok(None);
        }

        let audit_ctx = CommandAuditContext {
            conn: self.conn,
            session_id: Some(session_id),
            source: "offline_shell_nlu",
        };
        let mut sections = Vec::new();
        for command in commands {
            let bracket = format!("[RUN_COMMAND {}]", command);
            let output = crate::tools::shell::execute_command(
                &bracket,
                self.config,
                &self.exec_policy,
                None,
                Some(&audit_ctx),
                self.approver.clone(),
                self.runtime_events.clone(),
            )
            .await?;
            if output.trim().is_empty() {
                sections.push(format!("$ {}\n[no output]", command));
            } else {
                sections.push(format!("$ {}\n{}", command, output.trim_end()));
            }
        }
        self.notify_command_activity(session_id);
        Ok(Some(sections.join("\n\n")))
    }

    fn format_command_audit(
        &self,
        session_id: &str,
        params: &AuditParams,
    ) -> Result<String, HarperError> {
        let requested_limit = params.limit.unwrap_or(Self::AUDIT_LOG_LIMIT);
        let limit = requested_limit.clamp(1, 100);
        let fetch_limit = std::cmp::max(limit * 3, limit).min(300);

        let entries = self.fetch_command_logs(session_id, fetch_limit)?;

        let filtered: Vec<_> = entries
            .into_iter()
            .filter(|entry| params.matches(entry))
            .take(limit)
            .collect();

        let mut lines = Vec::new();
        let mut filters = Vec::new();
        if let Some(status) = &params.status {
            filters.push(format!("status={}", status));
        }
        if let Some(appr) = params.approval {
            filters.push(format!(
                "approval={}",
                match appr {
                    AuditApprovalFilter::Approved => "approved",
                    AuditApprovalFilter::Rejected => "rejected",
                    AuditApprovalFilter::Auto => "auto",
                }
            ));
        }
        if !filters.is_empty() {
            lines.push(format!(
                "Recent command executions (showing up to {}, filters: {})",
                limit,
                filters.join(", ")
            ));
        } else {
            lines.push(format!(
                "Recent command executions (showing up to {}):",
                limit
            ));
        }

        if filtered.is_empty() {
            lines.push("No commands have been executed yet.".to_string());
            return Ok(lines.join("\n"));
        }

        for (idx, entry) in filtered.iter().enumerate() {
            let approval_state = if entry.requires_approval {
                if entry.approved {
                    "approved"
                } else {
                    "rejected"
                }
            } else {
                "auto"
            };
            let exit = entry
                .exit_code
                .map(|code| format!("exit {}", code))
                .unwrap_or_else(|| "no exit code".to_string());
            let duration = entry
                .duration_ms
                .map(|ms| format!("{} ms", ms))
                .unwrap_or_else(|| "-".to_string());
            lines.push(format!(
                "{:>2}. {} [{} | {} | {}]\n    {}",
                idx + 1,
                entry.status,
                approval_state,
                exit,
                duration,
                entry.command
            ));
        }

        Ok(lines.join("\n"))
    }

    fn fetch_command_logs(
        &self,
        session_id: &str,
        limit: usize,
    ) -> Result<Vec<CommandLogEntry>, HarperError> {
        crate::memory::storage::load_command_logs_for_session(self.conn, session_id, limit)
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
        let user_message_count = history
            .iter()
            .filter(|message| message.role == "user")
            .count();
        if user_message_count == 1 {
            let title = Self::derive_session_title(content);
            let _ = crate::memory::storage::update_session_title(self.conn, session_id, &title);
        }
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

    fn derive_session_title(content: &str) -> String {
        let single_line = content
            .lines()
            .find(|line| !line.trim().is_empty())
            .unwrap_or(content)
            .trim();
        let normalized = single_line.split_whitespace().collect::<Vec<_>>().join(" ");
        if normalized.is_empty() {
            return "New session".to_string();
        }
        const MAX_LEN: usize = 48;
        if normalized.chars().count() <= MAX_LEN {
            normalized
        } else {
            let truncated = normalized.chars().take(MAX_LEN).collect::<String>();
            format!("{}...", truncated.trim_end())
        }
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

fn parse_audit_params(args: Option<&str>) -> HarperResult<AuditParams> {
    let mut params = AuditParams::default();

    let Some(arg_text) = args.filter(|a| !a.is_empty()) else {
        return Ok(params);
    };

    for token in arg_text.split_whitespace() {
        if token.chars().all(|c| c.is_ascii_digit()) {
            if params.limit.is_some() {
                return Err(HarperError::Api(
                    "Audit limit specified multiple times.".to_string(),
                ));
            }
            let parsed = token.parse::<usize>().map_err(|_| {
                HarperError::Api(format!(
                    "Invalid limit '{}'. Use a positive integer.",
                    token
                ))
            })?;
            if parsed == 0 {
                return Err(HarperError::Api(
                    "Audit limit must be greater than zero.".to_string(),
                ));
            }
            params.limit = Some(parsed);
            continue;
        }

        if let Some(value) = token.strip_prefix("status=") {
            if params.status.is_some() {
                return Err(HarperError::Api(
                    "Status filter specified multiple times.".to_string(),
                ));
            }
            if value.is_empty() {
                return Err(HarperError::Api(
                    "Status filter cannot be empty.".to_string(),
                ));
            }
            params.status = Some(value.to_lowercase());
            continue;
        }

        if let Some(value) = token.strip_prefix("approval=") {
            if params.approval.is_some() {
                return Err(HarperError::Api(
                    "Approval filter specified multiple times.".to_string(),
                ));
            }
            params.approval = Some(match value.to_lowercase().as_str() {
                "approved" => AuditApprovalFilter::Approved,
                "rejected" => AuditApprovalFilter::Rejected,
                "auto" => AuditApprovalFilter::Auto,
                _ => {
                    return Err(HarperError::Api(
                        "Approval filter must be one of: approved, rejected, auto.".to_string(),
                    ))
                }
            });
            continue;
        }

        let lower = token.to_lowercase();
        if matches!(
            lower.as_str(),
            "failed" | "succeeded" | "error" | "cancelled" | "blocked"
        ) {
            if params.status.is_some() {
                return Err(HarperError::Api(
                    "Status filter specified multiple times.".to_string(),
                ));
            }
            params.status = Some(lower);
            continue;
        }

        if matches!(lower.as_str(), "approved" | "rejected" | "auto") {
            if params.approval.is_some() {
                return Err(HarperError::Api(
                    "Approval filter specified multiple times.".to_string(),
                ));
            }
            params.approval = Some(match lower.as_str() {
                "approved" => AuditApprovalFilter::Approved,
                "rejected" => AuditApprovalFilter::Rejected,
                "auto" => AuditApprovalFilter::Auto,
                _ => unreachable!(),
            });
            continue;
        }

        return Err(HarperError::Api(format!(
            "Unknown audit argument '{}'. Use /audit [limit] [status=...] [approval=approved|rejected|auto].",
            token
        )));
    }

    Ok(params)
}

impl AuditParams {
    fn matches(&self, entry: &CommandLogEntry) -> bool {
        let status_match = self
            .status
            .as_ref()
            .map(|s| entry.status.eq_ignore_ascii_case(s))
            .unwrap_or(true);

        let approval_match = match self.approval {
            Some(AuditApprovalFilter::Approved) => entry.requires_approval && entry.approved,
            Some(AuditApprovalFilter::Rejected) => entry.requires_approval && !entry.approved,
            Some(AuditApprovalFilter::Auto) => !entry.requires_approval,
            None => true,
        };

        status_match && approval_match
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::core::{ApiConfig, ApiProvider};

    fn test_config() -> ApiConfig {
        ApiConfig {
            provider: ApiProvider::OpenAI,
            api_key: "test-key".to_string(),
            base_url: "https://api.openai.com/v1/chat/completions".to_string(),
            model_name: "gpt-5.5".to_string(),
        }
    }

    #[test]
    fn parse_audit_no_args() {
        let params = parse_audit_params(None).expect("parse");
        assert!(params.limit.is_none());
        assert!(params.status.is_none());
    }

    #[test]
    fn parse_audit_with_limit_and_filters() {
        let params = parse_audit_params(Some("25 status=failed approval=rejected")).expect("parse");
        assert_eq!(params.limit, Some(25));
        assert_eq!(params.status.as_deref(), Some("failed"));
        assert_eq!(params.approval, Some(AuditApprovalFilter::Rejected));
    }

    #[test]
    fn parse_audit_errors_on_unknown_token() {
        let err = parse_audit_params(Some("foo=bar")).expect_err("should fail");
        assert!(err.to_string().contains("Unknown audit argument"));
    }

    #[test]
    fn audit_params_matches_filters() {
        let mut entry = CommandLogEntry {
            command: "echo hi".to_string(),
            source: "test".to_string(),
            requires_approval: true,
            approved: true,
            status: "succeeded".to_string(),
            exit_code: Some(0),
            duration_ms: Some(10),
            created_at: "2026-02-12".to_string(),
        };

        let params = AuditParams {
            limit: Some(5),
            status: Some("succeeded".to_string()),
            approval: Some(AuditApprovalFilter::Approved),
        };
        assert!(params.matches(&entry));

        entry.status = "failed".to_string();
        assert!(!params.matches(&entry));
    }

    #[test]
    fn parse_audit_shorthand_tokens() {
        let params = parse_audit_params(Some("failed approved")).expect("parse");
        assert_eq!(params.status.as_deref(), Some("failed"));
        assert_eq!(params.approval, Some(AuditApprovalFilter::Approved));
    }

    #[test]
    fn parse_audit_duplicate_status_errors() {
        let err = parse_audit_params(Some("status=failed succeeded")).expect_err("should fail");
        assert!(err
            .to_string()
            .contains("Status filter specified multiple times"));
    }

    #[test]
    fn request_needs_plan_for_multi_step_work() {
        assert!(ChatService::request_needs_plan(
            "fix the HTTP tool path and then wire the UI refresh too"
        ));
        assert!(ChatService::request_needs_plan(
            "implement storage, update the prompt, and add tests"
        ));
        assert!(!ChatService::request_needs_plan("run git status"));
        assert!(!ChatService::request_needs_plan("fix typo"));
    }

    #[test]
    fn plan_prompt_mentions_blocked_step_when_plan_is_blocked() {
        let conn = Connection::open_in_memory().expect("in-memory db");
        crate::memory::storage::init_db(&conn).expect("init db");
        crate::memory::storage::save_plan_state(
            &conn,
            "blocked-plan-session",
            &crate::core::plan::PlanState {
                explanation: Some("blocked".to_string()),
                items: vec![crate::core::plan::PlanItem {
                    step: "Fix failing migration".to_string(),
                    status: crate::core::plan::PlanStepStatus::Blocked,
                    job_id: None,
                }],
                runtime: None,
                updated_at: None,
            },
        )
        .expect("save plan");

        let config = test_config();
        let exec_policy = ExecPolicyConfig::default();
        let chat = ChatService::new(
            &conn,
            &config,
            None,
            None,
            Some("blocked-plan-session".to_string()),
            HashMap::new(),
            exec_policy,
        );
        let history = vec![Message {
            role: "user".to_string(),
            content: "fix the migration and then update the docs".to_string(),
        }];

        let prompt = chat
            .plan_prompt_for_request(&history, "blocked-plan-session")
            .expect("plan prompt")
            .expect("prompt present");

        assert!(prompt.contains("blocked at 'Fix failing migration'"));
        assert!(prompt.contains("call update_plan"));
    }

    #[test]
    fn plan_prompt_mentions_retry_limit_when_followup_requires_replan() {
        let conn = Connection::open_in_memory().expect("in-memory db");
        crate::memory::storage::init_db(&conn).expect("init db");
        crate::memory::storage::save_plan_state(
            &conn,
            "retry-plan-session",
            &crate::core::plan::PlanState {
                explanation: Some("retry".to_string()),
                items: vec![crate::core::plan::PlanItem {
                    step: "Patch handler".to_string(),
                    status: crate::core::plan::PlanStepStatus::Blocked,
                    job_id: None,
                }],
                runtime: Some(crate::core::plan::PlanRuntime {
                    followup: Some(crate::core::plan::PlanFollowup::RetryOrReplan {
                        step: "Patch handler".to_string(),
                        command: Some("cargo test".to_string()),
                        retry_count: 2,
                    }),
                    ..Default::default()
                }),
                updated_at: None,
            },
        )
        .expect("save plan");

        let config = test_config();
        let exec_policy = ExecPolicyConfig::default();
        let chat = ChatService::new(
            &conn,
            &config,
            None,
            None,
            Some("retry-plan-session".to_string()),
            HashMap::new(),
            exec_policy,
        );
        let history = vec![Message {
            role: "user".to_string(),
            content: "fix the handler and then rerun the tests".to_string(),
        }];

        let prompt = chat
            .plan_prompt_for_request(&history, "retry-plan-session")
            .expect("plan prompt")
            .expect("prompt present");

        assert!(prompt.contains("already failed 2 times"));
        assert!(prompt.contains("Do not keep retrying blindly"));
    }

    #[test]
    fn forced_tool_retry_targets_prose_when_tool_is_required() {
        let user_msg = "read lib/harper-core/src/agent/chat.rs";
        let prose = "Sorry, I cannot inspect files directly from here.";
        assert_eq!(
            ChatService::forced_tool_retry_target(user_msg, prose, false),
            Some("read_file")
        );
    }

    #[test]
    fn forced_tool_retry_skips_real_tool_calls() {
        let user_msg = "read lib/harper-core/src/agent/chat.rs";
        let tool_call =
            r#"{"tool":"read_file","args":{"path":"lib/harper-core/src/agent/chat.rs"}}"#;
        assert_eq!(
            ChatService::forced_tool_retry_target(user_msg, tool_call, false),
            None
        );
    }

    #[test]
    fn forced_tool_retry_runs_only_once() {
        let user_msg = "run cargo test";
        let prose = "I would run tests to verify the change.";
        assert_eq!(
            ChatService::forced_tool_retry_target(user_msg, prose, true),
            None
        );
    }

    #[test]
    fn derive_session_title_uses_first_meaningful_line() {
        let title = ChatService::derive_session_title(
            "\n\nfix the tui spinner visibility\nand session names",
        );
        assert_eq!(title, "fix the tui spinner visibility");
    }

    #[test]
    fn derive_session_title_truncates_long_input() {
        let title = ChatService::derive_session_title(
            "this is a very long request title that should be shortened for the session list display",
        );
        assert!(title.ends_with("..."));
        assert!(title.len() <= 51);
    }
}
