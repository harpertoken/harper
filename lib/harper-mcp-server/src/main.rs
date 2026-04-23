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

use axum::{routing::post, Router};
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use tracing::info;

const PROTOCOL_VERSION: &str = "2024-11-05";

fn error_response(
    id: Option<Value>,
    code: i32,
    message: &str,
    data: Option<Value>,
) -> JsonRpcResponse {
    JsonRpcResponse {
        jsonrpc: "2.0".to_string(),
        id,
        result: None,
        error: Some(JsonRpcError {
            code,
            message: message.to_string(),
            data,
        }),
    }
}

#[derive(Serialize, Deserialize)]
struct JsonRpcRequest {
    jsonrpc: String,
    id: Value,
    method: String,
    params: Option<Value>,
}

#[derive(Serialize, Deserialize)]
struct JsonRpcResponse {
    jsonrpc: String,
    id: Option<Value>,
    result: Option<Value>,
    error: Option<JsonRpcError>,
}

#[derive(Serialize, Deserialize)]
struct JsonRpcError {
    code: i32,
    message: String,
    data: Option<Value>,
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
struct InitializeParams {
    protocol_version: String,
}

#[derive(Deserialize)]
struct ToolCallParams {
    name: String,
}

fn parse_params<T: for<'de> serde::de::Deserialize<'de>>(params: Option<&Value>) -> Option<T> {
    let args = params?.get("arguments")?;
    serde_json::from_value(args.clone()).ok()
}

fn handle_initialize(request_id: Value, params: Option<&Value>) -> JsonRpcResponse {
    let init_params: InitializeParams =
        match params.and_then(|p| serde_json::from_value(p.clone()).ok()) {
            Some(p) => p,
            None => return error_response(Some(request_id), -32600, "Invalid Request", None),
        };

    if init_params.protocol_version != PROTOCOL_VERSION {
        return error_response(
            Some(request_id),
            -32602,
            "Unsupported protocol version",
            None,
        );
    }

    JsonRpcResponse {
        jsonrpc: "2.0".to_string(),
        id: Some(request_id),
        result: Some(json!({
            "protocolVersion": PROTOCOL_VERSION,
            "capabilities": {
                "tools": {
                    "listChanged": true
                }
            },
            "serverInfo": {
                "name": "harper-mcp-server",
                "version": env!("CARGO_PKG_VERSION")
            }
        })),
        error: None,
    }
}

fn handle_tools_list(request_id: Value) -> JsonRpcResponse {
    let tools = json!({
        "tools": [
            {
                "name": "echo",
                "description": "Echo the input message",
                "inputSchema": {
                    "type": "object",
                    "properties": {
                        "message": {
                            "type": "string",
                            "description": "The message to echo"
                        }
                    },
                    "required": ["message"]
                }
            },
            {
                "name": "get_time",
                "description": "Get the current UTC time",
                "inputSchema": {
                    "type": "object",
                    "properties": {},
                    "required": []
                }
            }
        ]
    });
    JsonRpcResponse {
        jsonrpc: "2.0".to_string(),
        id: Some(request_id),
        result: Some(tools),
        error: None,
    }
}

fn handle_tools_call(request_id: Value, params: Option<&Value>) -> JsonRpcResponse {
    let tool_params: ToolCallParams =
        match params.and_then(|p| serde_json::from_value(p.clone()).ok()) {
            Some(p) => p,
            None => return error_response(Some(request_id), -32600, "Invalid Request", None),
        };

    match tool_params.name.as_str() {
        "echo" => {
            let msg = parse_params::<EchoArgs>(params)
                .and_then(|args| args.message)
                .unwrap_or_default();
            let result = json!({
                "content": [{
                    "type": "text",
                    "text": msg
                }]
            });
            JsonRpcResponse {
                jsonrpc: "2.0".to_string(),
                id: Some(request_id),
                result: Some(result),
                error: None,
            }
        }
        "get_time" => {
            let now = chrono::Utc::now().to_rfc3339();
            let result = json!({
                "content": [{
                    "type": "text",
                    "text": now
                }]
            });
            JsonRpcResponse {
                jsonrpc: "2.0".to_string(),
                id: Some(request_id),
                result: Some(result),
                error: None,
            }
        }
        _ => error_response(Some(request_id), -32601, "Method not found", None),
    }
}

#[derive(Deserialize)]
struct EchoArgs {
    #[serde(default)]
    message: Option<String>,
}

async fn handle_request(
    axum::Json(rpc_req): axum::Json<JsonRpcRequest>,
) -> axum::Json<JsonRpcResponse> {
    info!(
        "MCP request: method={}, id={:?}",
        rpc_req.method, rpc_req.id
    );
    let response = match rpc_req.method.as_str() {
        "initialize" => handle_initialize(rpc_req.id.clone(), rpc_req.params.as_ref()),
        "tools/list" => handle_tools_list(rpc_req.id.clone()),
        "tools/call" => handle_tools_call(rpc_req.id.clone(), rpc_req.params.as_ref()),
        _ => error_response(Some(rpc_req.id), -32601, "Method not found", None),
    };

    axum::Json(response)
}

async fn health() -> &'static str {
    "OK"
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let app = Router::new()
        .route("/", post(handle_request))
        .route("/health", axum::routing::get(health));
    let addr = std::net::SocketAddr::from(([127, 0, 0, 1], 5001));
    println!("MCP server listening on http://127.0.0.1:5001");
    let listener = tokio::net::TcpListener::bind(addr).await?;
    axum::serve(listener, app).await?;
    Ok(())
}
