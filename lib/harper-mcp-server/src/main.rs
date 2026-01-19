use axum::{routing::post, Router};
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};

#[derive(Serialize, Deserialize)]
struct JsonRpcRequest {
    jsonrpc: String,
    id: Option<Value>,
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

async fn handle_request(
    axum::Json(rpc_req): axum::Json<JsonRpcRequest>,
) -> axum::Json<JsonRpcResponse> {
    eprintln!(
        "MCP request: method={}, id={:?}",
        rpc_req.method, rpc_req.id
    );
    let response = match rpc_req.method.as_str() {
        "initialize" => {
            // Check protocol version
            if let Some(params) = &rpc_req.params {
                if let Some(version) = params.get("protocolVersion") {
                    if version != "2024-11-05" {
                        JsonRpcResponse {
                            jsonrpc: "2.0".to_string(),
                            id: rpc_req.id,
                            result: None,
                            error: Some(JsonRpcError {
                                code: -32602,
                                message: "Unsupported protocol version".to_string(),
                                data: None,
                            }),
                        }
                    } else {
                        JsonRpcResponse {
                            jsonrpc: "2.0".to_string(),
                            id: rpc_req.id,
                            result: Some(json!({
                                "protocolVersion": "2024-11-05",
                                "capabilities": {
                                    "tools": {
                                        "listChanged": true
                                    }
                                },
                                "serverInfo": {
                                    "name": "harper-mcp-server",
                                    "version": "0.1.0"
                                }
                            })),
                            error: None,
                        }
                    }
                } else {
                    JsonRpcResponse {
                        jsonrpc: "2.0".to_string(),
                        id: rpc_req.id,
                        result: None,
                        error: Some(JsonRpcError {
                            code: -32602,
                            message: "Missing protocol version".to_string(),
                            data: None,
                        }),
                    }
                }
            } else {
                JsonRpcResponse {
                    jsonrpc: "2.0".to_string(),
                    id: rpc_req.id,
                    result: None,
                    error: Some(JsonRpcError {
                        code: -32602,
                        message: "Missing params".to_string(),
                        data: None,
                    }),
                }
            }
        }
        "tools/list" => JsonRpcResponse {
            jsonrpc: "2.0".to_string(),
            id: rpc_req.id,
            result: Some(json!({
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
            })),
            error: None,
        },
        "tools/call" => {
            if let Some(params) = rpc_req.params {
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
                                    id: rpc_req.id,
                                    result: Some(result),
                                    error: None,
                                }
                            } else {
                                JsonRpcResponse {
                                    jsonrpc: "2.0".to_string(),
                                    id: rpc_req.id,
                                    result: None,
                                    error: Some(JsonRpcError {
                                        code: -32602,
                                        message: "Invalid params".to_string(),
                                        data: None,
                                    }),
                                }
                            }
                        } else {
                            JsonRpcResponse {
                                jsonrpc: "2.0".to_string(),
                                id: rpc_req.id,
                                result: None,
                                error: Some(JsonRpcError {
                                    code: -32602,
                                    message: "Invalid params".to_string(),
                                    data: None,
                                }),
                            }
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
                            id: rpc_req.id,
                            result: Some(result),
                            error: None,
                        }
                    } else {
                        JsonRpcResponse {
                            jsonrpc: "2.0".to_string(),
                            id: rpc_req.id,
                            result: None,
                            error: Some(JsonRpcError {
                                code: -32601,
                                message: "Method not found".to_string(),
                                data: None,
                            }),
                        }
                    }
                } else {
                    JsonRpcResponse {
                        jsonrpc: "2.0".to_string(),
                        id: rpc_req.id,
                        result: None,
                        error: Some(JsonRpcError {
                            code: -32602,
                            message: "Invalid params".to_string(),
                            data: None,
                        }),
                    }
                }
            } else {
                JsonRpcResponse {
                    jsonrpc: "2.0".to_string(),
                    id: rpc_req.id,
                    result: None,
                    error: Some(JsonRpcError {
                        code: -32600,
                        message: "Invalid Request".to_string(),
                        data: None,
                    }),
                }
            }
        }
        _ => JsonRpcResponse {
            jsonrpc: "2.0".to_string(),
            id: rpc_req.id,
            result: None,
            error: Some(JsonRpcError {
                code: -32601,
                message: "Method not found".to_string(),
                data: None,
            }),
        },
    };

    axum::Json(response)
}

async fn health() -> &'static str {
    "OK"
}

#[tokio::main]
async fn main() {
    let app = Router::new()
        .route("/", post(handle_request))
        .route("/health", axum::routing::get(health));
    let addr = std::net::SocketAddr::from(([127, 0, 0, 1], 5001));
    println!("MCP server listening on http://127.0.0.1:5001");
    axum::serve(tokio::net::TcpListener::bind(addr).await.unwrap(), app)
        .await
        .unwrap();
}
