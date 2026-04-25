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

//! Prompt building and management

use crate::core::error::HarperError;
use crate::core::ApiConfig;
use chrono::Datelike;
use turul_mcp_client::McpClient;

/// Prompt building functionality
pub struct PromptBuilder<'a> {
    pub config: &'a ApiConfig,
    pub prompt_id: Option<String>,
    pub mcp_client: Option<&'a McpClient>,
}

impl<'a> PromptBuilder<'a> {
    pub fn new(
        config: &'a ApiConfig,
        prompt_id: Option<String>,
        mcp_client: Option<&'a McpClient>,
    ) -> Self {
        Self {
            config,
            prompt_id,
            mcp_client,
        }
    }

    /// Build system prompt
    pub async fn build_system_prompt(&self, web_search_enabled: bool) -> String {
        // Load custom prompt if specified
        if let Some(ref id) = self.prompt_id {
            if id != "default" {
                if let Ok(custom_prompt) = self.load_custom_prompt(id) {
                    return custom_prompt;
                }
            }
        }

        let mut prompt = format!(
            "You are Harper, a system-integrated assistant.
Operating on model: {}.
Capabilities: File I/O, shell execution, and persistent memory{}.",
            self.config.model_name,
            if web_search_enabled {
                ", plus web search"
            } else {
                ""
            }
        );

        if let Ok(context) = self.get_project_context().await {
            prompt.push_str(&format!("\n\nContext:\n{}", context));
        }

        prompt.push_str(
            "\n\nInterface via JSON tool commands. Analysis should be concise and direct.

Core Tools:
- read_file(path)
- write_file(path, content)
- search_replace(path, old, new)
- run_command(command)
- todo(action, [desc], [index])
- list_changed_files([ext], [tracked], [since])
- firmware_list(), firmware_info(device), firmware_gpio(pin, state)

Example: {\"tool\": \"read_file\", \"path\": \"src/main.rs\"}",
        );

        if let Some(mcp_tools) = self.get_mcp_tools_text().await {
            prompt.push_str(&mcp_tools);
        }

        if let Ok(guidelines) = std::fs::read_to_string("docs/AGENTS.md") {
            prompt.push_str(&format!("\n\nGuidelines:\n{}\n", guidelines));
        }

        if web_search_enabled {
            let year = chrono::Local::now().year();
            prompt.push_str(&format!("\nWeb: `[SEARCH: query]`. Current: {}\n", year));
        }

        prompt
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
    async fn get_project_context(&self) -> Result<String, HarperError> {
        let mut context = String::new();

        let current_dir = std::env::current_dir()
            .map_err(|e| HarperError::Command(format!("Failed to get current dir: {}", e)))?;

        let mut entries = tokio::fs::read_dir(&current_dir)
            .await
            .map_err(|e| HarperError::Command(format!("Failed to read dir: {}", e)))?;

        let mut files = Vec::new();
        loop {
            match entries.next_entry().await {
                Ok(Some(entry)) => {
                    if let Ok(file_name) = entry.file_name().into_string() {
                        if !file_name.starts_with('.')
                            && file_name != "target"
                            && file_name != "node_modules"
                        {
                            let metadata = entry.metadata().await.map_err(|e| {
                                HarperError::Command(format!("Failed to get metadata: {}", e))
                            })?;
                            if metadata.is_dir() {
                                files.push(format!("{}/", file_name));
                            } else {
                                files.push(file_name);
                            }
                        }
                    }
                }
                Ok(None) => break,
                Err(e) => return Err(HarperError::Command(format!("Failed to read entry: {}", e))),
            }
        }

        context.push_str(&format!("Dir: {}\n", current_dir.display()));
        context.push_str(&format!("Files: {}\n", files.join(", ")));

        // Git status
        if let Ok(git_status) = std::process::Command::new("git")
            .args(["status", "--porcelain"])
            .output()
        {
            if git_status.status.success() {
                let status = String::from_utf8_lossy(&git_status.stdout);
                if !status.trim().is_empty() {
                    context.push_str(&format!("Git:\n{}", status));
                }
            }
        }

        Ok(context)
    }

    /// Get MCP tools text
    async fn get_mcp_tools_text(&self) -> Option<String> {
        if let Some(client) = self.mcp_client {
            match client.list_tools().await {
                Ok(tools) => {
                    if tools.is_empty() {
                        return None;
                    }

                    let mut tools_text = String::from("\n\nMCP Tools:\n");
                    for tool in &tools {
                        tools_text.push_str(&format!(
                            "- {}: {}\n",
                            tool.name,
                            tool.description.as_deref().unwrap_or("...")
                        ));
                    }
                    tools_text
                        .push_str("\nRespond: {\"mcp_tool\": \"name\", \"arguments\": {...}}");
                    Some(tools_text)
                }
                Err(e) => {
                    eprintln!("Warning: Failed to list MCP tools: {}", e);
                    None
                }
            }
        } else {
            None
        }
    }
}
