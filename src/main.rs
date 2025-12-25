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

// use turul_mcp_client::client::Client; // Temporarily disabled
use rusqlite::Connection;
use std::env;

use colored::Colorize;
use harper::core::ApiConfig;
use harper::error::HarperError;

use std::io::Write;

#[allow(unused_imports)]
use harper::runtime::config::{ExecPolicyConfig, HarperConfig};

fn exit_on_error<T, E: std::fmt::Display>(result: Result<T, E>, message: &str) -> T {
    result.unwrap_or_else(|e| {
        eprintln!("{}: {}", message, e);
        std::process::exit(1);
    })
}

macro_rules! handle_menu_error {
    ($expr:expr, $msg:expr) => {
        if let Err(e) = $expr {
            eprintln!("{}: {}", $msg, e);
        }
    };
}

fn print_version() {
    println!("harper v{}", harper::core::constants::VERSION);
    std::process::exit(0);
}

fn get_api_key(config: &HarperConfig) -> String {
    let mut api_key = config.api.api_key.clone();
    if config.api.provider == "Gemini" {
        if let Ok(env_key) = std::env::var("GEMINI_API_KEY") {
            api_key = env_key;
        }
    } else if config.api.provider == "OpenAI" {
        if let Ok(env_key) = std::env::var("OPENAI_API_KEY") {
            api_key = env_key;
        }
    } else if config.api.provider == "Sambanova" {
        if let Ok(env_key) = std::env::var("SAMBASTUDIO_API_KEY") {
            api_key = env_key;
        }
    }
    api_key
}

#[tokio::main]
async fn main() -> Result<(), HarperError> {
    // Load .env file if it exists
    let _ = dotenvy::dotenv();

    // Handle --version flag
    let args: Vec<String> = env::args().collect();
    if args.len() > 1 && (args[1] == "--version" || args[1] == "-v") {
        print_version();
    }
    let config = exit_on_error(HarperConfig::new(), "Failed to load configuration");

    let api_key = get_api_key(&config);

    let api_config = harper::core::ApiConfig {
        provider: config.api.get_provider().map_err(|e| {
            eprintln!("Configuration error: {}", e);
            e
        })?,
        api_key,
        base_url: config.api.base_url.clone(),
        model_name: config.api.model_name.clone(),
    };

    // Display selected model information (only for non-TUI)
    if !std::io::IsTerminal::is_terminal(&std::io::stdout()) {
        println!(
            "🤖 Using {} - {}",
            api_config.provider, api_config.model_name
        );
        println!("📍 API: {}", api_config.base_url);
        println!("💾 Database: {}", config.database.path);
    }

    // Ensure database directory exists
    if let Some(parent) = std::path::Path::new(&config.database.path).parent() {
        exit_on_error(
            std::fs::create_dir_all(parent),
            "Failed to create database directory",
        );
    }

    let conn = exit_on_error(
        Connection::open(&config.database.path),
        "Failed to open database",
    );
    exit_on_error(
        harper::memory::storage::init_db(&conn),
        "Failed to initialize database",
    );

    let _prompt_id = config.prompts.system_prompt_id.clone();
    let mut custom_commands = config.custom_commands.commands.clone().unwrap_or_default();
    // Add default custom commands if none configured
    if custom_commands.is_empty() {
        custom_commands.insert(
            "hello".to_string(),
            "Please greet me warmly and introduce yourself as Harper AI assistant".to_string(),
        );
        custom_commands.insert("status".to_string(), "Please provide a summary of the current system status, available features, and any recent updates".to_string());
    }
    let exec_policy = config.exec_policy.clone();

    // MCP client initialization
    // Note: MCP functionality is currently disabled due to dependency conflicts
    // with reqwest versions (mcp-client uses v0.11, harper uses v0.12).
    // This was done to resolve CodeQL duplicate dependency warnings and improve
    // security analysis accuracy. MCP can be re-enabled with a compatible client
    // version in the future.
    // MCP client temporarily disabled due to dependency conflicts
    let _mcp_client: Option<()> = None;

    #[allow(dead_code, unused_variables)]
    async fn text_menu(
        conn: &Connection,
        api_config: &ApiConfig,
        prompt_id: Option<String>,
        custom_commands: std::collections::HashMap<String, String>,
        exec_policy: ExecPolicyConfig,
    ) {
        loop {
            use harper::core::constants::messages;

            println!(
                "
{}",
                messages::MAIN_MENU_TITLE.bold().yellow()
            );
            println!("1. Start new chat session");
            println!("2. List previous sessions");
            println!("3. View a session's history");
            println!("4. Export a session's history");
            println!("5. Quit");
            print!("{}", messages::ENTER_CHOICE);
            exit_on_error(std::io::stdout().flush(), "Failed to flush stdout");

            let mut menu_choice = String::new();
            exit_on_error(
                std::io::stdin().read_line(&mut menu_choice),
                "Failed to read input",
            );

            let session_service = harper::memory::session_service::SessionService::new(conn);
            let mut api_cache = harper::core::cache::new_api_cache();

            match menu_choice.trim() {
                harper::core::constants::menu::START_CHAT => {
                    println!("Enable web search for this session? (y/n): ");
                    let mut choice = String::new();
                    let _ = std::io::stdin().read_line(&mut choice);
                    let web_search = choice.trim().eq_ignore_ascii_case("y");
                    let mut chat_service = harper::agent::chat::ChatService::new(
                        conn,
                        api_config,
                        // mcp_client.as_ref(), // Temporarily disabled
                        Some(&mut api_cache),
                        prompt_id.clone(),
                        custom_commands.clone(),
                        exec_policy.clone(),
                    );
                    handle_menu_error!(
                        chat_service.start_session(web_search).await,
                        "Error in chat session"
                    );
                }
                harper::core::constants::menu::LIST_SESSIONS => {
                    handle_menu_error!(session_service.list_sessions(), "Error listing sessions");
                }
                harper::core::constants::menu::VIEW_SESSION => {
                    handle_menu_error!(session_service.view_session(), "Error viewing session");
                }
                harper::core::constants::menu::EXPORT_SESSION => {
                    handle_menu_error!(session_service.export_session(), "Error exporting session");
                }
                harper::core::constants::menu::QUIT => {
                    println!("{}", messages::GOODBYE.bold().yellow());
                    break;
                }
                _ => println!("{}", "Invalid choice. Please try again.".red()),
            }
        }
    }

    let session_service = harper::memory::session_service::SessionService::new(&conn);

    // Create theme
    let theme = config
        .ui
        .theme
        .as_ref()
        .map(|t| harper::interfaces::ui::Theme::from_name(t))
        .unwrap_or_default();

    // Try TUI first, fall back to text menu if TUI fails
    let custom_commands = config.custom_commands.commands.clone().unwrap_or_default();
    if let Err(e) = harper::interfaces::ui::run_tui(
        &conn,
        &api_config,
        &session_service,
        &theme,
        custom_commands.clone(),
        &exec_policy,
    )
    .await
    {
        eprintln!("TUI not available ({}), falling back to text menu...", e);

        // Fall back to text menu
        text_menu(
            &conn,
            &api_config,
            _prompt_id,
            custom_commands,
            exec_policy.clone(),
        )
        .await;
    }

    Ok(())
}
