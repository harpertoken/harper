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
pub mod shell;
pub mod todo;
pub mod web;

/// Common parsing utilities for tool arguments
pub mod parsing;

use crate::core::constants::tools;
use crate::core::error::HarperError;
use crate::core::{ApiConfig, Message};
use crate::runtime::config::ExecPolicyConfig;
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
    exec_policy: &'a ExecPolicyConfig,
}

impl<'a> ToolService<'a> {
    /// Create a new tool service
    pub fn new(config: &'a ApiConfig, exec_policy: &'a ExecPolicyConfig) -> Self {
        Self {
            config,
            exec_policy,
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
        // JSON parsing commented out for debugging

        // Fallback to old bracket format
        if response.to_uppercase().starts_with(tools::RUN_COMMAND) {
            let command_result = shell::execute_command(response, self.config, self.exec_policy)?;
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
        } else if response.to_uppercase().starts_with(tools::GITHUB_ISSUE) {
            let issue_result = github::create_issue(response)?;
            let final_response = self
                .call_llm_after_tool(client, history, &issue_result)
                .await?;
            Ok(Some((final_response, issue_result)))
        } else if response.to_uppercase().starts_with(tools::GITHUB_PR) {
            let pr_result = github::create_pr(response)?;
            let final_response = self
                .call_llm_after_tool(client, history, &pr_result)
                .await?;
            Ok(Some((final_response, pr_result)))
        } else if response.to_uppercase().starts_with(tools::API_TEST) {
            let api_result = api::test_api(response).await?;
            let final_response = self
                .call_llm_after_tool(client, history, &api_result)
                .await?;
            Ok(Some((final_response, api_result)))
        } else if response.to_uppercase().starts_with(tools::CODE_ANALYZE) {
            let analysis_result = code_analysis::analyze_code(response)?;
            let final_response = self
                .call_llm_after_tool(client, history, &analysis_result)
                .await?;
            Ok(Some((final_response, analysis_result)))
        } else if response.to_uppercase().starts_with(tools::DB_QUERY) {
            let db_result = db::run_query(response)?;
            let final_response = self
                .call_llm_after_tool(client, history, &db_result)
                .await?;
            Ok(Some((final_response, db_result)))
        } else if response.to_uppercase().starts_with(tools::IMAGE_INFO) {
            let info_result = image::get_image_info(response)?;
            let final_response = self
                .call_llm_after_tool(client, history, &info_result)
                .await?;
            Ok(Some((final_response, info_result)))
        } else if response.to_uppercase().starts_with(tools::IMAGE_RESIZE) {
            let resize_result = image::resize_image(response)?;
            let final_response = self
                .call_llm_after_tool(client, history, &resize_result)
                .await?;
            Ok(Some((final_response, resize_result)))
        } else {
            Ok(None)
        }
    }
    #[allow(dead_code)]
    async fn handle_json_tool(
        &mut self,
        client: &Client,
        history: &[Message],
        tool_name: &str,
        json_value: &serde_json::Value,
    ) -> Result<Option<(String, String)>, HarperError> {
        // Get parameters from "args" (Gemini format) or directly from json_value
        let args = json_value.get("args");

        match tool_name {
            "run_command" => {
                let command = if let Some(args) = args {
                    args.get("command")
                } else {
                    json_value.get("command")
                }
                .and_then(|v| v.as_str());
                if let Some(command) = command {
                    let bracket_command = format!("[RUN_COMMAND {}]", command);
                    let command_result =
                        shell::execute_command(&bracket_command, self.config, self.exec_policy)?;
                    let final_response = self
                        .call_llm_after_tool(client, history, &command_result)
                        .await?;
                    Ok(Some((final_response, command_result)))
                } else {
                    Ok(None)
                }
            }

            "write_file" => {
                let path = if let Some(args) = args {
                    args.get("path")
                } else {
                    json_value.get("path")
                }
                .and_then(|v| v.as_str());
                let content = if let Some(args) = args {
                    args.get("content")
                } else {
                    json_value.get("content")
                }
                .and_then(|v| v.as_str());
                if let (Some(path), Some(content)) = (path, content) {
                    let bracket_command = format!("[WRITE_FILE {} {}]", path, content);
                    let write_result = filesystem::write_file(&bracket_command)?;
                    let final_response = self
                        .call_llm_after_tool(client, history, &write_result)
                        .await?;
                    Ok(Some((final_response, write_result)))
                } else {
                    Ok(None)
                }
            }
            "search_replace" => {
                let path = if let Some(args) = args {
                    args.get("path")
                } else {
                    json_value.get("path")
                }
                .and_then(|v| v.as_str());
                let old_string = if let Some(args) = args {
                    args.get("old_string")
                } else {
                    json_value.get("old_string")
                }
                .and_then(|v| v.as_str());
                let new_string = if let Some(args) = args {
                    args.get("new_string")
                } else {
                    json_value.get("new_string")
                }
                .and_then(|v| v.as_str());
                if let (Some(path), Some(old_string), Some(new_string)) =
                    (path, old_string, new_string)
                {
                    let bracket_command =
                        format!("[SEARCH_REPLACE {} {} {}]", path, old_string, new_string);
                    let replace_result = filesystem::search_replace(&bracket_command)?;
                    let final_response = self
                        .call_llm_after_tool(client, history, &replace_result)
                        .await?;
                    Ok(Some((final_response, replace_result)))
                } else {
                    Ok(None)
                }
            }
            "todo" => {
                let action = if let Some(args) = args {
                    args.get("action")
                } else {
                    json_value.get("action")
                }
                .and_then(|v| v.as_str());
                if let Some(action) = action {
                    let bracket_command = match action {
                        "add" => {
                            let description = if let Some(args) = args {
                                args.get("description")
                            } else {
                                json_value.get("description")
                            }
                            .and_then(|v| v.as_str());
                            if let Some(description) = description {
                                format!("[TODO add {}]", description)
                            } else {
                                "".to_string()
                            }
                        }
                        "list" => "[TODO list]".to_string(),
                        "remove" => {
                            let index = if let Some(args) = args {
                                args.get("index")
                            } else {
                                json_value.get("index")
                            }
                            .and_then(|v| v.as_i64());
                            if let Some(index) = index {
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
                    let todo_result = todo::manage_todo(&bracket_command)?;
                    let final_response = self
                        .call_llm_after_tool(client, history, &todo_result)
                        .await?;
                    Ok(Some((final_response, todo_result)))
                } else {
                    Ok(None)
                }
            }
            _ => Ok(None),
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
