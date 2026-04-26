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

//! Tool execution module
//!
//! This module provides a unified interface to various tools
//! for file operations, shell commands, web search, and todos.

pub mod api;
pub mod code_analysis;
pub mod codebase_investigator;
pub mod db;
pub mod filesystem;
pub mod firmware;
pub mod git;
pub mod github;
pub mod image;
pub mod plan;
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
use std::path::PathBuf;
use turul_mcp_client::{ContentBlock, McpClient};

// Git command constants
mod git_tools {
    pub const GIT_STATUS: &str = "[GIT_STATUS]";
    pub const GIT_DIFF: &str = "[GIT_DIFF]";
    pub const GIT_COMMIT: &str = "[GIT_COMMIT";
    pub const GIT_ADD: &str = "[GIT_ADD";
}

use crate::core::io_traits::{RuntimeEventSink, UserApproval};
use std::sync::Arc;

/// Tool execution service
pub struct ToolService<'a> {
    conn: &'a Connection,
    config: &'a ApiConfig,
    exec_policy: &'a ExecPolicyConfig,
    mcp_client: Option<&'a turul_mcp_client::McpClient>,
    session_id: Option<&'a str>,
    approver: Option<Arc<dyn UserApproval>>,
    runtime_events: Option<Arc<dyn RuntimeEventSink>>,
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
            runtime_events: None,
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

    fn emit_activity_update(&self, status: Option<String>) {
        let (Some(runtime_events), Some(session_id)) = (&self.runtime_events, self.session_id)
        else {
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
            self.sync_plan_before_tool("run_command")?;
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
                self.runtime_events.clone(),
            )
            .await?;
            let final_response = self
                .call_llm_after_tool(client, history, response, &command_result)
                .await?;
            Ok(Some((final_response, command_result)))
        } else if response.to_uppercase().starts_with(tools::READ_FILE) {
            self.sync_plan_before_tool("read_file")?;
            let tool_result = filesystem::read_file(response, self.approver.clone()).await?;
            let final_response = self
                .call_llm_after_tool(client, history, response, &tool_result)
                .await?;
            Ok(Some((final_response, tool_result)))
        } else if response.to_uppercase().starts_with(tools::WRITE_FILE) {
            self.sync_plan_before_tool("write_file")?;
            let tool_result = filesystem::write_file(response, self.approver.clone()).await?;
            let final_response = self
                .call_llm_after_tool(client, history, response, &tool_result)
                .await?;
            Ok(Some((final_response, tool_result)))
        } else if response.to_uppercase().starts_with(tools::SEARCH_REPLACE) {
            self.sync_plan_before_tool("search_replace")?;
            let tool_result = filesystem::search_replace(response, self.approver.clone()).await?;
            let final_response = self
                .call_llm_after_tool(client, history, response, &tool_result)
                .await?;
            Ok(Some((final_response, tool_result)))
        } else if response.to_uppercase().starts_with(tools::TODO) {
            self.sync_plan_before_tool("todo")?;
            self.execute_sync_tool(client, history, response, |conn, response| {
                let conn = conn.ok_or_else(|| {
                    HarperError::Database("Connection required for todo management".to_string())
                })?;
                todo::manage_todo(conn, response)
            })
            .await
        } else if web_search_enabled && response.to_uppercase().starts_with(tools::SEARCH) {
            self.sync_plan_before_tool("search")?;
            let search_result = web::perform_web_search(response).await?;
            let final_response = self
                .call_llm_after_tool(client, history, response, &search_result)
                .await?;
            Ok(Some((final_response, search_result)))
        } else if response.to_uppercase().starts_with(git_tools::GIT_STATUS) {
            self.sync_plan_before_tool("git_status")?;
            self.execute_sync_tool(client, history, response, |_, _| git::git_status())
                .await
        } else if response.to_uppercase().starts_with(git_tools::GIT_DIFF) {
            self.sync_plan_before_tool("git_diff")?;
            self.execute_sync_tool(client, history, response, |_, _| git::git_diff())
                .await
        } else if response.to_uppercase().starts_with(git_tools::GIT_COMMIT) {
            self.sync_plan_before_tool("git_commit")?;
            let commit_result = git::git_commit(response, self.approver.clone()).await?;
            let final_response = self
                .call_llm_after_tool(client, history, response, &commit_result)
                .await?;
            Ok(Some((final_response, commit_result)))
        } else if response.to_uppercase().starts_with(git_tools::GIT_ADD) {
            self.sync_plan_before_tool("git_add")?;
            let add_result = git::git_add(response, self.approver.clone()).await?;
            let final_response = self
                .call_llm_after_tool(client, history, response, &add_result)
                .await?;
            Ok(Some((final_response, add_result)))
        } else if response.to_uppercase().starts_with(tools::GITHUB_ISSUE) {
            self.sync_plan_before_tool("github_issue")?;
            self.execute_sync_tool(client, history, response, |_, response| {
                github::create_issue(response)
            })
            .await
        } else if response.to_uppercase().starts_with(tools::GITHUB_PR) {
            self.sync_plan_before_tool("github_pr")?;
            self.execute_sync_tool(client, history, response, |_, response| {
                github::create_pr(response)
            })
            .await
        } else if response.to_uppercase().starts_with(tools::API_TEST) {
            self.sync_plan_before_tool("api_test")?;
            let api_result = api::test_api(response).await?;
            let final_response = self
                .call_llm_after_tool(client, history, response, &api_result)
                .await?;
            Ok(Some((final_response, api_result)))
        } else if response.to_uppercase().starts_with(tools::CODE_ANALYZE) {
            self.sync_plan_before_tool("code_analyze")?;
            self.execute_sync_tool(client, history, response, |_, response| {
                code_analysis::analyze_code(response)
            })
            .await
        } else if response
            .to_uppercase()
            .starts_with(tools::CODEBASE_INVESTIGATE)
        {
            self.sync_plan_before_tool("codebase_investigator")?;
            let tool_result =
                codebase_investigator::investigate_codebase(response, self.approver.clone())
                    .await?;
            let final_response = self
                .call_llm_after_tool(client, history, response, &tool_result)
                .await?;
            Ok(Some((final_response, tool_result)))
        } else if response.to_uppercase().starts_with(tools::DB_QUERY) {
            self.sync_plan_before_tool("db_query")?;
            let tool_result = db::run_query(response, self.approver.clone()).await?;
            let final_response = self
                .call_llm_after_tool(client, history, response, &tool_result)
                .await?;
            Ok(Some((final_response, tool_result)))
        } else if response.to_uppercase().starts_with(tools::IMAGE_INFO) {
            self.sync_plan_before_tool("image_info")?;
            self.execute_sync_tool(client, history, response, |_, response| {
                image::get_image_info(response)
            })
            .await
        } else if response.to_uppercase().starts_with(tools::IMAGE_RESIZE) {
            self.sync_plan_before_tool("image_resize")?;
            self.execute_sync_tool(client, history, response, |_, response| {
                image::resize_image(response)
            })
            .await
        } else if response.to_uppercase().starts_with(tools::SCREENPIPE) {
            self.sync_plan_before_tool("screenpipe")?;
            let search_result = screenpipe::search_screenpipe(response).await?;
            let final_response = self
                .call_llm_after_tool(client, history, response, &search_result)
                .await?;
            Ok(Some((final_response, search_result)))
        } else if response.to_uppercase().starts_with(tools::FIRMWARE) {
            self.sync_plan_before_tool("firmware")?;
            self.execute_sync_tool(client, history, response, |_, response| {
                firmware::handle_firmware_command(response)
            })
            .await
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
        self.sync_plan_before_tool(tool_name)?;
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
                        self.runtime_events.clone(),
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
            "update_plan" => {
                let Some(session_id) = self.session_id else {
                    return Err(HarperError::Validation(
                        "update_plan requires an active session".to_string(),
                    ));
                };
                let plan_result = plan::update_plan(self.conn, session_id, args)?;
                let final_response = self
                    .call_llm_after_tool(client, history, raw_response, &plan_result)
                    .await?;
                Ok(Some((final_response, plan_result)))
            }
            "codebase_investigator" => {
                let action = args.get("action").and_then(|v| v.as_str());
                if let Some(act) = action {
                    let mut bracket_command = format!("[CODEBASE_INVESTIGATE {}", act);
                    match act {
                        "find_calls" => {
                            if let Some(symbol) = args.get("symbol").and_then(|v| v.as_str()) {
                                bracket_command.push_str(&format!(" {}]", symbol));
                            }
                        }
                        "trace_relationship" => {
                            let x = args.get("x").and_then(|v| v.as_str());
                            let y = args.get("y").and_then(|v| v.as_str());
                            if let (Some(x), Some(y)) = (x, y) {
                                bracket_command.push_str(&format!(" {} {}]", x, y));
                            }
                        }
                        "clone_context" => {
                            if let Some(repo_url) = args.get("repo_url").and_then(|v| v.as_str()) {
                                bracket_command.push_str(&format!(" {}]", repo_url));
                            }
                        }
                        _ => {}
                    }
                    if bracket_command.ends_with(']') {
                        let tool_result = codebase_investigator::investigate_codebase(
                            &bracket_command,
                            self.approver.clone(),
                        )
                        .await?;
                        let final_response = self
                            .call_llm_after_tool(client, history, raw_response, &tool_result)
                            .await?;
                        return Ok(Some((final_response, tool_result)));
                    }
                }
                Ok(None)
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
            "firmware_list" => {
                let result = firmware::handle_firmware_command("[FIRMWARE list]")?;
                let final_response = self
                    .call_llm_after_tool(client, history, raw_response, &result)
                    .await?;
                Ok(Some((final_response, result)))
            }
            "firmware_info" => {
                if let Some(device) = args.get("device").and_then(|v| v.as_str()) {
                    let command = format!("[FIRMWARE info {}]", device);
                    let result = firmware::handle_firmware_command(&command)?;
                    let final_response = self
                        .call_llm_after_tool(client, history, raw_response, &result)
                        .await?;
                    Ok(Some((final_response, result)))
                } else {
                    Ok(None)
                }
            }
            "firmware_gpio" => {
                if let Some(pin) = args.get("pin").and_then(|v| v.as_i64()) {
                    let state = args.get("state").and_then(|v| v.as_str()).unwrap_or("high");
                    let command = format!("[FIRMWARE gpio {} {}]", pin, state);
                    let result = firmware::handle_firmware_command(&command)?;
                    let final_response = self
                        .call_llm_after_tool(client, history, raw_response, &result)
                        .await?;
                    Ok(Some((final_response, result)))
                } else {
                    Ok(None)
                }
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
            Ok(result) => {
                // Format the response for the LLM
                let mut result_parts = Vec::new();

                for content in result.content {
                    match content {
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
                        ContentBlock::ToolUse { .. } => {
                            result_parts.push("[Tool use]".to_string());
                        }
                        ContentBlock::ToolResult { .. } => {
                            result_parts.push("[Tool result]".to_string());
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
        self.emit_activity_update(Some("thinking".to_string()));
        self.sync_plan_after_tool(tool_call_json)?;
        // Create a new history vector by cloning the existing one
        let mut new_history = history.to_vec();

        // Add the assistant's tool call message
        // This ensures the model knows it just asked for this tool
        new_history.push(Message {
            role: "assistant".to_string(),
            content: tool_call_json.to_string(),
        });

        // Add a new system message containing the tool's output and an instruction
        let mut system_message = format!(
            "Tool execution result:
{}

---
SYSTEM INSTRUCTION: The tool has completed successfully. The output above is the result.
1. DO NOT output the tool call JSON again.
2. DO NOT repeat the raw output.
3. Analyze the result and provide a human-readable answer to the user's request.
4. If the user asked to see a file, summarize it or show a relevant snippet, but do not dump the entire content if it is large.",
            tool_output
        );
        if !Self::is_update_plan_call(tool_call_json) {
            if let Some(plan_instruction) = self.plan_followup_instruction()? {
                system_message.push_str("\n5. ");
                system_message.push_str(&plan_instruction);
            }
        }
        new_history.push(Message {
            role: "system".to_string(),
            content: system_message,
        });

        // Call the LLM with the new history. If that fails (quota, blocked API, etc.),
        // fall back to returning the tool output directly so the user still gets a result.
        match crate::core::llm_client::call_llm(client, self.config, &new_history).await {
            Ok(response) => Ok(response),
            Err(_) => Ok(format!("Tool result:\n{}", tool_output)),
        }
    }

    fn sync_plan_before_tool(&self, tool_name: &str) -> HarperResult<()> {
        if tool_name == "update_plan" {
            return Ok(());
        }

        self.emit_activity_update(Some(format!("running {}", tool_name)));

        let Some(session_id) = self.session_id else {
            return Ok(());
        };
        let Some(mut plan) = crate::memory::storage::load_plan_state(self.conn, session_id)? else {
            return Ok(());
        };
        if plan.items.is_empty() {
            return Ok(());
        }
        if plan
            .items
            .iter()
            .any(|item| matches!(item.status, crate::core::plan::PlanStepStatus::InProgress))
        {
            plan.runtime = Some(crate::core::plan::PlanRuntime {
                active_tool: Some(tool_name.to_string()),
                active_command: None,
                active_status: Some("running".to_string()),
            });
            crate::memory::storage::save_plan_state(self.conn, session_id, &plan)?;
            return Ok(());
        }

        if let Some(first_pending) = plan
            .items
            .iter_mut()
            .find(|item| matches!(item.status, crate::core::plan::PlanStepStatus::Pending))
        {
            first_pending.status = crate::core::plan::PlanStepStatus::InProgress;
        }
        plan.runtime = Some(crate::core::plan::PlanRuntime {
            active_tool: Some(tool_name.to_string()),
            active_command: None,
            active_status: Some("running".to_string()),
        });
        crate::memory::storage::save_plan_state(self.conn, session_id, &plan)?;

        Ok(())
    }

    fn sync_plan_after_tool(&self, tool_call_json: &str) -> HarperResult<()> {
        let Some(session_id) = self.session_id else {
            return Ok(());
        };
        let Some(mut plan) = crate::memory::storage::load_plan_state(self.conn, session_id)? else {
            return Ok(());
        };
        let Some(current_index) = plan
            .items
            .iter()
            .position(|item| matches!(item.status, crate::core::plan::PlanStepStatus::InProgress))
        else {
            return Ok(());
        };

        let tool_name = Self::tool_name_from_call(tool_call_json);
        let Some(tool_name) = tool_name else {
            return Ok(());
        };
        if !Self::is_safe_auto_advance_tool(&tool_name) {
            if plan.runtime.is_some() {
                plan.runtime = None;
                crate::memory::storage::save_plan_state(self.conn, session_id, &plan)?;
            }
            return Ok(());
        }

        let step_text = plan.items[current_index].step.to_ascii_lowercase();
        if !Self::step_matches_safe_tool(&step_text, &tool_name) {
            if plan.runtime.is_some() {
                plan.runtime = None;
                crate::memory::storage::save_plan_state(self.conn, session_id, &plan)?;
            }
            return Ok(());
        }

        plan.items[current_index].status = crate::core::plan::PlanStepStatus::Completed;
        if let Some(next_pending) = plan
            .items
            .iter_mut()
            .find(|item| matches!(item.status, crate::core::plan::PlanStepStatus::Pending))
        {
            next_pending.status = crate::core::plan::PlanStepStatus::InProgress;
        }
        plan.runtime = None;
        crate::memory::storage::save_plan_state(self.conn, session_id, &plan)?;
        Ok(())
    }

    fn plan_followup_instruction(&self) -> HarperResult<Option<String>> {
        let Some(session_id) = self.session_id else {
            return Ok(None);
        };
        let Some(plan) = crate::memory::storage::load_plan_state(self.conn, session_id)? else {
            return Ok(None);
        };
        if plan.items.is_empty() {
            return Ok(None);
        }
        let has_remaining = plan
            .items
            .iter()
            .any(|item| !matches!(item.status, crate::core::plan::PlanStepStatus::Completed));
        if !has_remaining {
            return Ok(None);
        }

        Ok(Some(
            "A session plan is active. If this tool meaningfully advanced the work, call update_plan to mark the current step completed or move the next step to in_progress.".to_string(),
        ))
    }

    fn is_update_plan_call(tool_call_json: &str) -> bool {
        tool_call_json.contains("\"update_plan\"") || tool_call_json.contains("'update_plan'")
    }

    pub fn target_paths_for_tool_call(tool_call: &str) -> Vec<PathBuf> {
        let trimmed = tool_call.trim();
        if let Ok(json_value) = serde_json::from_str::<serde_json::Value>(trimmed) {
            let tool_name = json_value
                .get("tool")
                .and_then(|v| v.as_str())
                .or_else(|| json_value.get("name").and_then(|v| v.as_str()));
            let args = json_value
                .get("args")
                .or_else(|| json_value.get("arguments"))
                .cloned()
                .unwrap_or(serde_json::Value::Null);
            return extract_target_paths_from_json(tool_name, &args);
        }

        let upper = trimmed.to_ascii_uppercase();
        if upper.starts_with("[READ_FILE") {
            return parsing::extract_tool_arg(trimmed, "[READ_FILE")
                .map(|path| vec![PathBuf::from(path)])
                .unwrap_or_default();
        }
        if upper.starts_with("[WRITE_FILE") {
            return parsing::extract_tool_args(trimmed, "[WRITE_FILE", 2)
                .map(|args| vec![PathBuf::from(args[0].clone())])
                .unwrap_or_default();
        }
        if upper.starts_with("[SEARCH_REPLACE") {
            return parsing::extract_tool_args(trimmed, "[SEARCH_REPLACE", 3)
                .map(|args| vec![PathBuf::from(args[0].clone())])
                .unwrap_or_default();
        }

        Vec::new()
    }

    fn tool_name_from_call(tool_call_json: &str) -> Option<String> {
        let trimmed = tool_call_json.trim();
        if let Ok(json_value) = serde_json::from_str::<serde_json::Value>(trimmed) {
            if let Some(name) = json_value
                .get("tool")
                .and_then(|v| v.as_str())
                .or_else(|| json_value.get("name").and_then(|v| v.as_str()))
            {
                return Some(name.to_string());
            }
            if let Some(first) = json_value.as_array().and_then(|arr| arr.first()) {
                if let Some(name) = first
                    .get("function")
                    .and_then(|f| f.get("name"))
                    .and_then(|v| v.as_str())
                {
                    return Some(name.to_string());
                }
            }
        }

        let upper = trimmed.to_ascii_uppercase();
        if upper.starts_with("[READ_FILE") {
            Some("read_file".to_string())
        } else if upper.starts_with("[RUN_COMMAND") {
            Some("run_command".to_string())
        } else if upper.starts_with("[GIT_STATUS") {
            Some("git_status".to_string())
        } else if upper.starts_with("[GIT_DIFF") {
            Some("git_diff".to_string())
        } else if upper.starts_with("[SEARCH_REPLACE") {
            Some("search_replace".to_string())
        } else if upper.starts_with("[WRITE_FILE") {
            Some("write_file".to_string())
        } else {
            None
        }
    }

    fn is_safe_auto_advance_tool(tool_name: &str) -> bool {
        matches!(
            tool_name,
            "read_file"
                | "git_status"
                | "git_diff"
                | "list_changed_files"
                | "grep"
                | "codebase_investigator"
        )
    }

    fn step_matches_safe_tool(step_text: &str, tool_name: &str) -> bool {
        let inspect_words = [
            "inspect",
            "check",
            "review",
            "read",
            "look",
            "trace",
            "understand",
            "investigate",
            "explore",
            "find",
            "compare",
            "diff",
        ];
        if !inspect_words.iter().any(|word| step_text.contains(word)) {
            return false;
        }

        match tool_name {
            "read_file" => {
                step_text.contains("file")
                    || step_text.contains("read")
                    || step_text.contains("inspect")
            }
            "git_status" => step_text.contains("status") || step_text.contains("repo"),
            "git_diff" | "list_changed_files" => {
                step_text.contains("diff")
                    || step_text.contains("changed")
                    || step_text.contains("compare")
            }
            "grep" => {
                step_text.contains("find")
                    || step_text.contains("search")
                    || step_text.contains("grep")
            }
            "codebase_investigator" => {
                step_text.contains("trace")
                    || step_text.contains("investigate")
                    || step_text.contains("understand")
                    || step_text.contains("explore")
            }
            _ => false,
        }
    }
}

fn extract_target_paths_from_json(
    tool_name: Option<&str>,
    args: &serde_json::Value,
) -> Vec<PathBuf> {
    match tool_name {
        Some("read_file") | Some("write_file") | Some("search_replace") => args
            .get("path")
            .or_else(|| args.get("filePath"))
            .and_then(|v| v.as_str())
            .map(|path| vec![PathBuf::from(path)])
            .unwrap_or_default(),
        _ => Vec::new(),
    }
}

#[cfg(test)]
mod tests {
    use super::ToolService;
    use crate::core::plan::{PlanItem, PlanState, PlanStepStatus};
    use crate::core::{ApiConfig, ApiProvider};
    use crate::runtime::config::ExecPolicyConfig;
    use rusqlite::Connection;
    use std::path::PathBuf;

    fn test_config() -> ApiConfig {
        ApiConfig {
            provider: ApiProvider::OpenAI,
            api_key: "test-key".to_string(),
            base_url: "https://api.openai.com/v1/chat/completions".to_string(),
            model_name: "gpt-4".to_string(),
        }
    }

    #[test]
    fn sync_plan_before_tool_promotes_first_pending_step() {
        let conn = Connection::open_in_memory().expect("in-memory db");
        crate::memory::storage::init_db(&conn).expect("init db");
        crate::memory::storage::save_plan_state(
            &conn,
            "sync-plan-session",
            &PlanState {
                explanation: Some("Testing sync".to_string()),
                items: vec![
                    PlanItem {
                        step: "Inspect".to_string(),
                        status: PlanStepStatus::Pending,
                    },
                    PlanItem {
                        step: "Patch".to_string(),
                        status: PlanStepStatus::Pending,
                    },
                ],
                runtime: None,
                updated_at: None,
            },
        )
        .expect("save plan");

        let config = test_config();
        let exec_policy = ExecPolicyConfig::default();
        let service = ToolService::new(
            &conn,
            &config,
            &exec_policy,
            None,
            Some("sync-plan-session"),
        );

        service
            .sync_plan_before_tool("read_file")
            .expect("sync plan before tool");

        let plan = crate::memory::storage::load_plan_state(&conn, "sync-plan-session")
            .expect("load plan")
            .expect("plan present");
        assert_eq!(plan.items[0].status, PlanStepStatus::InProgress);
        assert_eq!(plan.items[1].status, PlanStepStatus::Pending);
    }

    #[test]
    fn sync_plan_after_tool_completes_matching_inspection_step() {
        let conn = Connection::open_in_memory().expect("in-memory db");
        crate::memory::storage::init_db(&conn).expect("init db");
        crate::memory::storage::save_plan_state(
            &conn,
            "after-tool-session",
            &PlanState {
                explanation: Some("Testing auto advance".to_string()),
                items: vec![
                    PlanItem {
                        step: "Inspect server file".to_string(),
                        status: PlanStepStatus::InProgress,
                    },
                    PlanItem {
                        step: "Patch handler".to_string(),
                        status: PlanStepStatus::Pending,
                    },
                ],
                runtime: None,
                updated_at: None,
            },
        )
        .expect("save plan");

        let config = test_config();
        let exec_policy = ExecPolicyConfig::default();
        let service = ToolService::new(
            &conn,
            &config,
            &exec_policy,
            None,
            Some("after-tool-session"),
        );

        service
            .sync_plan_after_tool(r#"{"tool":"read_file","args":{"path":"src/server.rs"}}"#)
            .expect("sync plan after tool");

        let plan = crate::memory::storage::load_plan_state(&conn, "after-tool-session")
            .expect("load plan")
            .expect("plan present");
        assert_eq!(plan.items[0].status, PlanStepStatus::Completed);
        assert_eq!(plan.items[1].status, PlanStepStatus::InProgress);
    }

    #[test]
    fn sync_plan_after_tool_does_not_complete_write_step() {
        let conn = Connection::open_in_memory().expect("in-memory db");
        crate::memory::storage::init_db(&conn).expect("init db");
        crate::memory::storage::save_plan_state(
            &conn,
            "no-auto-complete-session",
            &PlanState {
                explanation: None,
                items: vec![PlanItem {
                    step: "Patch handler".to_string(),
                    status: PlanStepStatus::InProgress,
                }],
                runtime: None,
                updated_at: None,
            },
        )
        .expect("save plan");

        let config = test_config();
        let exec_policy = ExecPolicyConfig::default();
        let service = ToolService::new(
            &conn,
            &config,
            &exec_policy,
            None,
            Some("no-auto-complete-session"),
        );

        service
            .sync_plan_after_tool(r#"{"tool":"read_file","args":{"path":"src/server.rs"}}"#)
            .expect("sync plan after tool");

        let plan = crate::memory::storage::load_plan_state(&conn, "no-auto-complete-session")
            .expect("load plan")
            .expect("plan present");
        assert_eq!(plan.items[0].status, PlanStepStatus::InProgress);
    }

    #[test]
    fn extracts_target_paths_from_json_tool_call() {
        let paths = ToolService::target_paths_for_tool_call(
            r#"{"tool":"write_file","args":{"path":"src/main.rs","content":"hi"}}"#,
        );
        assert_eq!(paths, vec![PathBuf::from("src/main.rs")]);
    }

    #[test]
    fn extracts_target_paths_from_bracket_tool_call() {
        let paths = ToolService::target_paths_for_tool_call(r#"[READ_FILE src/main.rs]"#);
        assert_eq!(paths, vec![PathBuf::from("src/main.rs")]);
    }
}
