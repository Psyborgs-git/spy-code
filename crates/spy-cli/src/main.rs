use anyhow::{Context, Result};
use clap::{Parser, Subcommand};
use spy_core::{Config, EdgeKind, NodeKind};
use spy_indexer::Indexer;
use spy_storage::Storage;
use std::path::PathBuf;
use std::sync::{Arc, Mutex};

#[derive(Parser)]
#[command(name = "spy-code")]
#[command(version = "0.1.0")]
#[command(about = "GraphQL-style compiler for codebases", long_about = None)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    Init,
    Index {
        #[arg(long)]
        full: bool,
        #[arg(long, default_value = ".")]
        path: PathBuf,
    },
    Query {
        query: String,
        #[arg(long)]
        json: bool,
    },
    Get {
        node_id: String,
    },
    Search {
        text: String,
        #[arg(long)]
        kind: Option<String>,
    },
    Callers {
        node_id: String,
        #[arg(long, default_value = "1")]
        depth: i32,
    },
    Callees {
        node_id: String,
        #[arg(long, default_value = "1")]
        depth: i32,
    },
    Changed {
        git_ref: String,
    },
    Stats,
    Serve {
        #[arg(long)]
        mcp: bool,
        #[arg(long)]
        http: bool,
        #[arg(long, default_value = "4000")]
        port: u16,
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
        Commands::Search { text, kind } => cmd_search(text, kind).await?,
        Commands::Callers { node_id, depth } => cmd_callers(node_id, depth).await?,
        Commands::Callees { node_id, depth } => cmd_callees(node_id, depth).await?,
        Commands::Changed { git_ref } => cmd_changed(git_ref).await?,
        Commands::Stats => cmd_stats().await?,
        Commands::Serve { mcp, http, port } => cmd_serve(mcp, http, port).await?,
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

async fn cmd_query(query_str: String, json: bool) -> Result<()> {
    let config = load_config()?;
    let storage = Storage::open(&config.db_path)?;
    let storage = Arc::new(Mutex::new(storage));

    let schema = spy_graph::create_schema(storage);
    let result = schema.execute(&query_str).await;

    if json {
        println!("{}", serde_json::to_string_pretty(&result)?);
    } else {
        println!("{}", serde_json::to_string_pretty(&result)?);
    }

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
        println!("  File: {}:{}:{}", node.file_path, node.start_line, node.end_line);
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

async fn cmd_search(text: String, kind: Option<String>) -> Result<()> {
    let config = load_config()?;
    let storage = Storage::open(&config.db_path)?;

    let results = storage.search_nodes(&text, 20)?;

    let results: Vec<_> = if let Some(kind_str) = kind {
        let kind = match kind_str.as_str() {
            "function" | "fn" => NodeKind::Function,
            "class" => NodeKind::Class,
            "constant" | "const" => NodeKind::Constant,
            _ => anyhow::bail!("Invalid kind: {}", kind_str),
        };
        results
            .into_iter()
            .filter(|(n, _)| n.kind == kind)
            .collect()
    } else {
        results
    };

    println!("Found {} results:", results.len());
    for (node, score) in results {
        println!("  {} ({}) - {} (score: {:.2})", node.node_id, node.kind, node.name, score);
    }

    Ok(())
}

async fn cmd_callers(node_id: String, _depth: i32) -> Result<()> {
    let config = load_config()?;
    let storage = Storage::open(&config.db_path)?;

    let edges = storage.get_incoming_edges(&node_id, EdgeKind::Calls)?;

    println!("Callers of {}:", node_id);
    for edge in edges {
        println!("  {} (confidence: {:.2})", edge.from_id, edge.confidence);
    }

    Ok(())
}

async fn cmd_callees(node_id: String, _depth: i32) -> Result<()> {
    let config = load_config()?;
    let storage = Storage::open(&config.db_path)?;

    let edges = storage.get_edges(&node_id, EdgeKind::Calls)?;

    println!("Callees of {}:", node_id);
    for edge in edges {
        println!("  {} (confidence: {:.2})", edge.to_id, edge.confidence);
    }

    Ok(())
}

async fn cmd_changed(_git_ref: String) -> Result<()> {
    println!("Changed since: Not implemented (git stub)");
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
        println!("MCP server: Not implemented (stub)");
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
            Html(async_graphql::http::playground_source(
                async_graphql::http::GraphQLPlaygroundConfig::new("/"),
            ))
        }

        let app = Router::new()
            .route("/", get(graphql_playground).post(graphql_handler))
            .layer(CorsLayer::permissive())
            .with_state(schema);

        let addr = format!("127.0.0.1:{}", port);
        println!("GraphQL server listening on http://{}", addr);
        println!("Playground: http://{}/", addr);

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
