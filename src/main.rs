use chrono::Datelike;
use colored::*;
use mcp_client::{
    transport::{sse::SseTransportHandle, SseTransport},
    McpClient, McpClientTrait, McpService, Transport,
};
use rusqlite::Connection;
use std::collections::HashMap;
use std::fs::File;
use std::io::{self, Write};
use std::time::Duration;
use tower::timeout::Timeout;
use uuid::Uuid;

mod config;
mod core;
mod providers;
mod storage;
mod ui;
mod utils;

use config::HarperConfig;
use core::*;
use providers::*;
use storage::*;
use utils::*;

fn list_sessions(conn: &Connection) {
    let mut stmt = conn
        .prepare("SELECT id, created_at FROM sessions ORDER BY created_at DESC")
        .unwrap();
    let rows = stmt
        .query_map([], |row| {
            Ok((row.get::<_, String>(0)?, row.get::<_, String>(1)?))
        })
        .unwrap();
    println!("{}", "Previous Sessions:".bold().yellow());
    for (i, row) in rows.enumerate() {
        let (id, created_at) = row.unwrap();
        println!("{}: {} ({})", i + 1, id, created_at);
    }
}

fn view_session(conn: &Connection) {
    print!("Enter session ID to view: ");
    io::stdout().flush().unwrap();
    let mut session_id = String::new();
    io::stdin().read_line(&mut session_id).unwrap();
    let session_id = session_id.trim();
    let history = load_history(conn, session_id).unwrap_or_default();
    println!("\n{}\n", "Session History:".bold().yellow());
    for msg in history {
        match msg.role.as_str() {
            "user" => println!("{} {}", "You:".bold().blue(), msg.content.blue()),
            "assistant" => println!("{} {}", "Assistant:".bold().green(), msg.content.green()),
            "system" => println!("{} {}", "System:".bold().magenta(), msg.content.magenta()),
            _ => println!("{}: {}", msg.role, msg.content),
        }
    }
}

fn export_session(conn: &Connection) {
    print!("Enter session ID to export: ");
    io::stdout().flush().unwrap();
    let mut session_id = String::new();
    io::stdin().read_line(&mut session_id).unwrap();
    let session_id = session_id.trim();
    let history = load_history(conn, session_id).unwrap_or_default();
    let filename = format!("session_{}.txt", session_id);
    let mut file = File::create(&filename).unwrap();
    for msg in &history {
        let line = format!("{}: {}\n", msg.role, msg.content);
        file.write_all(line.as_bytes()).unwrap();
    }
    println!("Session exported to {}", filename.bold().yellow());
}

async fn start_chat_session(
    conn: &Connection,
    config: &ApiConfig,
    mcp_client: Option<&McpClient<Timeout<McpService<SseTransportHandle>>>>,
) {
    let session_id = Uuid::new_v4().to_string();
    save_session(conn, &session_id).unwrap();

    print!("Enable web search for this session? (y/n): ");
    io::stdout().flush().unwrap();
    let mut web_search_choice = String::new();
    io::stdin().read_line(&mut web_search_choice).unwrap();
    let web_search_enabled = web_search_choice.trim().eq_ignore_ascii_case("y");

    println!(
        "{}\n",
        "New chat session started. Type 'exit' to quit."
            .bold()
            .yellow()
    );

    let system_prompt = if web_search_enabled {
        let current_year = chrono::Local::now().year();
        format!(
            "You are a helpful AI assistant powered by the {} model.
You have the ability to run any Linux shell command.
Your response MUST be ONLY the tool command. Do not add any explanation.
Do NOT use interactive commands (like 'nano', 'vim'). Use non-interactive commands like `cat` to read files.

Tool format:
- Run a shell command: `[RUN_COMMAND <command to run>]`
- Search the web: `[SEARCH: your query]`. Current year: {}",
            config.model_name, current_year
        )
    } else {
        format!(
            "You are an AI assistant powered by the {} model.",
            config.model_name
        )
    };

    let mut history = vec![Message {
        role: "system".to_string(),
        content: system_prompt,
    }];

    loop {
        print!("{} ", "You:".bold().blue());
        io::stdout().flush().unwrap();
        let mut user_input = String::new();
        io::stdin().read_line(&mut user_input).unwrap();
        let user_input = user_input.trim();

        if user_input.is_empty() {
            continue;
        }

        if user_input.eq_ignore_ascii_case("exit") || user_input.eq_ignore_ascii_case("quit") {
            println!("{}", "Session ended.".bold().yellow());
            break;
        }

        history.push(Message {
            role: "user".to_string(),
            content: user_input.to_string(),
        });
        save_message(conn, &session_id, "user", user_input).unwrap();

        let client = reqwest::Client::builder()
            .timeout(Duration::from_secs(90))
            .build()
            .unwrap();

        match call_llm(&client, config, &history, mcp_client).await {
            Ok(mut assistant_reply) => {
                let trimmed_reply = assistant_reply
                    .trim()
                    .trim_matches(|c| c == '\'' || c == '\"' || c == '`');

                let mut tool_used = false;

                if trimmed_reply.to_uppercase().starts_with("[RUN_COMMAND") {
                    tool_used = true;
                    let command_str = if let Some(pos) = trimmed_reply.find(' ') {
                        trimmed_reply[pos..].trim_start().trim_end_matches(']')
                    } else {
                        ""
                    };

                    if command_str.is_empty() {
                        println!(
                            "{} {}",
                            "System:".bold().magenta(),
                            "No command provided for [RUN_COMMAND].".red()
                        );
                        continue;
                    }

                    println!(
                        "{} Running command: {}",
                        "System:".bold().magenta(),
                        command_str.magenta()
                    );

                    let output = std::process::Command::new("sh")
                        .arg("-c")
                        .arg(command_str)
                        .output()
                        .expect("failed to execute process");

                    let result = if output.status.success() {
                        String::from_utf8_lossy(&output.stdout).to_string()
                    } else {
                        String::from_utf8_lossy(&output.stderr).to_string()
                    };

                    println!("{}\n{}", "Assistant:".bold().green(), result.green());
                    history.push(Message {
                        role: "assistant".to_string(),
                        content: assistant_reply.clone(),
                    });
                    history.push(Message {
                        role: "system".to_string(),
                        content: format!("Command output:\n{}", result),
                    });
                } else if web_search_enabled && trimmed_reply.to_uppercase().starts_with("[SEARCH:")
                {
                    tool_used = true;
                    let query_part = trimmed_reply
                        .split_once(':')
                        .map(|x| x.1)
                        .unwrap_or("")
                        .trim_end_matches(']');
                    println!(
                        "{} Searching the web for: {}",
                        "System:".bold().magenta(),
                        query_part.magenta()
                    );

                    let search_results = web_search(query_part)
                        .await
                        .unwrap_or_else(|e| format!("Failed to perform web search: {}", e));
                    let tool_result_prompt = format!(
                        "Web search results for '{}':\n{}",
                        query_part, search_results
                    );
                    history.push(Message {
                        role: "assistant".to_string(),
                        content: assistant_reply.clone(),
                    });
                    history.push(Message {
                        role: "system".to_string(),
                        content: tool_result_prompt,
                    });
                }

                if tool_used {
                    match call_llm(&client, config, &history, mcp_client).await {
                        Ok(final_reply) => {
                            assistant_reply = final_reply;
                        }
                        Err(e) => {
                            println!(
                                "Assistant: {} ({})",
                                "API Error after tool use".red(),
                                e.to_string().red()
                            );
                            continue;
                        }
                    }
                }

                println!(
                    "{} {}\n",
                    "Assistant:".bold().green(),
                    assistant_reply.green()
                );
                history.push(Message {
                    role: "assistant".to_string(),
                    content: assistant_reply.clone(),
                });
                save_message(conn, &session_id, "assistant", &assistant_reply).unwrap();
            }
            Err(e) => {
                println!("Assistant: {} ({})", "API Error".red(), e.to_string().red());
                continue;
            }
        }
    }
}

#[tokio::main]
async fn main() {
    let config = match HarperConfig::new() {
        Ok(c) => c,
        Err(e) => {
            eprintln!("Failed to load configuration: {}", e);
            return;
        }
    };

    let api_config = ApiConfig {
        provider: match config.api.provider.as_str() {
            "OpenAI" => ApiProvider::OpenAI,
            "Sambanova" => ApiProvider::Sambanova,
            "Gemini" => ApiProvider::Gemini,
            _ => {
                println!(
                    "{}",
                    "Invalid API provider in configuration. Exiting.".red()
                );
                return;
            }
        },
        api_key: config.api.api_key.clone(),
        base_url: config.api.base_url.clone(),
        model_name: config.api.model_name.clone(),
    };

    let conn = Connection::open(&config.database.path).unwrap();
    init_db(&conn).unwrap();

    let mcp_client = if config.mcp.enabled {
        // Create SSE transport
        let transport = SseTransport::new(config.mcp.server_url.clone(), HashMap::new());

        // Start transport and get handle
        let handle = match transport.start().await {
            Ok(handle) => handle,
            Err(e) => {
                eprintln!("Failed to start MCP transport: {}", e);
                return;
            }
        };

        // Create service with timeout
        let service = McpService::with_timeout(handle, Duration::from_secs(30));

        // Create client
        let mut client = McpClient::new(service);

        // Initialize client
        match client
            .initialize(
                mcp_client::client::ClientInfo {
                    name: "harper".into(),
                    version: "0.1.0".into(),
                },
                mcp_client::client::ClientCapabilities::default(),
            )
            .await
        {
            Ok(_) => Some(client),
            Err(e) => {
                eprintln!("Failed to initialize MCP client: {}", e);
                None
            }
        }
    } else {
        None
    };

    loop {
        println!("\n{}", "Main Menu".bold().yellow());
        println!("1. Start new chat session");
        println!("2. List previous sessions");
        println!("3. View a session's history");
        println!("4. Export a session's history");
        println!("5. Quit");
        print!("Enter your choice: ");
        io::stdout().flush().unwrap();

        let mut menu_choice = String::new();
        io::stdin().read_line(&mut menu_choice).unwrap();

        match menu_choice.trim() {
            "1" => start_chat_session(&conn, &api_config, mcp_client.as_ref()).await,
            "2" => list_sessions(&conn),
            "3" => view_session(&conn),
            "4" => export_session(&conn),
            "5" => {
                println!("{}", "Goodbye!".bold().yellow());
                break;
            }
            _ => println!("{}", "Invalid choice. Please try again.".red()),
        }
    }
}
