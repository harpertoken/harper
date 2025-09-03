//! Chat service for managing AI conversations
//!
//! This module provides the core functionality for handling chat sessions,
//! including user input processing, AI API calls, and tool execution.

use super::{ApiConfig, Message};
use crate::core::cache::{ApiCacheKey, ApiResponseCache};
use crate::core::constants::{exit_commands, timeouts, tools};
use crate::core::error::{HarperError, HarperResult};
use crate::utils::web_search;
use chrono::Datelike;
use colored::*;
// use mcp_client::{transport::sse::SseTransportHandle, McpClient, McpService}; // Temporarily disabled
use reqwest::Client;
use rusqlite::Connection;
use std::io::{self, Write};

// use tower::timeout::Timeout; // Temporarily disabled
use uuid::Uuid;

/// Service for handling chat sessions and interactions
///
/// Manages the lifecycle of chat sessions, including user input processing,
/// AI API communication, tool execution, and conversation persistence.
pub struct ChatService<'a> {
    conn: &'a Connection,
    config: &'a ApiConfig,
    // mcp_client: Option<&'a McpClient<Timeout<McpService<SseTransportHandle>>>>, // Temporarily disabled
    api_cache: Option<&'a mut ApiResponseCache>,
}

impl<'a> ChatService<'a> {
    /// Create a new chat service
    ///
    /// # Arguments
    /// * `conn` - Database connection for storing conversation history
    /// * `config` - API configuration for the AI provider
    /// * `api_cache` - Optional cache for API responses
    pub fn new(
        conn: &'a Connection,
        config: &'a ApiConfig,
        // mcp_client: Option<&'a McpClient<Timeout<McpService<SseTransportHandle>>>>, // Temporarily disabled
        api_cache: Option<&'a mut ApiResponseCache>,
    ) -> Self {
        Self {
            conn,
            config,
            // mcp_client, // Temporarily disabled
            api_cache,
        }
    }

    /// Create a new chat service for testing (without MCP client)
    #[cfg(test)]
    #[allow(dead_code)]
    pub fn new_test(conn: &'a Connection, config: &'a ApiConfig) -> Self {
        Self {
            conn,
            config,
            // mcp_client: None, // Temporarily disabled
            api_cache: None,
        }
    }

    /// Start a new interactive chat session
    ///
    /// This method handles the complete chat session lifecycle including:
    /// - Session initialization
    /// - User input processing
    /// - AI API calls
    /// - Tool execution (commands, web search)
    /// - Conversation persistence
    pub async fn start_session(&mut self) -> HarperResult<()> {
        let session_id = Uuid::new_v4().to_string();
        self.save_session(&session_id)?;

        let web_search_enabled = self.prompt_web_search()?;
        let system_prompt = self.build_system_prompt(web_search_enabled);

        let mut history = vec![Message {
            role: "system".to_string(),
            content: system_prompt,
        }];

        self.display_session_start();

        self.run_chat_loop(&session_id, &mut history, web_search_enabled)
            .await
    }

    /// Save a session to the database
    fn save_session(&self, session_id: &str) -> HarperResult<()> {
        crate::save_session(self.conn, session_id)
    }

    /// Prompt user to enable web search
    fn prompt_web_search(&self) -> HarperResult<bool> {
        print!("Enable web search for this session? (y/n): ");
        io::stdout().flush()?;
        let mut choice = String::new();
        io::stdin().read_line(&mut choice)?;
        Ok(choice.trim().eq_ignore_ascii_case("y"))
    }

    /// Build the system prompt based on configuration
    pub fn build_system_prompt(&self, web_search_enabled: bool) -> String {
        if web_search_enabled {
            let current_year = chrono::Local::now().year();
            format!(
                "You are a helpful AI assistant powered by the {} model.
You have the ability to run any Linux shell command.
Your response MUST be ONLY the tool command. Do not add any explanation.
Do NOT use interactive commands (like 'nano', 'vim'). Use non-interactive commands like `cat` to read files.

Tool format:
- Run a shell command: `[RUN_COMMAND <command to run>]`
- Search the web: `[SEARCH: your query]`. Current year: {}",
                self.config.model_name, current_year
            )
        } else {
            format!(
                "You are an AI assistant powered by the {} model.",
                self.config.model_name
            )
        }
    }

    /// Display session start message
    fn display_session_start(&self) {
        println!(
            "{}\n",
            "New chat session started. Type 'exit' to quit."
                .bold()
                .yellow()
        );
    }

    /// Run the main chat interaction loop
    async fn run_chat_loop(
        &mut self,
        session_id: &str,
        history: &mut Vec<Message>,
        web_search_enabled: bool,
    ) -> HarperResult<()> {
        loop {
            let user_input = self.get_user_input()?;
            if self.should_exit(&user_input) {
                self.display_session_end();
                break;
            }

            self.add_user_message(history, session_id, &user_input)?;
            let response = self.process_message(history, web_search_enabled).await?;
            self.display_response(&response);
            self.add_assistant_message(history, session_id, &response)?;
        }
        Ok(())
    }

    /// Get user input from stdin
    fn get_user_input(&self) -> HarperResult<String> {
        print!("{} ", "You:".bold().blue());
        io::stdout().flush()?;
        let mut input = String::new();
        io::stdin().read_line(&mut input)?;
        Ok(input.trim().to_string())
    }

    /// Check if user wants to exit
    pub fn should_exit(&self, input: &str) -> bool {
        input.is_empty()
            || input.eq_ignore_ascii_case(exit_commands::EXIT)
            || input.eq_ignore_ascii_case(exit_commands::QUIT)
    }

    /// Display session end message
    fn display_session_end(&self) {
        println!("{}", "Session ended.".bold().yellow());
    }

    /// Add user message to history and save to database
    fn add_user_message(
        &self,
        history: &mut Vec<Message>,
        session_id: &str,
        content: &str,
    ) -> HarperResult<()> {
        history.push(Message {
            role: "user".to_string(),
            content: content.to_string(),
        });
        crate::save_message(self.conn, session_id, "user", content)
    }

    /// Process a message and get AI response
    async fn process_message(
        &mut self,
        history: &[Message],
        web_search_enabled: bool,
    ) -> HarperResult<String> {
        let client = Client::builder().timeout(timeouts::API_REQUEST).build()?;

        let mut response = self.call_llm(&client, history).await?;
        let trimmed_response = response
            .trim()
            .trim_matches(|c| c == '\'' || c == '\"' || c == '`');

        if let Some(tool_result) = self
            .handle_tool_use(&client, history, trimmed_response, web_search_enabled)
            .await?
        {
            response = tool_result;
        }

        Ok(response)
    }

    /// Call the LLM API with caching
    async fn call_llm(&mut self, client: &Client, history: &[Message]) -> HarperResult<String> {
        // Check cache first
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
        let response = crate::call_llm(client, self.config, history).await?;

        // Cache the response
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

    /// Handle tool usage (commands and web search)
    async fn handle_tool_use(
        &mut self,
        client: &Client,
        history: &[Message],
        response: &str,
        web_search_enabled: bool,
    ) -> HarperResult<Option<String>> {
        if response.to_uppercase().starts_with(tools::RUN_COMMAND) {
            let command_result = self.execute_command(response)?;
            let final_response = self
                .call_llm_after_tool(client, history, &command_result)
                .await?;
            Ok(Some(final_response))
        } else if web_search_enabled && response.to_uppercase().starts_with(tools::SEARCH) {
            let search_result = self.perform_web_search(response).await?;
            let final_response = self
                .call_llm_after_tool(client, history, &search_result)
                .await?;
            Ok(Some(final_response))
        } else {
            Ok(None)
        }
    }

    /// Execute a shell command
    fn execute_command(&self, response: &str) -> HarperResult<String> {
        let command_str = if let Some(pos) = response.find(' ') {
            response[pos..].trim_start().trim_end_matches(']')
        } else {
            ""
        };

        if command_str.is_empty() {
            return Err(HarperError::Command("No command provided".to_string()));
        }

        println!(
            "{} Running command: {}",
            "System:".bold().magenta(),
            command_str.magenta()
        );

        let output = std::process::Command::new("sh")
            .arg("-c")
            .arg(command_str)
            .output()
            .map_err(|e| HarperError::Command(format!("Failed to execute command: {}", e)))?;

        let result = if output.status.success() {
            String::from_utf8_lossy(&output.stdout).to_string()
        } else {
            String::from_utf8_lossy(&output.stderr).to_string()
        };

        Ok(result)
    }

    /// Perform web search
    async fn perform_web_search(&self, response: &str) -> HarperResult<String> {
        let query_part = response
            .split_once(':')
            .map(|x| x.1)
            .unwrap_or("")
            .trim_end_matches(']');

        println!(
            "{} Searching the web for: {}",
            "System:".bold().magenta(),
            query_part.magenta()
        );

        web_search(query_part).await
    }

    /// Call LLM after tool usage
    async fn call_llm_after_tool(
        &mut self,
        client: &Client,
        history: &[Message],
        _tool_result: &str,
    ) -> HarperResult<String> {
        self.call_llm(client, history).await
    }

    /// Display assistant response
    fn display_response(&self, response: &str) {
        println!("{} {}\n", "Assistant:".bold().green(), response.green());
    }

    /// Add assistant message to history and save to database
    fn add_assistant_message(
        &self,
        history: &mut Vec<Message>,
        session_id: &str,
        content: &str,
    ) -> HarperResult<()> {
        history.push(Message {
            role: "assistant".to_string(),
            content: content.to_string(),
        });
        crate::save_message(self.conn, session_id, "assistant", content)
    }
}
