//! Tool execution module
//!
//! This module provides a unified interface to various tools
//! for file operations, shell commands, web search, and todos.

pub mod filesystem;
pub mod git;
pub mod shell;
pub mod todo;
pub mod web;

/// Common parsing utilities for tool arguments
pub mod parsing;

use crate::core::constants::tools;
use crate::core::error::HarperError;
use crate::core::{ApiConfig, Message};
use reqwest::Client;

// Git command constants
mod git_tools {
    pub const GIT_STATUS: &str = "[GIT_STATUS]";
    pub const GIT_DIFF: &str = "[GIT_DIFF]";
    pub const GIT_COMMIT: &str = "[GIT_COMMIT";
    pub const GIT_ADD: &str = "[GIT_ADD";
}

/// Tool execution service
pub struct ToolService<'a> {
    config: &'a ApiConfig,
}

impl<'a> ToolService<'a> {
    /// Create a new tool service
    pub fn new(config: &'a ApiConfig) -> Self {
        Self { config }
    }

    /// Handle tool usage (commands, web search, file operations)
    pub async fn handle_tool_use(
        &mut self,
        client: &Client,
        history: &[Message],
        response: &str,
        web_search_enabled: bool,
    ) -> Result<Option<(String, String)>, HarperError> {
        if response.to_uppercase().starts_with(tools::RUN_COMMAND) {
            let command_result = shell::execute_command(response, self.config)?;
            let final_response = self
                .call_llm_after_tool(client, history, &command_result)
                .await?;
            Ok(Some((final_response, command_result)))
        } else if response.to_uppercase().starts_with(tools::READ_FILE) {
            let file_content = filesystem::read_file(response)?;
            let final_response = self
                .call_llm_after_tool(client, history, &file_content)
                .await?;
            Ok(Some((final_response, file_content)))
        } else if response.to_uppercase().starts_with(tools::WRITE_FILE) {
            let write_result = filesystem::write_file(response)?;
            let final_response = self
                .call_llm_after_tool(client, history, &write_result)
                .await?;
            Ok(Some((final_response, write_result)))
        } else if response.to_uppercase().starts_with(tools::SEARCH_REPLACE) {
            let replace_result = filesystem::search_replace(response)?;
            let final_response = self
                .call_llm_after_tool(client, history, &replace_result)
                .await?;
            Ok(Some((final_response, replace_result)))
        } else if response.to_uppercase().starts_with(tools::TODO) {
            let todo_result = todo::manage_todo(response)?;
            let final_response = self
                .call_llm_after_tool(client, history, &todo_result)
                .await?;
            Ok(Some((final_response, todo_result)))
        } else if web_search_enabled && response.to_uppercase().starts_with(tools::SEARCH) {
            let search_result = web::perform_web_search(response).await?;
            let final_response = self
                .call_llm_after_tool(client, history, &search_result)
                .await?;
            Ok(Some((final_response, search_result)))
        } else if response.to_uppercase().starts_with(git_tools::GIT_STATUS) {
            let status_result = git::git_status()?;
            let final_response = self
                .call_llm_after_tool(client, history, &status_result)
                .await?;
            Ok(Some((final_response, status_result)))
        } else if response.to_uppercase().starts_with(git_tools::GIT_DIFF) {
            let diff_result = git::git_diff()?;
            let final_response = self
                .call_llm_after_tool(client, history, &diff_result)
                .await?;
            Ok(Some((final_response, diff_result)))
        } else if response.to_uppercase().starts_with(git_tools::GIT_COMMIT) {
            let commit_result = git::git_commit(response)?;
            let final_response = self
                .call_llm_after_tool(client, history, &commit_result)
                .await?;
            Ok(Some((final_response, commit_result)))
        } else if response.to_uppercase().starts_with(git_tools::GIT_ADD) {
            let add_result = git::git_add(response)?;
            let final_response = self
                .call_llm_after_tool(client, history, &add_result)
                .await?;
            Ok(Some((final_response, add_result)))
        } else {
            Ok(None)
        }
    }

    /// Call LLM after tool usage
    async fn call_llm_after_tool(
        &self,
        client: &Client,
        history: &[Message],
        tool_output: &str,
    ) -> Result<String, HarperError> {
        // Create a new history vector by cloning the existing one
        let mut new_history = history.to_vec();

        // Add a new system message containing the tool's output
        new_history.push(Message {
            role: "system".to_string(),
            content: format!("Tool execution result: {}", tool_output),
        });

        // Call the LLM with the new history
        crate::core::llm_client::call_llm(client, self.config, &new_history).await
    }
}
