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

use async_trait::async_trait;
use harper_core::core::io_traits::{DenyApproval, RuntimeEventSink, StdinApproval};
use harper_core::runtime::config::HarperConfig;
use harper_core::{
    agent::chat::{ChatService, ChatTurnDebugSummary},
    create_connection, init_db, ApiConfig, HarperError, Message, PlanState, ResolvedAgents,
};
use serde::Serialize;
use std::collections::HashMap;
use std::env;
use std::io::{self, IsTerminal};
use std::sync::{Arc, Mutex};
use turul_mcp_client::McpClient;

#[derive(Debug, Default, Clone, Serialize)]
struct TurnDebugOutput {
    activity: Vec<String>,
    command: Option<String>,
    command_output: Option<String>,
    command_error: bool,
    command_done: bool,
}

#[derive(Default)]
struct BatchRuntimeEvents {
    per_session: Mutex<HashMap<String, TurnDebugOutput>>,
}

impl BatchRuntimeEvents {
    fn take_turn(&self, session_id: &str) -> TurnDebugOutput {
        self.per_session
            .lock()
            .expect("batch runtime events lock")
            .remove(session_id)
            .unwrap_or_default()
    }
}

#[async_trait]
impl RuntimeEventSink for BatchRuntimeEvents {
    async fn plan_updated(
        &self,
        _session_id: &str,
        _plan: Option<PlanState>,
    ) -> Result<(), HarperError> {
        Ok(())
    }

    async fn agents_updated(
        &self,
        _session_id: &str,
        _agents: Option<ResolvedAgents>,
    ) -> Result<(), HarperError> {
        Ok(())
    }

    async fn activity_updated(
        &self,
        session_id: &str,
        status: Option<String>,
    ) -> Result<(), HarperError> {
        if let Some(status) = status {
            let mut sessions = self.per_session.lock().expect("batch runtime events lock");
            let entry = sessions.entry(session_id.to_string()).or_default();
            if entry.activity.last() != Some(&status) {
                entry.activity.push(status);
            }
        }
        Ok(())
    }

    async fn command_output_updated(
        &self,
        session_id: &str,
        command: String,
        chunk: String,
        is_error: bool,
        done: bool,
    ) -> Result<(), HarperError> {
        let mut sessions = self.per_session.lock().expect("batch runtime events lock");
        let entry = sessions.entry(session_id.to_string()).or_default();
        entry.command = Some(command);
        entry.command_error = is_error;
        entry.command_done = done;
        match entry.command_output.as_mut() {
            Some(output) => output.push_str(&chunk),
            None => entry.command_output = Some(chunk),
        }
        Ok(())
    }
}

#[derive(Default)]
struct BatchArgs {
    prompts: Vec<String>,
    strategy: Option<String>,
    json: bool,
    web: bool,
    help: bool,
}

#[derive(Serialize)]
struct TurnResult {
    prompt: String,
    response: String,
    routing: ChatTurnDebugSummary,
    debug: TurnDebugOutput,
}

fn print_help() {
    println!("Harper Batch Processor");
    println!();
    println!("Usage:");
    println!("  harper-batch --prompt \"...\" [--prompt \"...\"] [options]");
    println!("  printf 'prompt one\\nprompt two\\n' | harper-batch [options]");
    println!();
    println!("Options:");
    println!("  --prompt <text>          Add a prompt to run in the same session");
    println!("  --strategy <mode>       One of: auto, grounded, deterministic, model");
    println!("  --json                  Print JSON output");
    println!("  --web                   Enable web search for the session");
    println!("  --help                  Show this help");
}

fn parse_args(args: &[String]) -> Result<BatchArgs, HarperError> {
    let mut parsed = BatchArgs::default();
    let mut index = 1;
    while index < args.len() {
        match args[index].as_str() {
            "--prompt" => {
                index += 1;
                let Some(value) = args.get(index) else {
                    return Err(HarperError::Validation(
                        "--prompt requires a value".to_string(),
                    ));
                };
                parsed.prompts.push(value.clone());
            }
            "--strategy" => {
                index += 1;
                let Some(value) = args.get(index) else {
                    return Err(HarperError::Validation(
                        "--strategy requires a value".to_string(),
                    ));
                };
                parsed.strategy = Some(value.clone());
            }
            "--json" => parsed.json = true,
            "--web" => parsed.web = true,
            "--help" | "-h" => parsed.help = true,
            other => {
                return Err(HarperError::Validation(format!(
                    "Unknown batch argument: {}",
                    other
                )))
            }
        }
        index += 1;
    }
    Ok(parsed)
}

fn get_api_key(config: &HarperConfig) -> String {
    let mut api_key = config.api.api_key.clone();
    if config.api.provider == "Gemini" {
        if let Ok(env_key) = env::var("GEMINI_API_KEY") {
            api_key = env_key;
        }
    } else if config.api.provider == "OpenAI" {
        if let Ok(env_key) = env::var("OPENAI_API_KEY") {
            api_key = env_key;
        }
    } else if config.api.provider == "Sambanova" {
        if let Ok(env_key) = env::var("SAMBASTUDIO_API_KEY") {
            api_key = env_key;
        }
    }
    api_key
}

fn read_stdin_prompts() -> Result<Vec<String>, HarperError> {
    if io::stdin().is_terminal() {
        return Ok(Vec::new());
    }
    let input = io::read_to_string(io::stdin())
        .map_err(|e| HarperError::Io(format!("Failed to read stdin prompts: {}", e)))?;
    Ok(input
        .lines()
        .map(str::trim)
        .filter(|line| !line.is_empty())
        .map(ToOwned::to_owned)
        .collect())
}

fn build_api_config(config: &HarperConfig) -> Result<ApiConfig, HarperError> {
    Ok(ApiConfig {
        provider: config.api.get_provider()?,
        api_key: get_api_key(config),
        base_url: config.api.base_url.clone(),
        model_name: config.api.model_name.clone(),
    })
}

async fn init_mcp_client(config: &HarperConfig) -> Option<McpClient> {
    if !config.mcp.enabled {
        return None;
    }

    let transport = turul_mcp_client::transport::HttpTransport::new(&config.mcp.server_url).ok()?;
    let client = turul_mcp_client::McpClientBuilder::new()
        .with_transport(Box::new(transport))
        .build();
    client.connect().await.ok()?;
    Some(client)
}

fn latest_assistant_response(history: &[Message]) -> String {
    history
        .iter()
        .rev()
        .find(|message| message.role == "assistant")
        .map(|message| message.content.clone())
        .unwrap_or_default()
}

#[tokio::main]
async fn main() -> Result<(), HarperError> {
    let _ = dotenvy::dotenv();

    let raw_args: Vec<String> = env::args().collect();
    let mut args = parse_args(&raw_args)?;
    if args.help {
        print_help();
        return Ok(());
    }

    args.prompts.extend(read_stdin_prompts()?);
    if args.prompts.is_empty() {
        return Err(HarperError::Validation(
            "No prompts provided. Use --prompt or pipe prompts on stdin.".to_string(),
        ));
    }

    let config = HarperConfig::new()?;
    let api_config = build_api_config(&config)?;
    let conn = create_connection(&config.database.path)?;
    init_db(&conn)?;
    let mcp_client = init_mcp_client(&config).await;
    let runtime_events = Arc::new(BatchRuntimeEvents::default());

    let mut chat_service = ChatService::new(
        &conn,
        &api_config,
        mcp_client.as_ref(),
        None,
        Some(uuid::Uuid::new_v4().to_string()),
        config.custom_commands.commands.clone().unwrap_or_default(),
        config.exec_policy.clone(),
    )
    .with_runtime_events(runtime_events.clone());

    if io::stdin().is_terminal() {
        chat_service = chat_service.with_approver(Arc::new(StdinApproval));
    } else {
        chat_service = chat_service.with_approver(Arc::new(DenyApproval));
    }

    let (mut history, session_id) = chat_service.create_session(args.web).await?;

    if let Some(strategy) = args.strategy.as_deref() {
        let strategy_command = format!("/strategy {}", strategy);
        chat_service
            .send_message(&strategy_command, &mut history, args.web, &session_id)
            .await?;
        let _ = runtime_events.take_turn(&session_id);
    }

    let mut results = Vec::new();
    for prompt in args.prompts {
        let routing = chat_service.debug_turn_summary(&history, &prompt);
        chat_service
            .send_message(&prompt, &mut history, args.web, &session_id)
            .await?;
        let mut debug = runtime_events.take_turn(&session_id);
        let response = latest_assistant_response(&history);
        if routing.task_mode == "respond_only"
            && result_looks_like_backend_unavailable(&response)
            && routing.deterministic_intent.is_none()
            && routing.normalized_command.is_none()
            && routing.clarification.is_none()
        {
            debug.activity.retain(|status| status == "responding");
        }
        results.push(TurnResult {
            prompt,
            response,
            routing,
            debug,
        });
    }

    if args.json {
        println!(
            "{}",
            serde_json::to_string_pretty(&results)
                .map_err(|e| HarperError::Validation(format!("Failed to serialize JSON: {}", e)))?
        );
        return Ok(());
    }

    for result in results {
        println!("USER: {}", result.prompt);
        println!(
            "ROUTING: strategy={} task_mode={}",
            result.routing.strategy, result.routing.task_mode
        );
        if let Some(intent) = &result.routing.deterministic_intent {
            println!("DETERMINISTIC INTENT: {}", intent);
        }
        if let Some(command) = &result.routing.normalized_command {
            println!("NORMALIZED COMMAND: {}", command);
        }
        if let Some(clarification) = &result.routing.clarification {
            println!("CLARIFICATION: {}", clarification);
        }
        if !result.debug.activity.is_empty() {
            println!("ACTIVITY:");
            for status in &result.debug.activity {
                println!("  - {}", status);
            }
        }
        if let Some(command) = &result.debug.command {
            println!("COMMAND: {}", command);
        }
        if let Some(output) = &result.debug.command_output {
            if result.debug.command_error {
                println!("COMMAND OUTPUT (error):");
            } else {
                println!("COMMAND OUTPUT:");
            }
            println!("{}", output.trim_end());
        }
        println!("ASSISTANT: {}", result.response);
        println!();
    }

    Ok(())
}

fn result_looks_like_backend_unavailable(response: &str) -> bool {
    response
        .trim()
        .starts_with("The model backend is unavailable, and this request does not have a deterministic fallback.")
}
