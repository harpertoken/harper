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

//! Tool execution module
//!
//! This module provides a unified interface to various tools
//! for file operations, shell commands, web search, and todos.

pub mod api;
pub mod code_analysis;
pub mod db;
pub mod filesystem;
pub mod git;
pub mod github;
pub mod image;
pub mod screenpipe;
pub mod shell;
pub mod todo;
pub mod web;

/// Common parsing utilities for tool arguments
pub mod parsing;

use crate::core::constants::tools;
use crate::core::error::{HarperError, HarperResult};
use crate::core::{ApiConfig, Message};
use crate::runtime::config::ExecPolicyConfig;
use crate::tools::shell::CommandAuditContext;
use reqwest::Client;
use rusqlite::Connection;
use serde_json::json;
use turul_mcp_client::{ContentBlock, McpClient};

// Git command constants
mod git_tools {
    pub const GIT_STATUS: &str = "[GIT_STATUS]";
    pub const GIT_DIFF: &str = "[GIT_DIFF]";
    pub const GIT_COMMIT: &str = "[GIT_COMMIT";
    pub const GIT_ADD: &str = "[GIT_ADD";
}

use crate::core::io_traits::UserApproval;
use std::sync::Arc;

/// Tool execution service
pub struct ToolService<'a> {
    conn: &'a Connection,
    config: &'a ApiConfig,
    exec_policy: &'a ExecPolicyConfig,
    mcp_client: Option<&'a turul_mcp_client::McpClient>,
    session_id: Option<&'a str>,
    approver: Option<Arc<dyn UserApproval>>,
}

impl<'a> ToolService<'a> {
    /// Create a new tool service
    pub fn new(
        conn: &'a Connection,
        config: &'a ApiConfig,
        exec_policy: &'a ExecPolicyConfig,
        mcp_client: Option<&'a McpClient>,
        session_id: Option<&'a str>,
    ) -> Self {
        Self {
            conn,
            config,
            exec_policy,
            mcp_client,
            session_id,
            approver: None,
        }
    }

    /// Set a custom user approval provider
    pub fn with_approver(mut self, approver: Arc<dyn UserApproval>) -> Self {
        self.approver = Some(approver);
        self
    }

    /// Handle tool usage (commands, web search, file operations)
    pub async fn handle_tool_use(
        &mut self,
        client: &Client,
        history: &[Message],
        response: &str,
        web_search_enabled: bool,
    ) -> Result<Option<(String, String)>, HarperError> {
        // Try to parse as JSON tool call first
        if let Ok(json_value) = serde_json::from_str::<serde_json::Value>(response) {
            // Case 1: OpenAI tool_calls format (array of objects)
            if let Some(tool_calls) = json_value.as_array() {
                if let Some(first_call) = tool_calls.first() {
                    let function = first_call.get("function");
                    let tool_name = function
                        .and_then(|f| f.get("name"))
                        .and_then(|v| v.as_str());

                    if let Some(name) = tool_name {
                        // Extract arguments - OpenAI uses a JSON string for arguments
                        let args_val = function.and_then(|f| f.get("arguments"));
                        let args_json = if let Some(serde_json::Value::String(s)) = args_val {
                            serde_json::from_str::<serde_json::Value>(s).unwrap_or(json!({}))
                        } else {
                            args_val.cloned().unwrap_or(json!({}))
                        };

                        // Check if it's an MCP tool
                        if name.starts_with("mcp__") {
                            return self
                                .handle_mcp_tool_call(client, history, name, &args_json, response)
                                .await;
                        }

                        return self
                            .handle_regular_json_tool(client, history, name, &args_json, response)
                            .await;
                    }
                }
            }

            // Case 2: Gemini or custom single-object format
            // Check for MCP tool call first
            if let Some(mcp_tool_name) = json_value.get("mcp_tool").and_then(|v| v.as_str()) {
                let args = json_value.get("arguments").cloned().unwrap_or(json!({}));
                return self
                    .handle_mcp_tool_call(client, history, mcp_tool_name, &args, response)
                    .await;
            }

            // Regular tool call - check both "tool" and "name"
            let tool_name = json_value
                .get("tool")
                .and_then(|v| v.as_str())
                .or_else(|| json_value.get("name").and_then(|v| v.as_str()));

            if let Some(name) = tool_name {
                let args = json_value
                    .get("args")
                    .or_else(|| json_value.get("arguments"));
                let args_json = args.cloned().unwrap_or(json!({}));

                // Handle possible mcp__ prefix in Gemini format too
                if name.starts_with("mcp__") {
                    return self
                        .handle_mcp_tool_call(client, history, name, &args_json, response)
                        .await;
                }

                return self
                    .handle_regular_json_tool(client, history, name, &args_json, response)
                    .await;
            }
        }

        // Fallback to old bracket format
        if response.to_uppercase().starts_with(tools::RUN_COMMAND) {
            let audit_ctx = CommandAuditContext {
                conn: self.conn,
                session_id: self.session_id,
                source: "tool_run_command",
            };
            let command_result = shell::execute_command(
                response,
                self.config,
                self.exec_policy,
                Some(&audit_ctx),
                self.approver.clone(),
            )
            .await?;
            let final_response = self
                .call_llm_after_tool(client, history, response, &command_result)
                .await?;
            Ok(Some((final_response, command_result)))
        } else if response.to_uppercase().starts_with(tools::READ_FILE) {
            let tool_result = filesystem::read_file(response, self.approver.clone()).await?;
            let final_response = self
                .call_llm_after_tool(client, history, response, &tool_result)
                .await?;
            Ok(Some((final_response, tool_result)))
        } else if response.to_uppercase().starts_with(tools::WRITE_FILE) {
            let tool_result = filesystem::write_file(response, self.approver.clone()).await?;
            let final_response = self
                .call_llm_after_tool(client, history, response, &tool_result)
                .await?;
            Ok(Some((final_response, tool_result)))
        } else if response.to_uppercase().starts_with(tools::SEARCH_REPLACE) {
            let tool_result = filesystem::search_replace(response, self.approver.clone()).await?;
            let final_response = self
                .call_llm_after_tool(client, history, response, &tool_result)
                .await?;
            Ok(Some((final_response, tool_result)))
        } else if response.to_uppercase().starts_with(tools::TODO) {
            self.execute_sync_tool(client, history, response, |conn, response| {
                let conn = conn.ok_or_else(|| {
                    HarperError::Database("Connection required for todo management".to_string())
                })?;
                todo::manage_todo(conn, response)
            })
            .await
        } else if web_search_enabled && response.to_uppercase().starts_with(tools::SEARCH) {
            let search_result = web::perform_web_search(response).await?;
            let final_response = self
                .call_llm_after_tool(client, history, response, &search_result)
                .await?;
            Ok(Some((final_response, search_result)))
        } else if response.to_uppercase().starts_with(git_tools::GIT_STATUS) {
            self.execute_sync_tool(client, history, response, |_, _| git::git_status())
                .await
        } else if response.to_uppercase().starts_with(git_tools::GIT_DIFF) {
            self.execute_sync_tool(client, history, response, |_, _| git::git_diff())
                .await
        } else if response.to_uppercase().starts_with(git_tools::GIT_COMMIT) {
            let commit_result = git::git_commit(response, self.approver.clone()).await?;
            let final_response = self
                .call_llm_after_tool(client, history, response, &commit_result)
                .await?;
            Ok(Some((final_response, commit_result)))
        } else if response.to_uppercase().starts_with(git_tools::GIT_ADD) {
            let add_result = git::git_add(response, self.approver.clone()).await?;
            let final_response = self
                .call_llm_after_tool(client, history, response, &add_result)
                .await?;
            Ok(Some((final_response, add_result)))
        } else if response.to_uppercase().starts_with(tools::GITHUB_ISSUE) {
            self.execute_sync_tool(client, history, response, |_, response| {
                github::create_issue(response)
            })
            .await
        } else if response.to_uppercase().starts_with(tools::GITHUB_PR) {
            self.execute_sync_tool(client, history, response, |_, response| {
                github::create_pr(response)
            })
            .await
        } else if response.to_uppercase().starts_with(tools::API_TEST) {
            let api_result = api::test_api(response).await?;
            let final_response = self
                .call_llm_after_tool(client, history, response, &api_result)
                .await?;
            Ok(Some((final_response, api_result)))
        } else if response.to_uppercase().starts_with(tools::CODE_ANALYZE) {
            self.execute_sync_tool(client, history, response, |_, response| {
                code_analysis::analyze_code(response)
            })
            .await
        } else if response.to_uppercase().starts_with(tools::DB_QUERY) {
            let tool_result = db::run_query(response, self.approver.clone()).await?;
            let final_response = self
                .call_llm_after_tool(client, history, response, &tool_result)
                .await?;
            Ok(Some((final_response, tool_result)))
        } else if response.to_uppercase().starts_with(tools::IMAGE_INFO) {
            self.execute_sync_tool(client, history, response, |_, response| {
                image::get_image_info(response)
            })
            .await
        } else if response.to_uppercase().starts_with(tools::IMAGE_RESIZE) {
            self.execute_sync_tool(client, history, response, |_, response| {
                image::resize_image(response)
            })
            .await
        } else if response.to_uppercase().starts_with(tools::SCREENPIPE) {
            let search_result = screenpipe::search_screenpipe(response).await?;
            let final_response = self
                .call_llm_after_tool(client, history, response, &search_result)
                .await?;
            Ok(Some((final_response, search_result)))
        } else {
            Ok(None)
        }
    }

    #[allow(dead_code)]
    async fn handle_regular_json_tool(
        &mut self,
        client: &Client,
        history: &[Message],
        tool_name: &str,
        args: &serde_json::Value,
        raw_response: &str,
    ) -> Result<Option<(String, String)>, HarperError> {
        match tool_name {
            "run_command" => {
                if let Some(command) = args.get("command").and_then(|v| v.as_str()) {
                    let bracket_command = format!("[RUN_COMMAND {}]", command);
                    let audit_ctx = CommandAuditContext {
                        conn: self.conn,
                        session_id: self.session_id,
                        source: "tool_run_command",
                    };
                    let command_result = shell::execute_command(
                        &bracket_command,
                        self.config,
                        self.exec_policy,
                        Some(&audit_ctx),
                        self.approver.clone(),
                    )
                    .await?;
                    let final_response = self
                        .call_llm_after_tool(client, history, raw_response, &command_result)
                        .await?;
                    Ok(Some((final_response, command_result)))
                } else {
                    Ok(None)
                }
            }

            "read_file" => {
                if let Some(path) = args.get("path").and_then(|v| v.as_str()) {
                    let bracket_command = format!("[READ_FILE {}]", path);
                    let read_result =
                        filesystem::read_file(&bracket_command, self.approver.clone()).await?;
                    let final_response = self
                        .call_llm_after_tool(client, history, raw_response, &read_result)
                        .await?;
                    Ok(Some((final_response, read_result)))
                } else {
                    Ok(None)
                }
            }

            "write_file" => {
                let path = args.get("path").and_then(|v| v.as_str());
                let content = args.get("content").and_then(|v| v.as_str());
                if let (Some(path), Some(content)) = (path, content) {
                    let bracket_command = format!("[WRITE_FILE {} {}]", path, content);
                    let write_result =
                        filesystem::write_file(&bracket_command, self.approver.clone()).await?;
                    let final_response = self
                        .call_llm_after_tool(client, history, raw_response, &write_result)
                        .await?;
                    Ok(Some((final_response, write_result)))
                } else {
                    Ok(None)
                }
            }
            "search_replace" => {
                let path = args.get("path").and_then(|v| v.as_str());
                let old_string = args.get("old_string").and_then(|v| v.as_str());
                let new_string = args.get("new_string").and_then(|v| v.as_str());
                if let (Some(path), Some(old_string), Some(new_string)) =
                    (path, old_string, new_string)
                {
                    let bracket_command =
                        format!("[SEARCH_REPLACE {} {} {}]", path, old_string, new_string);
                    let replace_result =
                        filesystem::search_replace(&bracket_command, self.approver.clone()).await?;
                    let final_response = self
                        .call_llm_after_tool(client, history, raw_response, &replace_result)
                        .await?;
                    Ok(Some((final_response, replace_result)))
                } else {
                    Ok(None)
                }
            }
            "todo" => {
                if let Some(action) = args.get("action").and_then(|v| v.as_str()) {
                    let bracket_command = match action {
                        "add" => {
                            if let Some(description) =
                                args.get("description").and_then(|v| v.as_str())
                            {
                                format!("[TODO add {}]", description)
                            } else {
                                "".to_string()
                            }
                        }
                        "list" => "[TODO list]".to_string(),
                        "remove" => {
                            if let Some(index) = args.get("index").and_then(|v| v.as_i64()) {
                                format!("[TODO remove {}]", index)
                            } else {
                                "".to_string()
                            }
                        }
                        "clear" => "[TODO clear]".to_string(),
                        _ => "".to_string(),
                    };
                    if bracket_command.is_empty() {
                        return Ok(None);
                    }
                    let todo_result = todo::manage_todo(self.conn, &bracket_command)?;
                    let final_response = self
                        .call_llm_after_tool(client, history, raw_response, &todo_result)
                        .await?;
                    Ok(Some((final_response, todo_result)))
                } else {
                    Ok(None)
                }
            }
            "git_status" => {
                let status_result = git::git_status()?;
                let final_response = self
                    .call_llm_after_tool(client, history, raw_response, &status_result)
                    .await?;
                Ok(Some((final_response, status_result)))
            }
            "git_diff" => {
                let diff_result = git::git_diff()?;
                let final_response = self
                    .call_llm_after_tool(client, history, raw_response, &diff_result)
                    .await?;
                Ok(Some((final_response, diff_result)))
            }
            "git_add" => {
                let files = args.get("files").and_then(|v| v.as_str());
                let bracket_command = format!("[GIT_ADD {}]", files.unwrap_or("."));
                let add_result = git::git_add(&bracket_command, self.approver.clone()).await?;
                let final_response = self
                    .call_llm_after_tool(client, history, raw_response, &add_result)
                    .await?;
                Ok(Some((final_response, add_result)))
            }
            "git_commit" => {
                if let Some(message) = args.get("message").and_then(|v| v.as_str()) {
                    let bracket_command = format!("[GIT_COMMIT {}]", message);
                    let commit_result =
                        git::git_commit(&bracket_command, self.approver.clone()).await?;
                    let final_response = self
                        .call_llm_after_tool(client, history, raw_response, &commit_result)
                        .await?;
                    Ok(Some((final_response, commit_result)))
                } else {
                    Ok(None)
                }
            }
            "list_changed_files" => {
                let ext = args.get("ext").and_then(|v| v.as_str());
                let tracked_only = args
                    .get("tracked_only")
                    .and_then(|v| v.as_bool())
                    .unwrap_or(false);
                let since = args.get("since").and_then(|v| v.as_str());

                let audit_ctx = CommandAuditContext {
                    conn: self.conn,
                    session_id: self.session_id,
                    source: "tool_list_changed_files",
                };
                let files_result = git::list_changed_files_with_policy(
                    self.config,
                    self.exec_policy,
                    Some(&audit_ctx),
                    self.approver.clone(),
                    ext,
                    tracked_only,
                    since,
                )
                .await?;
                let final_response = self
                    .call_llm_after_tool(client, history, raw_response, &files_result)
                    .await?;
                Ok(Some((final_response, files_result)))
            }
            _ => Ok(None),
        }
    }

    /// Handle MCP tool calls
    async fn handle_mcp_tool_call(
        &mut self,
        client: &Client,
        history: &[Message],
        tool_name: &str,
        args: &serde_json::Value,
        raw_response: &str,
    ) -> Result<Option<(String, String)>, HarperError> {
        let Some(mcp_client) = self.mcp_client else {
            let error_msg = "Error: MCP client not configured".to_string();
            let final_response = self
                .call_llm_after_tool(client, history, raw_response, &error_msg)
                .await?;
            return Ok(Some((final_response, error_msg)));
        };

        match mcp_client.call_tool(tool_name, args.clone()).await {
            Ok(results) => {
                // Format the response for the LLM
                let mut result_parts = Vec::new();

                for result in &results {
                    match result {
                        ContentBlock::Text { text, .. } => {
                            result_parts.push(text.clone());
                        }
                        ContentBlock::Image {
                            data, mime_type, ..
                        } => {
                            result_parts.push(format!(
                                "[Image: {} bytes, type: {}]",
                                data.len(),
                                mime_type
                            ));
                        }
                        ContentBlock::Audio { .. } => {
                            result_parts.push("[Audio content]".to_string());
                        }
                        ContentBlock::ResourceLink { .. } => {
                            result_parts.push("[Resource link]".to_string());
                        }
                        ContentBlock::Resource { .. } => {
                            result_parts.push("[Resource content]".to_string());
                        }
                    }
                }

                let tool_result = if result_parts.is_empty() {
                    "Tool executed successfully (no output)".to_string()
                } else {
                    result_parts.join("\n")
                };

                let final_response = self
                    .call_llm_after_tool(client, history, raw_response, &tool_result)
                    .await?;
                Ok(Some((final_response, tool_result)))
            }
            Err(e) => {
                let error_msg = format!("MCP tool call failed: {}", e);
                let final_response = self
                    .call_llm_after_tool(client, history, raw_response, &error_msg)
                    .await?;
                Ok::<Option<(String, String)>, HarperError>(Some((final_response, error_msg)))
            }
        }
    }

    /// Execute a synchronous tool
    async fn execute_sync_tool<F>(
        &mut self,
        client: &Client,
        history: &[Message],
        response: &str,
        tool_fn: F,
    ) -> Result<Option<(String, String)>, HarperError>
    where
        F: FnOnce(Option<&Connection>, &str) -> HarperResult<String>,
    {
        let tool_result = tool_fn(Some(self.conn), response)?;
        let final_response = self
            .call_llm_after_tool(client, history, response, &tool_result)
            .await?;
        Ok(Some((final_response, tool_result)))
    }

    /// Call LLM after tool usage
    async fn call_llm_after_tool(
        &self,
        client: &Client,
        history: &[Message],
        tool_call_json: &str,
        tool_output: &str,
    ) -> Result<String, HarperError> {
        // Create a new history vector by cloning the existing one
        let mut new_history = history.to_vec();

        // Add the assistant's tool call message
        // This ensures the model knows it just asked for this tool
        new_history.push(Message {
            role: "assistant".to_string(),
            content: tool_call_json.to_string(),
        });

        // Add a new system message containing the tool's output and an instruction
        new_history.push(Message {
            role: "system".to_string(),
            content: format!(
                "Tool execution result:
{}

---
SYSTEM INSTRUCTION: The tool has completed successfully. The output above is the result.
1. DO NOT output the tool call JSON again.
2. DO NOT repeat the raw output.
3. Analyze the result and provide a human-readable answer to the user's request.
4. If the user asked to see a file, summarize it or show a relevant snippet, but do not dump the entire content if it is large.",
                tool_output
            ),
        });

        // Call the LLM with the new history. If that fails (quota, blocked API, etc.),
        // fall back to returning the tool output directly so the user still gets a result.
        match crate::core::llm_client::call_llm(client, self.config, &new_history).await {
            Ok(response) => Ok(response),
            Err(_) => Ok(format!("Tool result:\n{}", tool_output)),
        }
    }
}
