use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use spy_core::Config;
use spy_embeddings::EmbeddingManager;
use spy_storage::Storage;
use std::path::Path;
use std::sync::{Arc, Mutex};
use tokio::io::{AsyncBufReadExt, AsyncWriteExt, BufReader};

// ---------------------------------------------------------------------------
// MCP JSON-RPC types
// ---------------------------------------------------------------------------

#[derive(Debug, Deserialize)]
struct Request {
    #[allow(dead_code)]
    jsonrpc: String,
    id: Option<Value>,
    method: String,
    params: Option<Value>,
}

#[derive(Debug, Serialize)]
struct Response {
    jsonrpc: String,
    id: Option<Value>,
    #[serde(skip_serializing_if = "Option::is_none")]
    result: Option<Value>,
    #[serde(skip_serializing_if = "Option::is_none")]
    error: Option<JsonRpcError>,
}

#[derive(Debug, Serialize)]
struct JsonRpcError {
    code: i32,
    message: String,
}

impl Response {
    fn ok(id: Option<Value>, result: Value) -> Self {
        Response {
            jsonrpc: "2.0".to_string(),
            id,
            result: Some(result),
            error: None,
        }
    }

    fn err(id: Option<Value>, code: i32, message: impl Into<String>) -> Self {
        Response {
            jsonrpc: "2.0".to_string(),
            id,
            result: None,
            error: Some(JsonRpcError {
                code,
                message: message.into(),
            }),
        }
    }
}

// ---------------------------------------------------------------------------
// Public entry point
// ---------------------------------------------------------------------------

/// Run the MCP stdio server until EOF.  Reads `spy.config.json` from the
/// current directory (or `config_path`) to locate the SQLite database.
pub async fn run_mcp_server(config_path: &Path) -> Result<()> {
    let config_str = std::fs::read_to_string(config_path)
        .context("Failed to read config.  Run 'spy-code init' first.")?;
    let config: Config = serde_json::from_str(&config_str)?;

    let storage = Storage::open(&config.db_path).context("Failed to open database")?;
    let storage = Arc::new(Mutex::new(storage));

    let schema = spy_graph::create_schema(Arc::clone(&storage));

    let stdin = tokio::io::stdin();
    let mut reader = BufReader::new(stdin).lines();
    let mut stdout = tokio::io::stdout();

    while let Some(line) = reader.next_line().await? {
        let line = line.trim().to_string();
        if line.is_empty() {
            continue;
        }

        let response = match serde_json::from_str::<Request>(&line) {
            Err(e) => Response::err(None, -32700, format!("Parse error: {}", e)),
            Ok(req) => handle_request(req, &storage, &schema, &config).await,
        };

        let mut out = serde_json::to_string(&response)?;
        out.push('\n');
        stdout.write_all(out.as_bytes()).await?;
        stdout.flush().await?;
    }

    Ok(())
}

// ---------------------------------------------------------------------------
// Request dispatch
// ---------------------------------------------------------------------------

async fn handle_request(
    req: Request,
    storage: &Arc<Mutex<Storage>>,
    schema: &spy_graph::SpySchema,
    config: &Config,
) -> Response {
    match req.method.as_str() {
        "initialize" => Response::ok(
            req.id,
            json!({
                "protocolVersion": "2024-11-05",
                "capabilities": {
                    "tools": {},
                    "resources": {}
                },
                "serverInfo": {
                    "name": "spy-code",
                    "version": env!("CARGO_PKG_VERSION")
                },
                "instructions": "Spy-Code MCP Server - GraphQL-style compiler for codebases with semantic search capabilities.\n\nThis server provides tools for exploring, querying, and understanding codebases through a graph-based representation. Use it to:\n\n1. **Navigate the codebase**: Find functions, classes, and their relationships\n2. **Understand call graphs**: Trace callers and callees to understand impact\n3. **Semantic search**: Use natural language to find relevant code via embeddings\n4. **Track changes**: See what changed since a git reference\n\n**Available Tools:**\n- `query_graph`: Run raw GraphQL queries for complex operations\n- `get_node`: Get details of a specific node by ID\n- `search`: Fuzzy search for nodes by name/description\n- `find_callers`/`find_callees`: Navigate call graphs\n- `changed_since`: Find nodes changed since a git ref\n- `stats`: Get index statistics\n- `embed`: Generate embeddings for semantic search (run once per codebase)\n- `ask`: Ask natural language questions about the codebase using semantic search\n\n**Best Practices:**\n- Start with `search` or `ask` to find relevant code\n- Use `get_node` to get detailed information\n- Use `find_callers`/`find_callees` to understand relationships\n- Run `embed` once to enable semantic search with `ask`\n- Use `changed_since` after rebasing to find what to re-read\n\n**Node IDs:** Node IDs follow the format: `dir:file:class:symbol` (e.g., `src:main.rs:main:main`)"
            }),
        ),
        "initialized" => Response::ok(req.id, json!(null)),
        "tools/list" => Response::ok(req.id, tools_list()),
        "tools/call" => match handle_tool_call(req.params, storage, schema, config).await {
            Ok(result) => Response::ok(req.id, result),
            Err(e) => Response::err(req.id, -32603, e.to_string()),
        },
        "resources/list" => Response::ok(req.id, resources_list()),
        "resources/read" => match handle_resource_read(req.params, storage, schema, config).await {
            Ok(result) => Response::ok(req.id, result),
            Err(e) => Response::err(req.id, -32603, e.to_string()),
        },
        other => Response::err(req.id, -32601, format!("Method not found: {}", other)),
    }
}

// ---------------------------------------------------------------------------
// Tools
// ---------------------------------------------------------------------------

fn tools_list() -> Value {
    json!({
        "tools": [
            {
                "name": "query_graph",
                "description": "Run a raw GraphQL query against the codebase graph. Use this for complex multi-hop queries. Schema is documented at spy-code://schema. Prefer the specialized tools below for common ops.",
                "inputSchema": {
                    "type": "object",
                    "properties": {
                        "query": { "type": "string" },
                        "variables": { "type": "object" }
                    },
                    "required": ["query"]
                }
            },
            {
                "name": "get_node",
                "description": "Fetch one node by its ID. Node IDs are 'dir:file:class:symbol'. Returns name, description, signatures (params + returns), and location.",
                "inputSchema": {
                    "type": "object",
                    "properties": {
                        "node_id": { "type": "string" }
                    },
                    "required": ["node_id"]
                }
            },
            {
                "name": "search",
                "description": "Find nodes by fuzzy name/description match. Use this when you know roughly what something is called but not its exact ID.",
                "inputSchema": {
                    "type": "object",
                    "properties": {
                        "query": { "type": "string" },
                        "kind": { "type": "string", "enum": ["function", "class", "constant"] },
                        "limit": { "type": "integer" }
                    },
                    "required": ["query"]
                }
            },
            {
                "name": "find_callers",
                "description": "List all functions/methods that call the given node. Use depth > 1 to walk transitively up the call graph.",
                "inputSchema": {
                    "type": "object",
                    "properties": {
                        "node_id": { "type": "string" },
                        "depth": { "type": "integer" }
                    },
                    "required": ["node_id"]
                }
            },
            {
                "name": "find_callees",
                "description": "List all functions/methods called by the given node. Use depth > 1 to walk transitively down.",
                "inputSchema": {
                    "type": "object",
                    "properties": {
                        "node_id": { "type": "string" },
                        "depth": { "type": "integer" }
                    },
                    "required": ["node_id"]
                }
            },
            {
                "name": "changed_since",
                "description": "List nodes whose source changed since the given git ref. Use this to find what an AI agent needs to re-read after a rebase.",
                "inputSchema": {
                    "type": "object",
                    "properties": {
                        "git_ref": { "type": "string" }
                    },
                    "required": ["git_ref"]
                }
            },
            {
                "name": "stats",
                "description": "Return index statistics (node count, edge count, file count, last indexed SHA).",
                "inputSchema": {
                    "type": "object",
                    "properties": {}
                }
            },
            {
                "name": "embed",
                "description": "Generate embeddings for semantic search. This should be run once after indexing to enable natural language queries. Uses TF-IDF-based embedding generation.",
                "inputSchema": {
                    "type": "object",
                    "properties": {
                        "full": {
                            "type": "boolean",
                            "description": "Force full re-embedding of all nodes (default: false)"
                        }
                    }
                }
            },
            {
                "name": "ask",
                "description": "Ask natural language questions about the codebase using semantic search. Returns relevant code nodes ranked by similarity. Requires embeddings to be generated first (run 'embed' tool).",
                "inputSchema": {
                    "type": "object",
                    "properties": {
                        "query": {
                            "type": "string",
                            "description": "Natural language question or search query"
                        },
                        "limit": {
                            "type": "integer",
                            "description": "Maximum number of results to return (default: 20)"
                        }
                    },
                    "required": ["query"]
                }
            }
        ]
    })
}

async fn handle_tool_call(
    params: Option<Value>,
    storage: &Arc<Mutex<Storage>>,
    schema: &spy_graph::SpySchema,
    config: &Config,
) -> Result<Value> {
    let params = params.unwrap_or_default();
    let name = params
        .get("name")
        .and_then(|v| v.as_str())
        .context("Missing tool name")?;
    let args = params.get("arguments").cloned().unwrap_or_default();

    match name {
        "query_graph" => {
            let query = args
                .get("query")
                .and_then(|v| v.as_str())
                .context("Missing 'query'")?;
            let result = schema.execute(query).await;
            let json = serde_json::to_value(&result)?;
            Ok(json!({ "content": [{ "type": "text", "text": json.to_string() }] }))
        }

        "get_node" => {
            let node_id = args
                .get("node_id")
                .and_then(|v| v.as_str())
                .context("Missing 'node_id'")?;
            let st = storage.lock().unwrap();
            let node = st.get_node(node_id)?;
            Ok(json!({ "content": [{ "type": "text", "text": serde_json::to_string(&node)? }] }))
        }

        "search" => {
            let query = args
                .get("query")
                .and_then(|v| v.as_str())
                .context("Missing 'query'")?;
            let limit = args.get("limit").and_then(|v| v.as_u64()).unwrap_or(20) as usize;
            let kind_filter = args.get("kind").and_then(|v| v.as_str());
            let st = storage.lock().unwrap();
            let mut results = st.search_nodes(query, limit)?;
            if let Some(k) = kind_filter {
                results.retain(|(n, _)| n.kind.as_str() == k);
            }
            Ok(
                json!({ "content": [{ "type": "text", "text": serde_json::to_string(&results.iter().map(|(n, s)| json!({ "node_id": n.node_id.as_str(), "name": &n.name, "kind": n.kind.as_str(), "score": s })).collect::<Vec<_>>())? }] }),
            )
        }

        "find_callers" => {
            let gql = format!(
                r#"{{ callers(id: "{}", depth: {}) {{ fromId toId confidence }} }}"#,
                args.get("node_id").and_then(|v| v.as_str()).unwrap_or(""),
                args.get("depth").and_then(|v| v.as_i64()).unwrap_or(1)
            );
            let result = schema.execute(&gql).await;
            Ok(
                json!({ "content": [{ "type": "text", "text": serde_json::to_value(&result)?.to_string() }] }),
            )
        }

        "find_callees" => {
            let gql = format!(
                r#"{{ callees(id: "{}", depth: {}) {{ fromId toId confidence }} }}"#,
                args.get("node_id").and_then(|v| v.as_str()).unwrap_or(""),
                args.get("depth").and_then(|v| v.as_i64()).unwrap_or(1)
            );
            let result = schema.execute(&gql).await;
            Ok(
                json!({ "content": [{ "type": "text", "text": serde_json::to_value(&result)?.to_string() }] }),
            )
        }

        "changed_since" => {
            let git_ref = args
                .get("git_ref")
                .and_then(|v| v.as_str())
                .context("Missing 'git_ref'")?;
            let paths = spy_git::GitRepo::discover(Path::new("."))
                .context("Failed to open git repo")?
                .map(|repo| repo.files_changed_since_ref(git_ref))
                .transpose()
                .context("git diff failed")?
                .unwrap_or_default();
            let path_strings: Vec<String> = paths
                .into_iter()
                .map(|p| p.to_string_lossy().into_owned())
                .collect();
            let nodes = storage.lock().unwrap().get_nodes_for_files(&path_strings)?;
            Ok(
                json!({ "content": [{ "type": "text", "text": serde_json::to_string(&nodes.iter().map(|n| json!({ "node_id": n.node_id.as_str(), "name": &n.name, "file_path": &n.file_path })).collect::<Vec<_>>())? }] }),
            )
        }

        "stats" => {
            let stats = storage.lock().unwrap().get_stats()?;
            Ok(
                json!({ "content": [{ "type": "text", "text": serde_json::to_string(&json!({ "node_count": stats.node_count, "edge_count": stats.edge_count, "file_count": stats.file_count, "last_git_sha": stats.last_git_sha }))? }] }),
            )
        }

        "embed" => {
            let _full = args.get("full").and_then(|v| v.as_bool()).unwrap_or(false);
            let embedding_storage = Storage::open(&config.db_path)?;
            let mut embedding_manager = EmbeddingManager::new(embedding_storage);
            embedding_manager.initialize_schema()?;

            let model_path = std::path::PathBuf::from(".spy-code/models/all-MiniLM-L6-v2");
            embedding_manager.generate_node_embeddings(&model_path)?;

            Ok(json!({ "content": [{ "type": "text", "text": "Embeddings generated successfully. You can now use the 'ask' tool for semantic search." }] }))
        }

        "ask" => {
            let query = args
                .get("query")
                .and_then(|v| v.as_str())
                .context("Missing 'query'")?;
            let limit = args.get("limit").and_then(|v| v.as_u64()).unwrap_or(20) as usize;

            let embedding_storage = Storage::open(&config.db_path)?;
            let embedding_manager = EmbeddingManager::new(embedding_storage);
            let results = embedding_manager.semantic_search(query, limit)?;

            let results_json: Vec<_> = results
                .iter()
                .map(|(node, score)| json!({
                    "node_id": node.node_id.as_str(),
                    "name": &node.name,
                    "kind": node.kind.as_str(),
                    "file_path": &node.file_path,
                    "start_line": node.start_line,
                    "score": score
                }))
                .collect();

            Ok(json!({ "content": [{ "type": "text", "text": serde_json::to_string(&results_json)? }] }))
        }

        other => anyhow::bail!("Unknown tool: {}", other),
    }
}

// ---------------------------------------------------------------------------
// Resources
// ---------------------------------------------------------------------------

fn resources_list() -> Value {
    json!({
        "resources": [
            {
                "uri": "spy-code://schema",
                "name": "GraphQL Schema",
                "description": "Full GraphQL SDL for the spy-code graph API",
                "mimeType": "text/plain"
            },
            {
                "uri": "spy-code://stats",
                "name": "Index Stats",
                "description": "Current index statistics",
                "mimeType": "application/json"
            },
            {
                "uri": "spy-code://config",
                "name": "Config",
                "description": "Loaded configuration (sanitized)",
                "mimeType": "application/json"
            }
        ]
    })
}

async fn handle_resource_read(
    params: Option<Value>,
    storage: &Arc<Mutex<Storage>>,
    schema: &spy_graph::SpySchema,
    config: &Config,
) -> Result<Value> {
    let uri = params
        .as_ref()
        .and_then(|p| p.get("uri"))
        .and_then(|v| v.as_str())
        .context("Missing 'uri'")?;

    match uri {
        "spy-code://schema" => {
            let sdl = schema.sdl();
            Ok(json!({
                "contents": [{
                    "uri": uri,
                    "mimeType": "text/plain",
                    "text": sdl
                }]
            }))
        }
        "spy-code://stats" => {
            let stats = storage.lock().unwrap().get_stats()?;
            Ok(json!({
                "contents": [{
                    "uri": uri,
                    "mimeType": "application/json",
                    "text": serde_json::to_string(&json!({
                        "node_count": stats.node_count,
                        "edge_count": stats.edge_count,
                        "file_count": stats.file_count,
                        "last_git_sha": stats.last_git_sha
                    }))?
                }]
            }))
        }
        "spy-code://config" => Ok(json!({
            "contents": [{
                "uri": uri,
                "mimeType": "application/json",
                "text": serde_json::to_string_pretty(config)?
            }]
        })),
        other => anyhow::bail!("Unknown resource URI: {}", other),
    }
}
