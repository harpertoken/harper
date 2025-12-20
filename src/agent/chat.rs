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

use crate::core::cache::{ApiCacheKey, ApiResponseCache};
use crate::core::error::HarperError;
use crate::core::{ApiConfig, Message};
use crate::runtime::config::ExecPolicyConfig;
use crate::tools::ToolService;
use chrono::Datelike;
use colored::*;
use reqwest::Client;
use rusqlite::Connection;
use std::collections::HashMap;
use std::io::{self, Write};

#[derive(Debug)]
enum CommandAction {
    Clear,
    Exit,
    Custom(String),
    Unknown(String),
}

/// Chat service for handling conversations
pub struct ChatService<'a> {
    conn: &'a Connection,
    config: &'a ApiConfig,
    api_cache: Option<&'a mut ApiResponseCache>,
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
        api_cache: Option<&'a mut ApiResponseCache>,
        prompt_id: Option<String>,
        custom_commands: HashMap<String, String>,
        exec_policy: ExecPolicyConfig,
    ) -> Self {
        Self {
            conn,
            config,
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

    /// Start a new interactive chat session
    pub fn create_session(
        &self,
        web_search_enabled: bool,
    ) -> Result<(Vec<Message>, String), HarperError> {
        let session_id = uuid::Uuid::new_v4().to_string();
        self.save_session(&session_id)?;

        let system_prompt = self.build_system_prompt(web_search_enabled);

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

        // Normal message processing
        self.add_user_message(history, session_id, user_msg)?;
        let response = self.process_message(history, web_search_enabled).await?;
        self.add_assistant_message(history, session_id, &response)?;
        self.trim_history(history);
        Ok(())
    }

    /// Generate help text for commands
    fn generate_help_text(&self) -> String {
        let mut help_text = "Available commands:\n".to_string();
        help_text.push_str("  /help - Show this help\n");
        help_text.push_str("  /exit - Exit the session\n");
        help_text.push_str("  /clear - Clear chat history\n");
        help_text.push_str("  !command - Execute shell command directly\n");
        help_text.push_str("  @file - Reference files (with Tab completion)\n");
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
        let (mut history, session_id) = self.create_session(web_search_enabled)?;

        self.display_session_start();

        self.run_chat_loop(&session_id, &mut history, web_search_enabled)
            .await
    }

    /// Build system prompt
    pub fn build_system_prompt(&self, web_search_enabled: bool) -> String {
        // Load custom prompt if specified
        if let Some(ref id) = self.prompt_id {
            if id != "default" {
                if let Ok(custom_prompt) = self.load_custom_prompt(id) {
                    return custom_prompt;
                }
            }
        }

        let mut prompt = format!(
            "You are a helpful AI assistant powered by the {} model.
You have the ability to read and write files, search and replace text in files, and run shell commands{}.",
            self.config.model_name,
            if web_search_enabled { " and search the web" } else { "" }
        );

        // Add project context
        if let Ok(context) = self.get_project_context() {
            prompt.push_str(&format!("\n\nProject Context:\n{}\n", context));
        }

        prompt.push_str("

You have tools to interact with the system. To use a tool, respond with ONLY the tool command. Do not add any other text. If you cannot use a tool for the user's request, explain why.

Available tools:
- read_file(path): Read the contents of a file
- write_file(path, content): Write content to a file
- search_replace(path, old_string, new_string): Search and replace text in a file
- run_command(command): Run a shell command
- todo(action, description?, index?): Manage todo list (actions: add, list, remove, clear)

To use a tool, respond with a JSON object like: {\"tool\": \"write_file\", \"path\": \"example.txt\", \"content\": \"Hello world\"}");

        // Load and append agent guidelines
        match std::fs::read_to_string("AGENTS.md") {
            Ok(guidelines) => prompt.push_str(&format!("\n\nAgent Guidelines:\n{}\n", guidelines)),
            Err(e) => eprintln!(
                "Warning: Could not load AGENTS.md: {}. Agent will proceed without guidelines.",
                e
            ),
        }

        if web_search_enabled {
            let current_year = chrono::Local::now().year();
            prompt.push_str(&format!(
                "\n- Search the web: `[SEARCH: your query]`. Current year: {}\n",
                current_year
            ));
        }

        prompt
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
                        let help_lines: Vec<&str> = help_text.lines().collect();
                        println!("{}", help_lines[0].bold().yellow());
                        for line in &help_lines[1..] {
                            println!("{}", line);
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

    /// Get user input
    fn get_user_input(&self) -> Result<String, HarperError> {
        print!("{} ", "You:".bold().blue());
        io::stdout().flush()?;
        let mut input = String::new();
        io::stdin().read_line(&mut input)?;
        Ok(input.trim().to_string())
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

        let mut tool_service = ToolService::new(self.config, &self.exec_policy);
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
