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

fn handle_initialize(request_id: Value, params: Option<&Value>) -> JsonRpcResponse {
    // Check protocol version
    if let Some(params) = params {
        if let Some(version) = params.get("protocolVersion") {
            if version != PROTOCOL_VERSION {
                return error_response(
                    Some(request_id),
                    -32602,
                    "Unsupported protocol version",
                    None,
                );
            }
        } else {
            return error_response(Some(request_id), -32600, "Invalid Request", None);
        }
    } else {
        return error_response(Some(request_id), -32601, "Method not found", None);
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
    if let Some(params) = params {
        if let Some(name) = params.get("name") {
            if name == "echo" {
                if let Some(args) = params.get("arguments") {
                    if let Some(Value::String(msg)) = args.get("message") {
                        let result = json!({
                            "content": [
                                {
                                    "type": "text",
                                    "text": msg
                                }
                            ]
                        });
                        JsonRpcResponse {
                            jsonrpc: "2.0".to_string(),
                            id: Some(request_id),
                            result: Some(result),
                            error: None,
                        }
                    } else {
                        error_response(Some(request_id), -32602, "Invalid params", None)
                    }
                } else {
                    error_response(Some(request_id), -32602, "Invalid params", None)
                }
            } else if name == "get_time" {
                let now = chrono::Utc::now().to_rfc3339();
                let result = json!({
                    "content": [
                        {
                            "type": "text",
                            "text": now
                        }
                    ]
                });
                JsonRpcResponse {
                    jsonrpc: "2.0".to_string(),
                    id: Some(request_id),
                    result: Some(result),
                    error: None,
                }
            } else {
                error_response(Some(request_id), -32601, "Method not found", None)
            }
        } else {
            error_response(Some(request_id), -32602, "Invalid params", None)
        }
    } else {
        error_response(Some(request_id), -32600, "Invalid Request", None)
    }
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
