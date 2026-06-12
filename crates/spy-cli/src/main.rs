use anyhow::{Context, Result};
use clap::{Parser, Subcommand};
use spy_core::{Config, EdgeKind, NodeKind};
use spy_embeddings::{EmbeddingManager, ModelRegistry};
use spy_indexer::Indexer;
use spy_storage::Storage;
use std::path::PathBuf;
use std::sync::{Arc, Mutex};

const GRAPH_UI_HTML: &str = include_str!("../../../crates/spy-graph-ui/index.html");

#[derive(Parser)]
#[command(name = "spy-code")]
#[command(version = env!("CARGO_PKG_VERSION"))]
#[command(about = "GraphQL-style compiler for codebases", long_about = None)]
#[command(after_help = "
AI CODING ENVIRONMENT SETUP:
  For automatic setup with Cursor, Windsurf, Claude Desktop, or GitHub Copilot:
    ./scripts/install-spy-code-skill.sh

  For manual MCP configuration, see docs/INTEGRATIONS.md
")]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Initialize a new spy.config.json in the current directory
    Init,
    /// Index the codebase files to extract nodes and edges
    Index {
        /// Force full re-indexing of all files
        #[arg(long)]
        full: bool,
        /// Path to the codebase root to index
        #[arg(long, default_value = ".")]
        path: PathBuf,
    },
    /// Run a custom GraphQL query against the indexed codebase graph
    Query {
        /// GraphQL query string
        query: String,
        /// Output the raw JSON response
        #[arg(long)]
        json: bool,
    },
    /// Get details of a specific node by its ID
    Get {
        /// The unique ID of the node to inspect
        node_id: String,
    },
    /// Search the codebase for symbol names, descriptions, or comments
    Search {
        /// Search query/term
        text: String,
        /// Filter by node kind (e.g. function, class, constant)
        #[arg(long)]
        kind: Option<String>,
        /// Use local TF-IDF semantic vector search
        #[arg(long)]
        semantic: bool,
        /// Use hybrid (FTS5 + TF-IDF RRF) search
        #[arg(long)]
        hybrid: bool,
    },
    /// List all incoming callers of a function
    Callers {
        /// Unique ID of the target function
        node_id: String,
        /// Maximum transitive call depth to traverse
        #[arg(long, default_value = "1")]
        depth: i32,
    },
    /// List all outgoing callees of a function
    Callees {
        /// Unique ID of the source function
        node_id: String,
        /// Maximum transitive call depth to traverse
        #[arg(long, default_value = "1")]
        depth: i32,
    },
    /// List all nodes that changed since a specific git reference
    Changed {
        /// Git commit hash, branch, or reference (e.g. HEAD~1, main)
        git_ref: String,
    },
    /// Get statistics of the codebase index
    Stats,
    /// Serve the GraphQL API over HTTP or MCP (Model Context Protocol)
    Serve {
        /// Start in MCP server mode for direct integration with LLM clients
        #[arg(long)]
        mcp: bool,
        /// Start in HTTP mode with an interactive GraphiQL playground
        #[arg(long)]
        http: bool,
        /// Port to bind the HTTP server to
        #[arg(long, default_value = "4000")]
        port: u16,
    },
    /// List the base classes, interfaces, or traits that a node implements or inherits from
    Bases {
        /// Unique ID of the class, struct, or interface
        node_id: String,
    },
    /// List the derived subclasses, interfaces, or structures implementing a base trait/class
    Derived {
        /// Unique ID of the base class, interface, or trait
        node_id: String,
    },
    /// Diff symbol signatures and bodies between two git references
    DiffSymbols {
        /// Git reference to diff from
        from_ref: String,
        /// Git reference to diff to
        to_ref: String,
    },
    /// Find the shortest call path between two functions in the call graph
    CallPath {
        /// Unique ID of the source function
        from_id: String,
        /// Unique ID of the target function
        to_id: String,
    },
    /// Analyze the downstream impact of changing a symbol (transitive callers)
    Impact {
        /// Unique ID of the symbol to analyze
        node_id: String,
        /// Maximum traversal depth in the call graph
        #[arg(long, default_value = "3")]
        depth: i32,
    },
    /// List external project package dependencies parsed from config files
    Dependencies,
    /// List code elements inside currently modified or active workspace files in Git
    ActiveContext,
    /// Generate embeddings for semantic search
    Embed {
        /// Force full re-embedding of all nodes
        #[arg(long)]
        full: bool,
        /// Model name to use for embeddings
        #[arg(long)]
        model: Option<String>,
    },
    /// Manage embedding models
    Model {
        #[command(subcommand)]
        command: ModelCommands,
    },
    /// Ask natural language questions about the codebase
    Ask {
        /// Natural language query
        query: String,
        /// Output the raw JSON response
        #[arg(long)]
        json: bool,
    },
    /// Generate and serve graph visualization
    Graph {
        /// Path to the codebase root
        #[arg(long, default_value = ".")]
        path: PathBuf,
        /// Open browser automatically
        #[arg(long)]
        open: bool,
    },
    /// Install spy-code skills for AI coding environments (Cursor, Windsurf, Claude Desktop, etc.)
    InstallSkills {
        /// Show what would be done without making changes
        #[arg(long)]
        dry_run: bool,
        /// Skip codebase indexing
        #[arg(long)]
        skip_index: bool,
        /// Force re-creation of spy.config.json
        #[arg(long)]
        force_config: bool,
    },
}

#[derive(Subcommand)]
enum ModelCommands {
    /// List available embedding models
    List,
    /// Download a model
    Download {
        /// Model name to download
        name: String,
    },
}

#[tokio::main]
async fn main() -> Result<()> {
    let cli = Cli::parse();

    match cli.command {
        Commands::Init => cmd_init()?,
        Commands::Index { full, path } => cmd_index(full, path)?,
        Commands::Query { query, json } => cmd_query(query, json).await?,
        Commands::Get { node_id } => cmd_get(node_id).await?,
        Commands::Search {
            text,
            kind,
            semantic,
            hybrid,
        } => cmd_search(text, kind, semantic, hybrid).await?,
        Commands::Callers { node_id, depth } => cmd_callers(node_id, depth).await?,
        Commands::Callees { node_id, depth } => cmd_callees(node_id, depth).await?,
        Commands::Changed { git_ref } => cmd_changed(git_ref).await?,
        Commands::Stats => cmd_stats().await?,
        Commands::Serve { mcp, http, port } => cmd_serve(mcp, http, port).await?,
        Commands::Bases { node_id } => cmd_bases(node_id).await?,
        Commands::Derived { node_id } => cmd_derived(node_id).await?,
        Commands::DiffSymbols { from_ref, to_ref } => cmd_diff_symbols(from_ref, to_ref).await?,
        Commands::CallPath { from_id, to_id } => cmd_call_path(from_id, to_id).await?,
        Commands::Impact { node_id, depth } => cmd_impact(node_id, depth).await?,
        Commands::Dependencies => cmd_dependencies().await?,
        Commands::ActiveContext => cmd_active_context().await?,
        Commands::Embed { full, model } => cmd_embed(full, model)?,
        Commands::Model { command } => cmd_model(command)?,
        Commands::Ask { query, json } => cmd_ask(query, json).await?,
        Commands::Graph { path, open } => cmd_graph(path, open).await?,
        Commands::InstallSkills {
            dry_run,
            skip_index,
            force_config,
        } => cmd_install_skills(dry_run, skip_index, force_config)?,
    }

    Ok(())
}

fn cmd_init() -> Result<()> {
    let config = Config::default();
    let json = serde_json::to_string_pretty(&config)?;
    std::fs::write("spy.config.json", json)?;
    println!("Created spy.config.json");
    Ok(())
}

fn cmd_index(full: bool, path: PathBuf) -> Result<()> {
    let config = load_config()?;
    let storage = Storage::open(&config.db_path)?;
    let mut indexer = Indexer::new(storage, config);

    println!("Indexing {} (full={})", path.display(), full);
    let stats = indexer.index(&path, full)?;

    println!("Indexed {} files", stats.files_scanned);
    println!("  Parsed: {}", stats.files_parsed);
    println!("  Failed: {}", stats.files_failed);
    println!("  Nodes: {}", stats.nodes_extracted);
    println!("  Edges: {}", stats.edges_extracted);

    Ok(())
}

async fn cmd_query(query_str: String, _json: bool) -> Result<()> {
    let config = load_config()?;
    let storage = Storage::open(&config.db_path)?;
    let storage = Arc::new(Mutex::new(storage));

    let schema = spy_graph::create_schema(storage);
    let result = schema.execute(&query_str).await;

    println!("{}", serde_json::to_string_pretty(&result)?);

    Ok(())
}

async fn cmd_get(node_id: String) -> Result<()> {
    let config = load_config()?;
    let storage = Storage::open(&config.db_path)?;

    if let Some(node) = storage.get_node(&node_id)? {
        println!("Node: {}", node.name);
        println!("  ID: {}", node.node_id);
        println!("  Kind: {}", node.kind);
        println!("  Language: {}", node.language);
        println!(
            "  File: {}:{}:{}",
            node.file_path, node.start_line, node.end_line
        );
        if let Some(desc) = node.description {
            println!("  Description: {}", desc);
        }
        if !node.signatures.is_empty() {
            println!("  Signatures:");
            for sig in &node.signatures {
                println!("    Params: {:?}", sig.params);
                if let Some(ret) = &sig.returns {
                    println!("    Returns: {}", ret);
                }
            }
        }
    } else {
        println!("Node not found: {}", node_id);
    }

    Ok(())
}

async fn cmd_search(
    text: String,
    kind: Option<String>,
    semantic: bool,
    hybrid: bool,
) -> Result<()> {
    let limit = 20;
    if semantic {
        let query = format!(
            r#"query {{
                semanticSearch(query: "{}", limit: {}) {{
                    node {{
                        id
                        kind
                        name
                        filePath
                        startLine
                    }}
                    score
                }}
            }}"#,
            text.replace('"', "\\\""),
            limit
        );
        let data = execute_query(&query).await?;
        let results = data["semanticSearch"]
            .as_array()
            .context("Invalid response format")?;

        let filtered_results: Vec<_> = if let Some(ref kind_str) = kind {
            results
                .iter()
                .filter(|r| {
                    r["node"]["kind"].as_str().map(|k| k.to_lowercase())
                        == Some(kind_str.to_lowercase())
                })
                .collect()
        } else {
            results.iter().collect()
        };

        println!("Found {} semantic results:", filtered_results.len());
        for res in filtered_results {
            let node = &res["node"];
            println!(
                "  {} ({}) - {} (score: {:.4}) [{}:{}]",
                node["id"].as_str().unwrap_or(""),
                node["kind"].as_str().unwrap_or(""),
                node["name"].as_str().unwrap_or(""),
                res["score"].as_f64().unwrap_or(0.0),
                node["filePath"].as_str().unwrap_or(""),
                node["startLine"].as_i64().unwrap_or(0)
            );
        }
    } else if hybrid {
        let query = format!(
            r#"query {{
                hybridSearch(query: "{}", limit: {}) {{
                    node {{
                        id
                        kind
                        name
                        filePath
                        startLine
                    }}
                    score
                }}
            }}"#,
            text.replace('"', "\\\""),
            limit
        );
        let data = execute_query(&query).await?;
        let results = data["hybridSearch"]
            .as_array()
            .context("Invalid response format")?;

        let filtered_results: Vec<_> = if let Some(ref kind_str) = kind {
            results
                .iter()
                .filter(|r| {
                    r["node"]["kind"].as_str().map(|k| k.to_lowercase())
                        == Some(kind_str.to_lowercase())
                })
                .collect()
        } else {
            results.iter().collect()
        };

        println!("Found {} hybrid results:", filtered_results.len());
        for res in filtered_results {
            let node = &res["node"];
            println!(
                "  {} ({}) - {} (score: {:.4}) [{}:{}]",
                node["id"].as_str().unwrap_or(""),
                node["kind"].as_str().unwrap_or(""),
                node["name"].as_str().unwrap_or(""),
                res["score"].as_f64().unwrap_or(0.0),
                node["filePath"].as_str().unwrap_or(""),
                node["startLine"].as_i64().unwrap_or(0)
            );
        }
    } else {
        let config = load_config()?;
        let storage = Storage::open(&config.db_path)?;
        let results = storage.search_nodes(&text, limit)?;
        let results: Vec<_> = if let Some(kind_str) = kind {
            let target_kind = match kind_str.as_str() {
                "function" | "fn" | "FUNCTION" => NodeKind::Function,
                "class" | "CLASS" => NodeKind::Class,
                "constant" | "const" | "CONSTANT" => NodeKind::Constant,
                "dependency" | "DEPENDENCY" => NodeKind::Dependency,
                _ => anyhow::bail!("Invalid kind: {}", kind_str),
            };
            results
                .into_iter()
                .filter(|(n, _)| n.kind == target_kind)
                .collect()
        } else {
            results
        };

        println!("Found {} keyword results:", results.len());
        for (node, score) in results {
            println!(
                "  {} ({}) - {} (score: {:.4}) [{}:{}]",
                node.node_id, node.kind, node.name, score, node.file_path, node.start_line
            );
        }
    }
    Ok(())
}

async fn cmd_callers(node_id: String, depth: i32) -> Result<()> {
    let config = load_config()?;
    let storage = Storage::open(&config.db_path)?;

    let edges = storage.get_incoming_edges_transitive(&node_id, EdgeKind::Calls, depth)?;

    println!("Callers of {} (depth {}):", node_id, depth);
    for edge in edges {
        println!("  {} (confidence: {:.2})", edge.from_id, edge.confidence);
    }

    Ok(())
}

async fn cmd_callees(node_id: String, depth: i32) -> Result<()> {
    let config = load_config()?;
    let storage = Storage::open(&config.db_path)?;

    let edges = storage.get_edges_transitive(&node_id, EdgeKind::Calls, depth)?;

    println!("Callees of {} (depth {}):", node_id, depth);
    for edge in edges {
        println!("  {} (confidence: {:.2})", edge.to_id, edge.confidence);
    }

    Ok(())
}

async fn cmd_changed(git_ref: String) -> Result<()> {
    let config = load_config()?;
    let storage = Storage::open(&config.db_path)?;

    let repo = spy_git::GitRepo::discover(std::path::Path::new("."))
        .context("Failed to inspect git repository")?;

    let Some(repo) = repo else {
        anyhow::bail!("Not inside a git repository");
    };

    let changed_paths = repo
        .files_changed_since_ref(&git_ref)
        .context("Failed to compute changed files")?;

    if changed_paths.is_empty() {
        println!("No changed files since {}", git_ref);
        return Ok(());
    }

    let path_strings: Vec<String> = changed_paths
        .iter()
        .map(|p| p.to_string_lossy().into_owned())
        .collect();

    let nodes = storage.get_nodes_for_files(&path_strings)?;

    println!("Nodes changed since {}:", git_ref);
    for node in &nodes {
        println!("  {} ({}) — {}", node.node_id, node.kind, node.name);
    }
    if nodes.is_empty() {
        println!("  (no indexed nodes in changed files)");
    }

    Ok(())
}

async fn cmd_stats() -> Result<()> {
    let config = load_config()?;
    let storage = Storage::open(&config.db_path)?;

    let stats = storage.get_stats()?;

    println!("Index Statistics:");
    println!("  Nodes: {}", stats.node_count);
    println!("  Edges: {}", stats.edge_count);
    println!("  Files: {}", stats.file_count);
    if let Some(sha) = stats.last_git_sha {
        println!("  Last Git SHA: {}", sha);
    }

    Ok(())
}

async fn cmd_serve(mcp: bool, http: bool, port: u16) -> Result<()> {
    if mcp {
        spy_mcp::run_mcp_server(std::path::Path::new("spy.config.json")).await?;
        return Ok(());
    }

    if http {
        let config = load_config()?;
        let storage = Storage::open(&config.db_path)?;
        let storage = Arc::new(Mutex::new(storage));

        let schema = spy_graph::create_schema(storage);

        use async_graphql_axum::{GraphQLRequest, GraphQLResponse};
        use axum::{
            extract::State,
            response::{Html, IntoResponse},
            routing::get,
            Router,
        };
        use tower_http::cors::CorsLayer;

        async fn graphql_handler(
            State(schema): State<spy_graph::SpySchema>,
            req: GraphQLRequest,
        ) -> GraphQLResponse {
            schema.execute(req.into_inner()).await.into()
        }

        async fn graphql_playground() -> impl IntoResponse {
            Html(
                async_graphql::http::GraphiQLSource::build()
                    .endpoint("/")
                    .finish(),
            )
        }

        async fn graph_ui() -> impl IntoResponse {
            Html(GRAPH_UI_HTML)
        }

        let app = Router::new()
            .route("/", get(graphql_playground).post(graphql_handler))
            .route("/graph", get(graph_ui))
            .layer(CorsLayer::permissive())
            .with_state(schema);

        let addr = format!("127.0.0.1:{}", port);
        println!("GraphQL server listening on http://{}", addr);
        println!("Playground: http://{}/", addr);
        println!("Graph UI: http://{}/graph", addr);

        let listener = tokio::net::TcpListener::bind(&addr).await?;
        axum::serve(listener, app).await?;
    }

    Ok(())
}

fn load_config() -> Result<Config> {
    let config_str = std::fs::read_to_string("spy.config.json")
        .context("Failed to read spy.config.json. Run 'spy-code init' first.")?;
    let config: Config = serde_json::from_str(&config_str)?;
    Ok(config)
}

async fn execute_query(query_str: &str) -> Result<serde_json::Value> {
    let config = load_config()?;
    let storage = Storage::open(&config.db_path)?;
    let storage = Arc::new(Mutex::new(storage));
    let schema = spy_graph::create_schema(storage);
    let result = schema.execute(query_str).await;
    if !result.errors.is_empty() {
        anyhow::bail!("GraphQL Error: {:?}", result.errors);
    }
    let data = serde_json::to_value(result.data)?;
    Ok(data)
}

async fn cmd_bases(node_id: String) -> Result<()> {
    let query = format!(
        r#"query {{
            node(id: "{}") {{
                bases {{
                    id
                    kind
                    name
                    filePath
                    startLine
                }}
            }}
        }}"#,
        node_id.replace('"', "\\\"")
    );
    let data = execute_query(&query).await?;
    let node = &data["node"];
    if node.is_null() {
        println!("Node not found: {}", node_id);
        return Ok(());
    }
    let bases = node["bases"]
        .as_array()
        .context("Invalid response format")?;
    println!("Base classes/interfaces/traits of {}:", node_id);
    for base in bases {
        println!(
            "  {} ({}) - {} [{}:{}]",
            base["id"].as_str().unwrap_or(""),
            base["kind"].as_str().unwrap_or(""),
            base["name"].as_str().unwrap_or(""),
            base["filePath"].as_str().unwrap_or(""),
            base["startLine"].as_i64().unwrap_or(0)
        );
    }
    if bases.is_empty() {
        println!("  (none found)");
    }
    Ok(())
}

async fn cmd_derived(node_id: String) -> Result<()> {
    let query = format!(
        r#"query {{
            node(id: "{}") {{
                derived {{
                    id
                    kind
                    name
                    filePath
                    startLine
                }}
            }}
        }}"#,
        node_id.replace('"', "\\\"")
    );
    let data = execute_query(&query).await?;
    let node = &data["node"];
    if node.is_null() {
        println!("Node not found: {}", node_id);
        return Ok(());
    }
    let derived = node["derived"]
        .as_array()
        .context("Invalid response format")?;
    println!("Derived classes/structs implementing {}:", node_id);
    for der in derived {
        println!(
            "  {} ({}) - {} [{}:{}]",
            der["id"].as_str().unwrap_or(""),
            der["kind"].as_str().unwrap_or(""),
            der["name"].as_str().unwrap_or(""),
            der["filePath"].as_str().unwrap_or(""),
            der["startLine"].as_i64().unwrap_or(0)
        );
    }
    if derived.is_empty() {
        println!("  (none found)");
    }
    Ok(())
}

async fn cmd_diff_symbols(from_ref: String, to_ref: String) -> Result<()> {
    let query = format!(
        r#"query {{
            diffSymbols(fromRef: "{}", toRef: "{}") {{
                filePath
                symbolName
                changeType
            }}
        }}"#,
        from_ref.replace('"', "\\\""),
        to_ref.replace('"', "\\\"")
    );
    let data = execute_query(&query).await?;
    let diffs = data["diffSymbols"]
        .as_array()
        .context("Invalid response format")?;
    println!("Symbol diffs between {} and {}:", from_ref, to_ref);
    for diff in diffs {
        println!(
            "  [{}] {}::{}",
            diff["changeType"].as_str().unwrap_or(""),
            diff["filePath"].as_str().unwrap_or(""),
            diff["symbolName"].as_str().unwrap_or("")
        );
    }
    if diffs.is_empty() {
        println!("  (no symbol-level differences found)");
    }
    Ok(())
}

async fn cmd_call_path(from_id: String, to_id: String) -> Result<()> {
    let query = format!(
        r#"query {{
            callPath(sourceNodeId: "{}", targetNodeId: "{}") {{
                fromId
                toId
                confidence
            }}
        }}"#,
        from_id.replace('"', "\\\""),
        to_id.replace('"', "\\\"")
    );
    let data = execute_query(&query).await?;
    let path = data["callPath"]
        .as_array()
        .context("Invalid response format")?;
    println!("Shortest call path from {} to {}:", from_id, to_id);
    for edge in path {
        println!(
            "  {} -> {} (confidence: {:.2})",
            edge["fromId"].as_str().unwrap_or(""),
            edge["toId"].as_str().unwrap_or(""),
            edge["confidence"].as_f64().unwrap_or(0.0)
        );
    }
    if path.is_empty() {
        println!("  (no call path found)");
    }
    Ok(())
}

async fn cmd_impact(node_id: String, depth: i32) -> Result<()> {
    let query = format!(
        r#"query {{
            impactedSymbols(nodeId: "{}", maxDepth: {}) {{
                id
                kind
                name
                filePath
                startLine
            }}
        }}"#,
        node_id.replace('"', "\\\""),
        depth
    );
    let data = execute_query(&query).await?;
    let symbols = data["impactedSymbols"]
        .as_array()
        .context("Invalid response format")?;
    println!(
        "Downstream impacted symbols for {} (depth {}):",
        node_id, depth
    );
    for node in symbols {
        println!(
            "  {} ({}) - {} [{}:{}]",
            node["id"].as_str().unwrap_or(""),
            node["kind"].as_str().unwrap_or(""),
            node["name"].as_str().unwrap_or(""),
            node["filePath"].as_str().unwrap_or(""),
            node["startLine"].as_i64().unwrap_or(0)
        );
    }
    if symbols.is_empty() {
        println!("  (none found)");
    }
    Ok(())
}

async fn cmd_dependencies() -> Result<()> {
    let query = r#"query {
        projectDependencies {
            id
            name
            description
            filePath
        }
    }"#;
    let data = execute_query(query).await?;
    let deps = data["projectDependencies"]
        .as_array()
        .context("Invalid response format")?;
    println!("External project dependencies:");
    for dep in deps {
        println!(
            "  {} - {} ({}) [{}]",
            dep["id"].as_str().unwrap_or(""),
            dep["name"].as_str().unwrap_or(""),
            dep["description"].as_str().unwrap_or(""),
            dep["filePath"].as_str().unwrap_or("")
        );
    }
    if deps.is_empty() {
        println!("  (none found)");
    }
    Ok(())
}

async fn cmd_active_context() -> Result<()> {
    let query = r#"query {
        activeContext {
            id
            kind
            name
            filePath
            startLine
            endLine
        }
    }"#;
    let data = execute_query(query).await?;
    let active = data["activeContext"]
        .as_array()
        .context("Invalid response format")?;
    println!("Active workspace context (nodes in modified files):");
    for node in active {
        println!(
            "  {} ({}) - {} [{}:{}-{}]",
            node["id"].as_str().unwrap_or(""),
            node["kind"].as_str().unwrap_or(""),
            node["name"].as_str().unwrap_or(""),
            node["filePath"].as_str().unwrap_or(""),
            node["startLine"].as_i64().unwrap_or(0),
            node["endLine"].as_i64().unwrap_or(0)
        );
    }
    if active.is_empty() {
        println!("  (no active context nodes found in modified files)");
    }
    Ok(())
}

fn cmd_embed(full: bool, model: Option<String>) -> Result<()> {
    let config = load_config()?;
    let storage = Storage::open(&config.db_path)?;

    let mut embedding_manager = EmbeddingManager::new(storage);
    embedding_manager.initialize_schema()?;

    let mut registry = ModelRegistry::from_config();
    let model_name = model.unwrap_or_else(|| registry.default_model_name().to_string());

    println!("Generating embeddings using model: {}", model_name);

    if full {
        println!("Full re-embedding requested (ignoring existing embeddings)");
    }

    let model = registry.get_model(&model_name)?;
    embedding_manager.generate_node_embeddings(model.as_ref())?;

    println!("Embedding generation completed successfully");

    Ok(())
}

fn cmd_model(command: ModelCommands) -> Result<()> {
    match command {
        ModelCommands::List => {
            let registry = ModelRegistry::from_config();
            println!("Available embedding models:");
            for model_name in registry.list_models() {
                if let Some(model_config) = registry.get_model_config(model_name) {
                    println!(
                        "  - {} (type: {}, dimension: {})",
                        model_name,
                        model_config.model_type.as_str(),
                        model_config.dimension
                    );
                }
            }
            println!("\nDefault model: {}", registry.default_model_name());
        }
        ModelCommands::Download { name } => {
            println!("Downloading model: {}", name);
            println!("Model download not yet implemented. Please download manually.");
        }
    }
    Ok(())
}

async fn cmd_ask(query: String, json: bool) -> Result<()> {
    let graphql_query = format!(
        r#"query {{
            semanticSearchEmbeddings(query: "{}", limit: 20) {{
                node {{
                    id
                    kind
                    name
                    filePath
                    startLine
                }}
                score
            }}
        }}"#,
        query.replace('"', "\\\"")
    );

    let data = execute_query(&graphql_query).await?;
    let results = data["semanticSearchEmbeddings"]
        .as_array()
        .context("Invalid response format")?;

    if json {
        println!("{}", serde_json::to_string_pretty(&results)?);
    } else {
        println!("Results for query: {}", query);
        for res in results {
            let node = &res["node"];
            println!(
                "  {} ({}) - {} (score: {:.4}) [{}:{}]",
                node["id"].as_str().unwrap_or(""),
                node["kind"].as_str().unwrap_or(""),
                node["name"].as_str().unwrap_or(""),
                res["score"].as_f64().unwrap_or(0.0),
                node["filePath"].as_str().unwrap_or(""),
                node["startLine"].as_i64().unwrap_or(0)
            );
        }
    }

    Ok(())
}

async fn cmd_graph(path: PathBuf, open: bool) -> Result<()> {
    let config = load_config()?;
    let _storage = Storage::open(&config.db_path)?;

    println!("Graph visualization for: {}", path.display());
    println!("Starting HTTP server with graph UI...\n");

    // Start the HTTP server with graph UI
    let storage = Arc::new(Mutex::new(_storage));
    let schema = spy_graph::create_schema(storage);

    use async_graphql_axum::{GraphQLRequest, GraphQLResponse};
    use axum::{
        extract::State,
        response::{Html, IntoResponse},
        routing::get,
        Router,
    };
    use tower_http::cors::CorsLayer;

    async fn graphql_handler(
        State(schema): State<spy_graph::SpySchema>,
        req: GraphQLRequest,
    ) -> GraphQLResponse {
        schema.execute(req.into_inner()).await.into()
    }

    async fn graphql_playground() -> impl IntoResponse {
        Html(
            async_graphql::http::GraphiQLSource::build()
                .endpoint("/")
                .finish(),
        )
    }

    async fn graph_ui() -> impl IntoResponse {
        Html(GRAPH_UI_HTML)
    }

    let app = Router::new()
        .route("/", get(graphql_playground).post(graphql_handler))
        .route("/graph", get(graph_ui))
        .layer(CorsLayer::permissive())
        .with_state(schema);

    let port = 4000;
    let addr = format!("127.0.0.1:{}", port);
    println!("GraphQL server listening on http://{}", addr);
    println!("Playground: http://{}/", addr);
    println!("Graph UI: http://{}/graph", addr);

    if open {
        println!("\nOpening browser...");
        if let Err(e) = opener::open(format!("http://{}/graph", addr)) {
            eprintln!("Failed to open browser: {}", e);
        }
    }

    let listener = tokio::net::TcpListener::bind(&addr).await?;
    axum::serve(listener, app).await?;

    Ok(())
}

fn cmd_install_skills(dry_run: bool, skip_index: bool, force_config: bool) -> Result<()> {
    // Get the path to the install script - try multiple locations
    let exe_path = std::env::current_exe().context("Failed to get executable path")?;

    // Try to find the script in various possible locations
    let possible_paths = vec![
        // npm installation: script is in npm/scripts/ relative to binary
        exe_path.parent()
            .and_then(|p| p.parent())
            .map(|p| p.join("scripts/install-spy-code-skill.sh")),
        // npm global installation: script might be in node_modules/spy-code/scripts/
        exe_path.parent()
            .and_then(|p| p.parent())
            .and_then(|p| p.parent())
            .map(|p| p.join("spy-code/scripts/install-spy-code-skill.sh")),
        // Development build: script is in scripts/ relative to repo root
        exe_path.parent()
            .and_then(|p| p.parent())
            .and_then(|p| p.parent())
            .map(|p| p.join("scripts/install-spy-code-skill.sh")),
        // Current directory (for running from repo root)
        Some(std::path::PathBuf::from("scripts/install-spy-code-skill.sh")),
    ];

    let script_path = possible_paths
        .into_iter()
        .flatten()
        .find(|p| p.exists())
        .ok_or_else(|| {
            anyhow::anyhow!(
                "Install script not found. Tried multiple locations. \
                 If running from npm, ensure the package was installed correctly. \
                 If running from source, ensure you're in the repository root."
            )
        })?;

    // Build the command arguments
    let mut args = vec![script_path.to_string_lossy().to_string()];

    if dry_run {
        args.push("--dry-run".to_string());
    }
    if skip_index {
        args.push("--skip-index".to_string());
    }
    if force_config {
        args.push("--force-config".to_string());
    }

    println!("Running spy-code skill installer...");
    println!("Command: bash {}", args.join(" "));
    println!();

    // Execute the script
    let status = std::process::Command::new("bash")
        .args(&args)
        .status()
        .context("Failed to execute install script")?;

    if status.success() {
        println!();
        println!("Skill installation completed successfully!");
        Ok(())
    } else {
        anyhow::bail!("Install script failed with exit code: {:?}", status.code());
    }
}
