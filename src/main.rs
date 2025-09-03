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
use crate::core::constants::menu;
// use crate::core::constants::timeouts; // Temporarily unused
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

    // MCP client temporarily disabled due to dependency conflicts
    // TODO: Re-enable MCP functionality with a compatible client version
    let _mcp_client: Option<()> = None; // if config.mcp.enabled {
    //     // Create SSE transport
    //     let transport = SseTransport::new(config.mcp.server_url.clone(), HashMap::new());
    //
    //     // Start transport and get handle
    //     let handle = match transport.start().await {
    //         Ok(handle) => handle,
    //         Err(e) => {
    //             eprintln!("Failed to start MCP transport: {}", e);
    //             return;
    //         }
    //     };
    //
    //     // Create service with timeout
    //     let service = McpService::with_timeout(handle, timeouts::MCP_SERVICE);
    //
    //     // Create client
    //     let mut client = McpClient::new(service);
    //
    //     // Initialize client
    //     match client
    //         .initialize(
    //             mcp_client::client::ClientInfo {
    //                 name: "harper".into(),
    //                 version: "0.1.0".into(),
    //             },
    //             mcp_client::client::ClientCapabilities::default(),
    //         )
    //         .await
    //     {
    //         Ok(_) => Some(client),
    //         Err(e) => {
    //             eprintln!("Failed to initialize MCP client: {}", e);
    //             None
    //         }
    //     }
    // } else {
    //     None
    // };

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
