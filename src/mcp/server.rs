//! MCP server implementation for Liath
//!
//! Implements the Model Context Protocol over stdio using JSON-RPC 2.0

use anyhow::Result;
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use std::io::{BufRead, BufReader, Write};

use crate::query::QueryExecutor;
use super::tools::{get_tools, LiathService};

/// JSON-RPC request
#[derive(Debug, Deserialize)]
struct JsonRpcRequest {
    #[allow(dead_code)]
    jsonrpc: String,
    id: Option<Value>,
    method: String,
    #[serde(default)]
    params: Value,
}

/// JSON-RPC response
#[derive(Debug, Serialize)]
struct JsonRpcResponse {
    jsonrpc: String,
    id: Value,
    #[serde(skip_serializing_if = "Option::is_none")]
    result: Option<Value>,
    #[serde(skip_serializing_if = "Option::is_none")]
    error: Option<JsonRpcError>,
}

#[derive(Debug, Serialize)]
struct JsonRpcError {
    code: i32,
    message: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    data: Option<Value>,
}

impl JsonRpcResponse {
    fn success(id: Value, result: Value) -> Self {
        Self {
            jsonrpc: "2.0".to_string(),
            id,
            result: Some(result),
            error: None,
        }
    }

    fn error(id: Value, code: i32, message: String) -> Self {
        Self {
            jsonrpc: "2.0".to_string(),
            id,
            result: None,
            error: Some(JsonRpcError {
                code,
                message,
                data: None,
            }),
        }
    }
}

/// Run the MCP server over stdio
pub async fn run_mcp_server(query_executor: QueryExecutor, user_id: String) -> Result<()> {
    let service = LiathService::new(query_executor, user_id);

    let stdin = std::io::stdin();
    let mut stdout = std::io::stdout();
    let reader = BufReader::new(stdin.lock());

    eprintln!("Liath MCP server started");

    for line in reader.lines() {
        let line = match line {
            Ok(l) => l,
            Err(_) => break,
        };

        if line.is_empty() {
            continue;
        }

        let request: JsonRpcRequest = match serde_json::from_str(&line) {
            Ok(r) => r,
            Err(e) => {
                let response = JsonRpcResponse::error(
                    Value::Null,
                    -32700,
                    format!("Parse error: {}", e),
                );
                writeln!(stdout, "{}", serde_json::to_string(&response)?)?;
                stdout.flush()?;
                continue;
            }
        };

        let id = request.id.clone().unwrap_or(Value::Null);
        let response = handle_request(&service, &request).await;

        let json_response = match response {
            Ok(result) => JsonRpcResponse::success(id, result),
            Err(e) => JsonRpcResponse::error(id, -32603, e),
        };

        writeln!(stdout, "{}", serde_json::to_string(&json_response)?)?;
        stdout.flush()?;
    }

    Ok(())
}

async fn handle_request(service: &LiathService, request: &JsonRpcRequest) -> Result<Value, String> {
    match request.method.as_str() {
        "initialize" => {
            Ok(json!({
                "protocolVersion": "2024-11-05",
                "capabilities": {
                    "tools": {
                        "listChanged": false
                    }
                },
                "serverInfo": {
                    "name": "liath",
                    "version": env!("CARGO_PKG_VERSION")
                },
                "instructions": "Liath is an AI-first database with key-value storage, vector search, and Lua scripting."
            }))
        }

        "initialized" => {
            // Notification, no response needed but we return empty for consistency
            Ok(json!({}))
        }

        "tools/list" => {
            let tools: Vec<Value> = get_tools()
                .into_iter()
                .map(|t| {
                    json!({
                        "name": t.name,
                        "description": t.description,
                        "inputSchema": t.input_schema
                    })
                })
                .collect();
            Ok(json!({ "tools": tools }))
        }

        "tools/call" => {
            let name = request.params.get("name")
                .and_then(|v| v.as_str())
                .ok_or("Missing tool name")?;
            let arguments = request.params.get("arguments")
                .cloned()
                .unwrap_or(json!({}));

            let result = service.handle_tool_call(name, arguments).await;

            // Convert CallToolResult to JSON
            let content: Vec<Value> = result.content
                .into_iter()
                .map(|c| {
                    json!({
                        "type": "text",
                        "text": c.as_text().map(|t| t.text.clone()).unwrap_or_default()
                    })
                })
                .collect();

            Ok(json!({
                "content": content,
                "isError": result.is_error.unwrap_or(false)
            }))
        }

        "ping" => {
            Ok(json!({}))
        }

        "resources/list" => {
            // List namespaces as browsable resources
            let namespaces = service.query_executor.list_namespaces();
            let resources: Vec<Value> = namespaces.iter().map(|ns| {
                json!({
                    "uri": format!("liath://namespace/{}", ns),
                    "name": ns,
                    "mimeType": "application/json",
                    "description": format!("Liath namespace: {}", ns)
                })
            }).collect();
            Ok(json!({ "resources": resources }))
        }

        "resources/read" => {
            let uri = request.params.get("uri")
                .and_then(|v| v.as_str())
                .ok_or("Missing uri")?;

            // Parse the URI to get namespace
            if let Some(ns) = uri.strip_prefix("liath://namespace/") {
                if service.query_executor.namespace_exists(ns) {
                    Ok(json!({
                        "contents": [{
                            "uri": uri,
                            "mimeType": "application/json",
                            "text": format!("Namespace '{}' exists. Use liath_kv_get to read specific keys.", ns)
                        }]
                    }))
                } else {
                    Err(format!("Namespace '{}' not found", ns))
                }
            } else {
                Err(format!("Unknown resource URI: {}", uri))
            }
        }

        "prompts/list" => {
            let prompts = vec![
                json!({
                    "name": "liath-intro",
                    "description": "Get started with Liath database",
                    "arguments": []
                }),
                json!({
                    "name": "semantic-search-example",
                    "description": "Example of storing and searching documents",
                    "arguments": [
                        {
                            "name": "namespace",
                            "description": "Namespace to use",
                            "required": true
                        }
                    ]
                }),
                json!({
                    "name": "agent-memory-example",
                    "description": "Example of using agent memory for AI context",
                    "arguments": [
                        {
                            "name": "agent_id",
                            "description": "Agent ID to use",
                            "required": true
                        }
                    ]
                }),
            ];
            Ok(json!({ "prompts": prompts }))
        }

        "prompts/get" => {
            let name = request.params.get("name")
                .and_then(|v| v.as_str())
                .ok_or("Missing prompt name")?;
            let args = request.params.get("arguments")
                .cloned()
                .unwrap_or(json!({}));

            match name {
                "liath-intro" => Ok(json!({
                    "messages": [{
                        "role": "user",
                        "content": {
                            "type": "text",
                            "text": "Liath is a fast embedded database for running agents. Here are the available tools:\n\n\
                                     - **liath_execute_lua**: Run Lua queries\n\
                                     - **liath_kv_get/put/delete**: Key-value operations\n\
                                     - **liath_create_namespace**: Create storage spaces\n\
                                     - **liath_store_document**: Store text with embeddings\n\
                                     - **liath_semantic_search**: Search by meaning\n\
                                     - **liath_agent_***: Agent memory and conversations\n\n\
                                     Use liath_list_namespaces to see existing namespaces."
                        }
                    }]
                })),
                "semantic-search-example" => {
                    let ns = args.get("namespace")
                        .and_then(|v| v.as_str())
                        .unwrap_or("documents");
                    Ok(json!({
                        "messages": [{
                            "role": "user",
                            "content": {
                                "type": "text",
                                "text": format!("To use semantic search in namespace '{}':\n\n\
                                    1. Create namespace (if needed):\n\
                                       liath_create_namespace(name=\"{}\", dimensions=384)\n\n\
                                    2. Store documents:\n\
                                       liath_store_document(namespace=\"{}\", key=\"doc1\", text=\"content\", id=1)\n\n\
                                    3. Search by meaning:\n\
                                       liath_semantic_search(namespace=\"{}\", query=\"search terms\", k=5)\n\n\
                                    The database automatically generates embeddings for semantic matching.", ns, ns, ns, ns)
                            }
                        }]
                    }))
                }
                "agent-memory-example" => {
                    let agent_id = args.get("agent_id")
                        .and_then(|v| v.as_str())
                        .unwrap_or("my-agent");
                    Ok(json!({
                        "messages": [{
                            "role": "user",
                            "content": {
                                "type": "text",
                                "text": format!("Agent memory example for agent '{}':\n\n\
                                    1. Create agent:\n\
                                       liath_agent_create(id=\"{}\")\n\n\
                                    2. Store memories:\n\
                                       liath_agent_store_memory(agent_id=\"{}\", content=\"important fact\", tags=[\"facts\"])\n\n\
                                    3. Recall by semantic search:\n\
                                       liath_agent_recall_memory(agent_id=\"{}\", query=\"what is the fact?\")\n\n\
                                    4. Recall by tags:\n\
                                       liath_agent_recall_by_tags(agent_id=\"{}\", tags=[\"facts\"])", agent_id, agent_id, agent_id, agent_id, agent_id)
                            }
                        }]
                    }))
                }
                _ => Err(format!("Unknown prompt: {}", name))
            }
        }

        _ => {
            Err(format!("Unknown method: {}", request.method))
        }
    }
}
