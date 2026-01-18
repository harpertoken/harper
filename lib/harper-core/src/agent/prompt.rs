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
            "You are a helpful AI assistant powered by the {} model.
You have the ability to read and write files, search and replace text in files, and run shell commands{}.",
            self.config.model_name,
            if web_search_enabled { " and search the web" } else { "" }
        );

        // Add project context
        if let Ok(context) = self.get_project_context().await {
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

        // Add MCP tools if available
        if let Some(mcp_tools) = self.get_mcp_tools_text().await {
            prompt.push_str(&mcp_tools);
        }

        // Load and append agent guidelines
        match std::fs::read_to_string("docs/AGENTS.md") {
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

        context.push_str(&format!("Current directory: {}\n", current_dir.display()));
        context.push_str(&format!("Files in project root: {}\n", files.join(", ")));

        // Git status
        if let Ok(git_status) = std::process::Command::new("git")
            .args(["status", "--porcelain"])
            .output()
        {
            if git_status.status.success() {
                let status = String::from_utf8_lossy(&git_status.stdout);
                if !status.trim().is_empty() {
                    context.push_str(&format!("Git status:\n{}", status));
                } else {
                    context.push_str("Git status: clean\n");
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

                    let mut tools_text = String::from("\n\nMCP Tools (Model Context Protocol):\n");
                    for tool in &tools {
                        tools_text.push_str(&format!(
                            "- {}: {}\n",
                            tool.name,
                            tool.description.as_deref().unwrap_or("No description")
                        ));
                    }
                    tools_text.push_str("\nTo use an MCP tool, respond with: {\"mcp_tool\": \"tool_name\", \"arguments\": {...}}");
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
