// use mcp_client::{transport::SseTransport, McpClient, McpClientTrait, McpService, Transport}; // Temporarily disabled
use rusqlite::Connection;
use std::env;

mod config;
mod core;
mod providers;
mod storage;
mod ui;
mod utils;

use crate::core::ApiConfig;
use colored::Colorize;
use std::io::Write;
use ui::tui::run_tui;

use config::HarperConfig;

use providers::*;
use storage::*;

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
    println!("harper v{}", crate::core::constants::VERSION);
    std::process::exit(0);
}

#[tokio::main]
async fn main() {
    // Handle --version flag
    let args: Vec<String> = env::args().collect();
    if args.len() > 1 && (args[1] == "--version" || args[1] == "-v") {
        print_version();
    }
    let config = exit_on_error(HarperConfig::new(), "Failed to load configuration");

    let api_config = crate::core::ApiConfig {
        provider: config
            .api
            .get_provider()
            .map_err(|e| {
                eprintln!("Configuration error: {}", e);
                e
            })
            .unwrap(),
        api_key: config.api.api_key.clone(),
        base_url: config.api.base_url.clone(),
        model_name: config.api.model_name.clone(),
    };

    let conn = exit_on_error(
        Connection::open(&config.database.path),
        "Failed to open database",
    );
    exit_on_error(init_db(&conn), "Failed to initialize database");

    // MCP client initialization
    // Note: MCP functionality is currently disabled due to dependency conflicts
    // with reqwest versions (mcp-client uses v0.11, harper uses v0.12).
    // This was done to resolve CodeQL duplicate dependency warnings and improve
    // security analysis accuracy. MCP can be re-enabled with a compatible client
    // version in the future.
    // MCP client temporarily disabled due to dependency conflicts
    // This resolves CodeQL duplicate dependency warnings and improves security analysis
    let _mcp_client: Option<()> = None;

    async fn text_menu(conn: &Connection, api_config: &ApiConfig) {
        loop {
            use crate::core::constants::messages;

            println!("\n{}", messages::MAIN_MENU_TITLE.bold().yellow());
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

            let session_service = crate::core::session_service::SessionService::new(conn);
            let mut api_cache = crate::core::cache::new_api_cache();

            match menu_choice.trim() {
                crate::core::constants::menu::START_CHAT => {
                    let mut chat_service = crate::core::chat_service::ChatService::new(
                        conn,
                        api_config,
                        // mcp_client.as_ref(), // Temporarily disabled
                        Some(&mut api_cache),
                    );
                    handle_menu_error!(chat_service.start_session().await, "Error in chat session");
                }
                crate::core::constants::menu::LIST_SESSIONS => {
                    handle_menu_error!(session_service.list_sessions(), "Error listing sessions");
                }
                crate::core::constants::menu::VIEW_SESSION => {
                    handle_menu_error!(session_service.view_session(), "Error viewing session");
                }
                crate::core::constants::menu::EXPORT_SESSION => {
                    handle_menu_error!(session_service.export_session(), "Error exporting session");
                }
                crate::core::constants::menu::QUIT => {
                    println!("{}", messages::GOODBYE.bold().yellow());
                    break;
                }
                _ => println!("{}", "Invalid choice. Please try again.".red()),
            }
        }
    }

    // Run TUI or text menu based on terminal
    if std::io::IsTerminal::is_terminal(&std::io::stdin()) {
        if let Err(e) = run_tui(&conn, &api_config).await {
            eprintln!("TUI error: {}", e);
        } else {
            use crate::core::constants::messages;
            println!("{}", messages::GOODBYE.bold().yellow());
        }
    } else {
        // Fallback to text menu for non-interactive environments
        text_menu(&conn, &api_config).await;
    }
}
