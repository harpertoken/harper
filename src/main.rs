// use turul_mcp_client::client::Client; // Temporarily disabled
use rusqlite::Connection;
use std::env;

mod agent;
mod core;
mod interfaces;
mod memory;
mod runtime;
mod tools;

use crate::core::ApiConfig;
use colored::Colorize;
use interfaces::ui::tui::run_tui;
use std::io::Write;

use runtime::config::HarperConfig;

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
    // Load .env file if it exists
    let _ = dotenvy::dotenv();

    // Handle --version flag
    let args: Vec<String> = env::args().collect();
    if args.len() > 1 && (args[1] == "--version" || args[1] == "-v") {
        print_version();
    }
    let config = exit_on_error(HarperConfig::new(), "Failed to load configuration");

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

    let api_config = crate::core::ApiConfig {
        provider: config
            .api
            .get_provider()
            .map_err(|e| {
                eprintln!("Configuration error: {}", e);
                e
            })
            .unwrap(),
        api_key,
        base_url: config.api.base_url.clone(),
        model_name: config.api.model_name.clone(),
    };

    // Display selected model information
    println!(
        "🤖 Using {} - {}",
        api_config.provider, api_config.model_name
    );
    println!("📍 API: {}", api_config.base_url);
    println!("💾 Database: {}", config.database.path);

    let conn = exit_on_error(
        Connection::open(&config.database.path),
        "Failed to open database",
    );
    exit_on_error(
        crate::memory::storage::init_db(&conn),
        "Failed to initialize database",
    );

    let prompt_id = config.prompts.system_prompt_id.clone();

    // MCP client initialization
    // Note: MCP functionality is currently disabled due to dependency conflicts
    // with reqwest versions (mcp-client uses v0.11, harper uses v0.12).
    // This was done to resolve CodeQL duplicate dependency warnings and improve
    // security analysis accuracy. MCP can be re-enabled with a compatible client
    // version in the future.
    // MCP client temporarily disabled due to dependency conflicts
    let _mcp_client: Option<()> = None;

    async fn text_menu(conn: &Connection, api_config: &ApiConfig, prompt_id: Option<String>) {
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

            let session_service = crate::memory::session_service::SessionService::new(conn);
            let mut api_cache = crate::core::cache::new_api_cache();

            match menu_choice.trim() {
                crate::core::constants::menu::START_CHAT => {
                    println!("Enable web search for this session? (y/n): ");
                    let mut choice = String::new();
                    let _ = std::io::stdin().read_line(&mut choice);
                    let web_search = choice.trim().eq_ignore_ascii_case("y");
                    let mut chat_service = crate::agent::chat::ChatService::new(
                        conn,
                        api_config,
                        // mcp_client.as_ref(), // Temporarily disabled
                        Some(&mut api_cache),
                        prompt_id.clone(),
                    );
                    handle_menu_error!(
                        chat_service.start_session(web_search).await,
                        "Error in chat session"
                    );
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
        if let Err(e) = run_tui(&conn, &api_config, prompt_id).await {
            eprintln!("TUI error: {}", e);
        } else {
            use crate::core::constants::messages;
            println!("{}", messages::GOODBYE.bold().yellow());
        }
    } else {
        // Fallback to text menu for non-interactive environments
        text_menu(&conn, &api_config, prompt_id).await;
    }
}
