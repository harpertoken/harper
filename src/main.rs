use colored::*;
// use mcp_client::{transport::SseTransport, McpClient, McpClientTrait, McpService, Transport}; // Temporarily disabled
use rusqlite::Connection;
// use std::collections::HashMap; // Temporarily unused
use std::io::{self, Write};

mod config;
mod core;
mod providers;
mod storage;
mod ui;
mod utils;

use config::HarperConfig;

use crate::core::cache::new_api_cache;
use crate::core::chat_service::ChatService;
use crate::core::constants::{menu, timeouts};
use crate::core::session_service::SessionService;
use providers::*;
use storage::*;

#[tokio::main]
async fn main() {
    let config = match HarperConfig::new() {
        Ok(c) => c,
        Err(e) => {
            eprintln!("Failed to load configuration: {}", e);
            return;
        }
    };

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

    let conn = Connection::open(&config.database.path)
        .map_err(|e| {
            eprintln!("Failed to open database: {}", e);
            e
        })
        .unwrap();
    init_db(&conn)
        .map_err(|e| {
            eprintln!("Failed to initialize database: {}", e);
            e
        })
        .unwrap();

    // MCP client initialization
    // Note: MCP functionality is currently disabled due to dependency conflicts
    // with reqwest versions (mcp-client uses v0.11, harper uses v0.12).
    // This was done to resolve CodeQL duplicate dependency warnings and improve
    // security analysis accuracy. MCP can be re-enabled with a compatible client
    // version in the future.
    // MCP client temporarily disabled due to dependency conflicts
    // This resolves CodeQL duplicate dependency warnings and improves security analysis
    let mcp_client: Option<()> = None;

    loop {
        println!("\n{}", "Main Menu".bold().yellow());
        println!("1. Start new chat session");
        println!("2. List previous sessions");
        println!("3. View a session's history");
        println!("4. Export a session's history");
        println!("5. Quit");
        print!("Enter your choice: ");
        io::stdout()
            .flush()
            .map_err(|e| {
                eprintln!("Failed to flush stdout: {}", e);
            })
            .unwrap();

        let mut menu_choice = String::new();
        io::stdin()
            .read_line(&mut menu_choice)
            .map_err(|e| {
                eprintln!("Failed to read input: {}", e);
            })
            .unwrap();

        let session_service = SessionService::new(&conn);
        let mut api_cache = new_api_cache();

        match menu_choice.trim() {
            menu::START_CHAT => {
                let mut chat_service = ChatService::new(
                    &conn,
                    &api_config,
                    // mcp_client.as_ref(), // Temporarily disabled
                    Some(&mut api_cache),
                );
                if let Err(e) = chat_service.start_session().await {
                    eprintln!("Error in chat session: {}", e);
                }
            }
            menu::LIST_SESSIONS => {
                if let Err(e) = session_service.list_sessions() {
                    eprintln!("Error listing sessions: {}", e);
                }
            }
            menu::VIEW_SESSION => {
                if let Err(e) = session_service.view_session() {
                    eprintln!("Error viewing session: {}", e);
                }
            }
            menu::EXPORT_SESSION => {
                if let Err(e) = session_service.export_session() {
                    eprintln!("Error exporting session: {}", e);
                }
            }
            menu::QUIT => {
                println!("{}", "Goodbye!".bold().yellow());
                break;
            }
            _ => println!("{}", "Invalid choice. Please try again.".red()),
        }
    }
}
