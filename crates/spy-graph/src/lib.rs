use async_graphql::{Context, EmptyMutation, EmptySubscription, Object, Schema, SimpleObject};
use spy_core::{EdgeKind, Language, NodeKind};
use spy_storage::Storage;
use std::path::PathBuf;
use std::sync::{Arc, Mutex};
mod diff;
mod tfidf;

pub type SpySchema = Schema<QueryRoot, EmptyMutation, EmptySubscription>;

pub struct GraphState {
    pub storage: Arc<Mutex<Storage>>,
    pub tfidf: tokio::sync::OnceCell<Arc<tfidf::TfIdfIndex>>,
}

pub fn create_schema(storage: Arc<Mutex<Storage>>) -> SpySchema {
    let state = Arc::new(GraphState {
        storage,
        tfidf: tokio::sync::OnceCell::new(),
    });
    Schema::build(QueryRoot, EmptyMutation, EmptySubscription)
        .data(state)
        .finish()
}

pub struct QueryRoot;

#[Object]
impl QueryRoot {
    async fn node(&self, ctx: &Context<'_>, id: String) -> async_graphql::Result<Option<Node>> {
        let state = ctx.data::<Arc<GraphState>>()?;
        let storage = state.storage.lock().unwrap();

        let node = storage.get_node(&id)?;
        Ok(node.map(|n| n.into()))
    }

    async fn search(
        &self,
        ctx: &Context<'_>,
        query: String,
        kind: Option<NodeKindGQL>,
        limit: Option<i32>,
    ) -> async_graphql::Result<Vec<SearchResult>> {
        let state = ctx.data::<Arc<GraphState>>()?;
        let storage = state.storage.lock().unwrap();

        let limit = limit.unwrap_or(20) as usize;
        let results = storage.search_nodes(&query, limit)?;

        Ok(results
            .into_iter()
            .filter(|(node, _)| {
                if let Some(ref k) = kind {
                    matches_kind(&node.kind, k)
                } else {
                    true
                }
            })
            .map(|(node, score)| SearchResult {
                node: node.into(),
                score,
            })
            .collect())
    }

    async fn callers(
        &self,
        ctx: &Context<'_>,
        id: String,
        depth: Option<i32>,
    ) -> async_graphql::Result<Vec<Edge>> {
        let state = ctx.data::<Arc<GraphState>>()?;
        let storage = state.storage.lock().unwrap();

        let depth = depth.unwrap_or(1).max(1);
        let edges = storage.get_incoming_edges_transitive(&id, EdgeKind::Calls, depth)?;
        Ok(edges.into_iter().map(|e| e.into()).collect())
    }

    async fn callees(
        &self,
        ctx: &Context<'_>,
        id: String,
        depth: Option<i32>,
    ) -> async_graphql::Result<Vec<Edge>> {
        let state = ctx.data::<Arc<GraphState>>()?;
        let storage = state.storage.lock().unwrap();

        let depth = depth.unwrap_or(1).max(1);
        let edges = storage.get_edges_transitive(&id, EdgeKind::Calls, depth)?;
        Ok(edges.into_iter().map(|e| e.into()).collect())
    }

    async fn stats(&self, ctx: &Context<'_>) -> async_graphql::Result<IndexStatsGQL> {
        let state = ctx.data::<Arc<GraphState>>()?;
        let storage = state.storage.lock().unwrap();

        let stats = storage.get_stats()?;
        Ok(IndexStatsGQL {
            node_count: stats.node_count as i32,
            edge_count: stats.edge_count as i32,
            file_count: stats.file_count as i32,
            last_indexed: None,
            last_git_sha: stats.last_git_sha,
        })
    }

    #[graphql(name = "activeContext")]
    async fn active_context(&self, ctx: &Context<'_>) -> async_graphql::Result<Vec<Node>> {
        let state = ctx.data::<Arc<GraphState>>()?;
        let storage = state.storage.lock().unwrap();

        let repo_root = std::env::current_dir().unwrap_or_else(|_| PathBuf::from("."));
        let mut active_nodes = Vec::new();

        if let Ok(Some(repo)) = spy_git::GitRepo::discover(&repo_root) {
            if let Ok(files) = repo.get_active_files() {
                for file_path in files {
                    let rel_path = file_path
                        .strip_prefix(repo.workdir())
                        .unwrap_or(&file_path)
                        .to_string_lossy()
                        .to_string();
                    if let Ok(nodes) = storage.get_all_nodes() {
                        for node in nodes {
                            if node.file_path == rel_path {
                                active_nodes.push(node.into());
                            }
                        }
                    }
                }
            }
        }

        Ok(active_nodes)
    }

    #[graphql(name = "semanticSearch")]
    async fn semantic_search(
        &self,
        ctx: &Context<'_>,
        query: String,
        limit: Option<i32>,
    ) -> async_graphql::Result<Vec<SearchResult>> {
        let state = ctx.data::<Arc<GraphState>>()?;
        let limit = limit.unwrap_or(20) as usize;

        let index = state
            .tfidf
            .get_or_try_init(|| async {
                let nodes = {
                    let storage = state.storage.lock().unwrap();
                    storage
                        .get_all_nodes()
                        .map_err(|e| async_graphql::Error::new(e.to_string()))?
                };
                Ok::<_, async_graphql::Error>(Arc::new(tfidf::TfIdfIndex::build(nodes)))
            })
            .await?;

        let results = index.search(&query, limit);
        Ok(results
            .into_iter()
            .map(|(n, score)| SearchResult {
                node: n.into(),
                score,
            })
            .collect())
    }

    #[graphql(name = "hybridSearch")]
    async fn hybrid_search(
        &self,
        ctx: &Context<'_>,
        query: String,
        limit: Option<i32>,
    ) -> async_graphql::Result<Vec<SearchResult>> {
        let state = ctx.data::<Arc<GraphState>>()?;
        let limit = limit.unwrap_or(20) as usize;

        let fts5_results = {
            let storage = state.storage.lock().unwrap();
            storage
                .search_nodes(&query, limit * 2)
                .map_err(|e| async_graphql::Error::new(e.to_string()))?
        };

        let index = state
            .tfidf
            .get_or_try_init(|| async {
                let nodes = {
                    let storage = state.storage.lock().unwrap();
                    storage
                        .get_all_nodes()
                        .map_err(|e| async_graphql::Error::new(e.to_string()))?
                };
                Ok::<_, async_graphql::Error>(Arc::new(tfidf::TfIdfIndex::build(nodes)))
            })
            .await?;

        let tfidf_results = index.search(&query, limit * 2);

        use std::collections::HashMap;
        let mut rrf_scores: HashMap<String, (spy_core::Node, f64)> = HashMap::new();

        for (rank, (node, _)) in fts5_results.into_iter().enumerate() {
            let id = node.node_id.to_string();
            let score = 1.0 / (60.0 + rank as f64);
            let entry = rrf_scores.entry(id).or_insert((node, 0.0));
            entry.1 += score;
        }

        for (rank, (node, _)) in tfidf_results.into_iter().enumerate() {
            let id = node.node_id.to_string();
            let score = 1.0 / (60.0 + rank as f64);
            let entry = rrf_scores.entry(id).or_insert((node, 0.0));
            entry.1 += score;
        }

        let mut combined: Vec<(spy_core::Node, f64)> = rrf_scores.into_values().collect();
        combined.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap_or(std::cmp::Ordering::Equal));
        combined.truncate(limit);

        Ok(combined
            .into_iter()
            .map(|(n, score)| SearchResult {
                node: n.into(),
                score,
            })
            .collect())
    }

    #[graphql(name = "diffSymbols")]
    async fn diff_symbols(
        &self,
        ctx: &Context<'_>,
        from_ref: String,
        to_ref: String,
    ) -> async_graphql::Result<Vec<diff::SignatureDiff>> {
        use spy_core::Language;
        use spy_git::GitRepo;
        use std::path::Path;

        let _state = ctx.data::<Arc<GraphState>>()?;

        let repo = GitRepo::discover(Path::new("."))
            .map_err(|e| async_graphql::Error::new(e.to_string()))?
            .ok_or_else(|| async_graphql::Error::new("Not in a git repository"))?;

        // 1. Get changed files
        let diffs = repo
            .diff_files_since(&from_ref)
            .map_err(|e| async_graphql::Error::new(e.to_string()))?;

        let mut all_diffs = Vec::new();

        for file_diff in diffs {
            let path = file_diff.path.clone();

            // Detect language
            let ext = path.extension().and_then(|e| e.to_str()).unwrap_or("");
            let lang = match ext {
                "rs" => Language::Rust,
                "py" => Language::Python,
                "ts" | "tsx" | "js" | "jsx" => Language::TypeScript,
                "go" => Language::Go,
                _ => continue,
            };

            // Get source code
            let old_src = repo.cat_file_at_ref(&from_ref, &path).unwrap_or_default();
            let new_src = repo.cat_file_at_ref(&to_ref, &path).unwrap_or_default();

            if old_src.is_empty() && new_src.is_empty() {
                continue;
            }

            if let Ok(file_diffs) =
                diff::compute_diff(&path, lang, old_src.as_bytes(), new_src.as_bytes())
            {
                all_diffs.extend(file_diffs);
            }
        }

        Ok(all_diffs)
    }

    #[graphql(name = "callPath")]
    async fn call_path(
        &self,
        ctx: &Context<'_>,
        source_node_id: String,
        target_node_id: String,
    ) -> async_graphql::Result<Vec<Edge>> {
        let state = ctx.data::<Arc<GraphState>>()?;
        let storage = state.storage.lock().unwrap();

        let path = storage
            .find_shortest_path(&source_node_id, &target_node_id, spy_core::EdgeKind::Calls)
            .map_err(|e| async_graphql::Error::new(e.to_string()))?;
        Ok(path.into_iter().map(|e| e.into()).collect())
    }

    #[graphql(name = "impactedSymbols")]
    async fn impacted_symbols(
        &self,
        ctx: &Context<'_>,
        node_id: String,
        max_depth: Option<i32>,
    ) -> async_graphql::Result<Vec<Node>> {
        let depth = max_depth.unwrap_or(10);
        let state = ctx.data::<Arc<GraphState>>()?;
        let storage = state.storage.lock().unwrap();

        let edges = storage
            .get_incoming_edges_transitive(&node_id, spy_core::EdgeKind::Calls, depth)
            .map_err(|e| async_graphql::Error::new(e.to_string()))?;

        let mut nodes: Vec<Node> = Vec::new();
        // Since we want impacted symbols, we look at the 'from' side of the incoming calls
        for edge in edges {
            if let Ok(Some(node)) = storage.get_node(edge.from_id.as_str()) {
                nodes.push(node.into());
            }
        }

        // Remove duplicates since multiple call paths could go through the same node
        let mut unique_nodes = std::collections::HashMap::new();
        for n in nodes {
            unique_nodes.insert(n.id.clone(), n);
        }

        Ok(unique_nodes.into_values().collect())
    }

    #[graphql(name = "projectDependencies")]
    async fn project_dependencies(&self, ctx: &Context<'_>) -> async_graphql::Result<Vec<Node>> {
        let state = ctx.data::<Arc<GraphState>>()?;
        let storage = state.storage.lock().unwrap();

        // Return all Dependency nodes
        let nodes = storage
            .get_all_nodes()
            .map_err(|e| async_graphql::Error::new(e.to_string()))?;
        Ok(nodes
            .into_iter()
            .filter(|n| n.kind == spy_core::NodeKind::Dependency)
            .map(|n| n.into())
            .collect())
    }

    async fn files(&self, ctx: &Context<'_>) -> async_graphql::Result<Vec<String>> {
        let state = ctx.data::<Arc<GraphState>>()?;
        let storage = state.storage.lock().unwrap();
        Ok(storage.list_files()?)
    }

    async fn changed_since(
        &self,
        ctx: &Context<'_>,
        #[graphql(name = "ref")] git_ref: String,
    ) -> async_graphql::Result<Vec<Node>> {
        let state = ctx.data::<Arc<GraphState>>()?;
        let storage = state.storage.lock().unwrap();

        // Resolve changed file paths using git
        let changed_paths = spy_git::GitRepo::discover(std::path::Path::new("."))
            .map_err(|e| async_graphql::Error::new(e.to_string()))?
            .map(|repo| repo.files_changed_since_ref(&git_ref))
            .transpose()
            .map_err(|e| async_graphql::Error::new(e.to_string()))?
            .unwrap_or_default();

        if changed_paths.is_empty() {
            return Ok(vec![]);
        }

        let path_strings: Vec<String> = changed_paths
            .into_iter()
            .map(|p| p.to_string_lossy().into_owned())
            .collect();

        let nodes = storage.get_nodes_for_files(&path_strings)?;
        Ok(nodes.into_iter().map(Into::into).collect())
    }

    #[graphql(name = "fileSkeleton")]
    async fn file_skeleton(
        &self,
        ctx: &Context<'_>,
        file_path: String,
    ) -> async_graphql::Result<String> {
        let state = ctx.data::<Arc<GraphState>>()?;
        let storage = state.storage.lock().unwrap();

        // 1. Fetch nodes for this file from storage
        let nodes = storage.get_nodes_for_files(std::slice::from_ref(&file_path))?;

        // 2. Read the file contents from disk
        let file_content = std::fs::read_to_string(&file_path)
            .map_err(|e| async_graphql::Error::new(format!("Failed to read file: {}", e)))?;

        // 3. Find the language from the nodes, or fallback to file extension
        let language = if let Some(node) = nodes.first() {
            node.language
        } else {
            let path = std::path::Path::new(&file_path);
            match path.extension().and_then(|ext| ext.to_str()) {
                Some("rs") => Language::Rust,
                Some("py") => Language::Python,
                Some("ts") | Some("tsx") => Language::TypeScript,
                Some("js") | Some("jsx") => Language::JavaScript,
                Some("go") => Language::Go,
                _ => return Err(async_graphql::Error::new("Unsupported file language")),
            }
        };

        // 4. Filter only Function nodes
        let functions: Vec<spy_core::Node> = nodes
            .into_iter()
            .filter(|n| n.kind == spy_core::NodeKind::Function)
            .collect();

        // 5. Generate the skeleton
        let skeleton = skeletonize(&file_content, language, &functions);

        Ok(skeleton)
    }

    #[graphql(name = "semanticSearchEmbeddings")]
    async fn semantic_search_embeddings(
        &self,
        ctx: &Context<'_>,
        query: String,
        limit: Option<i32>,
    ) -> async_graphql::Result<Vec<SearchResult>> {
        let state = ctx.data::<Arc<GraphState>>()?;
        let storage = state.storage.lock().unwrap();
        let limit = limit.unwrap_or(20) as usize;

        // Get model and generate query embedding
        let mut registry = spy_embeddings::ModelRegistry::from_config();
        let model = registry
            .get_default_model()
            .map_err(|e: anyhow::Error| async_graphql::Error::new(e.to_string()))?;
        let query_embedding = model
            .embed(&query)
            .map_err(|e: anyhow::Error| async_graphql::Error::new(e.to_string()))?;

        // Query embeddings with model filter
        let rows: Vec<(String, Vec<u8>)> = storage
            .query_raw(
                "SELECT node_id, embedding FROM node_embeddings WHERE embedding_model = ?1",
                &[&model.model_name()],
                |row| {
                    let node_id: String = row.get(0)?;
                    let embedding: Vec<u8> = row.get(1)?;
                    Ok((node_id, embedding))
                },
            )
            .map_err(|e: anyhow::Error| async_graphql::Error::new(e.to_string()))?;

        // Calculate similarities
        let mut results = Vec::new();
        for (node_id, embedding_bytes) in rows {
            let embedding_vec: Vec<f32> = embedding_bytes
                .chunks_exact(4)
                .map(|chunk| f32::from_le_bytes(chunk.try_into().unwrap()))
                .collect();

            let similarity = cosine_similarity(&query_embedding, &embedding_vec);

            if let Ok(Some(node)) = storage.get_node(&node_id) {
                results.push((node, similarity));
            }
        }

        results.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap());
        results.truncate(limit);

        Ok(results
            .into_iter()
            .map(|(n, score): (spy_core::Node, f64)| SearchResult {
                node: n.into(),
                score,
            })
            .collect())
    }

    #[graphql(name = "embeddingsStatus")]
    async fn embeddings_status(
        &self,
        ctx: &Context<'_>,
    ) -> async_graphql::Result<EmbeddingStatusGQL> {
        let state = ctx.data::<Arc<GraphState>>()?;
        let storage = state.storage.lock().unwrap();

        let status: Option<String> = storage
            .query_row_raw(
                "SELECT status FROM embedding_progress WHERE id = (SELECT MAX(id) FROM embedding_progress)",
                &[],
                |row| row.get(0),
            )
            .map_err(|e: anyhow::Error| async_graphql::Error::new(e.to_string()))?;

        let total_nodes: i32 = storage
            .query_row_raw(
                "SELECT total_nodes FROM embedding_progress WHERE id = (SELECT MAX(id) FROM embedding_progress)",
                &[],
                |row| row.get(0),
            )
            .map_err(|e: anyhow::Error| async_graphql::Error::new(e.to_string()))?
            .unwrap_or(0);

        let processed_nodes: i32 = storage
            .query_row_raw(
                "SELECT processed_nodes FROM embedding_progress WHERE id = (SELECT MAX(id) FROM embedding_progress)",
                &[],
                |row| row.get(0),
            )
            .map_err(|e: anyhow::Error| async_graphql::Error::new(e.to_string()))?
            .unwrap_or(0);

        let started_at: i64 = storage
            .query_row_raw(
                "SELECT started_at FROM embedding_progress WHERE id = (SELECT MAX(id) FROM embedding_progress)",
                &[],
                |row| row.get(0),
            )
            .map_err(|e: anyhow::Error| async_graphql::Error::new(e.to_string()))?
            .unwrap_or(0);

        let completed_at: Option<i64> = storage
            .query_row_raw(
                "SELECT completed_at FROM embedding_progress WHERE id = (SELECT MAX(id) FROM embedding_progress)",
                &[],
                |row| row.get(0),
            )
            .map_err(|e: anyhow::Error| async_graphql::Error::new(e.to_string()))?;

        Ok(EmbeddingStatusGQL {
            total_nodes,
            processed_nodes,
            status: status.unwrap_or_else(|| "not_started".to_string()),
            started_at,
            completed_at,
        })
    }

    #[graphql(name = "graphData")]
    async fn graph_data(
        &self,
        ctx: &Context<'_>,
        filter: Option<GraphFilterInput>,
    ) -> async_graphql::Result<GraphData> {
        let state = ctx.data::<Arc<GraphState>>()?;
        let storage = state.storage.lock().unwrap();

        let filter = filter.unwrap_or_default();
        let mut nodes = storage
            .get_all_nodes()
            .map_err(|e| async_graphql::Error::new(e.to_string()))?;

        // Apply filters
        if let Some(ref file_path) = filter.file_path {
            nodes.retain(|n| n.file_path.contains(file_path));
        }

        if let Some(ref node_kinds) = filter.node_kinds {
            let kind_set: std::collections::HashSet<NodeKindGQL> =
                node_kinds.iter().cloned().collect();
            nodes.retain(|n| {
                let gql_kind = match n.kind {
                    spy_core::NodeKind::Function => NodeKindGQL::Function,
                    spy_core::NodeKind::Class => NodeKindGQL::Class,
                    spy_core::NodeKind::Constant => NodeKindGQL::Constant,
                    spy_core::NodeKind::Dependency => NodeKindGQL::Dependency,
                };
                kind_set.contains(&gql_kind)
            });
        }

        if let Some(ref languages) = filter.languages {
            let lang_set: std::collections::HashSet<LanguageGQL> =
                languages.iter().cloned().collect();
            nodes.retain(|n| lang_set.contains(&LanguageGQL::from(n.language)));
        }

        // Convert to GraphQL nodes
        let gql_nodes: Vec<Node> = nodes.into_iter().map(Into::into).collect();

        // Get edges for filtered nodes
        let mut edges = Vec::new();
        if let Some(ref edge_kinds) = filter.edge_kinds {
            for node in &gql_nodes {
                for kind in edge_kinds {
                    let core_kind = match kind {
                        EdgeKindGQL::Calls => spy_core::EdgeKind::Calls,
                        EdgeKindGQL::Imports => spy_core::EdgeKind::Imports,
                        EdgeKindGQL::References => spy_core::EdgeKind::References,
                        EdgeKindGQL::InheritsFrom => spy_core::EdgeKind::InheritsFrom,
                        EdgeKindGQL::Implements => spy_core::EdgeKind::Implements,
                        EdgeKindGQL::DependsOn => spy_core::EdgeKind::DependsOn,
                    };
                    if let Ok(node_edges) = storage.get_edges(&node.id, core_kind) {
                        for edge in node_edges {
                            // Only include edges where both nodes are in our filtered set
                            if gql_nodes.iter().any(|n| n.id == edge.to_id.as_str()) {
                                edges.push(edge.into());
                            }
                        }
                    }
                }
            }
        }

        Ok(GraphData {
            nodes: gql_nodes,
            edges,
        })
    }
}

// ---------------------------------------------------------------------------
// Helpers
// ---------------------------------------------------------------------------

fn cosine_similarity(a: &[f32], b: &[f32]) -> f64 {
    if a.len() != b.len() || a.is_empty() {
        return 0.0;
    }

    let dot_product: f32 = a.iter().zip(b.iter()).map(|(x, y)| x * y).sum();
    let norm_a: f32 = a.iter().map(|x| x * x).sum::<f32>().sqrt();
    let norm_b: f32 = b.iter().map(|x| x * x).sum::<f32>().sqrt();

    if norm_a == 0.0 || norm_b == 0.0 {
        return 0.0;
    }

    (dot_product / (norm_a * norm_b)) as f64
}

fn matches_kind(node_kind: &NodeKind, gql_kind: &NodeKindGQL) -> bool {
    matches!(
        (node_kind, gql_kind),
        (NodeKind::Function, NodeKindGQL::Function)
            | (NodeKind::Class, NodeKindGQL::Class)
            | (NodeKind::Constant, NodeKindGQL::Constant)
    )
}

// ---------------------------------------------------------------------------
// GQL enums
// ---------------------------------------------------------------------------

#[derive(async_graphql::Enum, Copy, Clone, Eq, PartialEq, Hash)]
pub enum NodeKindGQL {
    Function,
    Class,
    Constant,
    #[graphql(name = "DEPENDENCY")]
    Dependency,
}

#[derive(async_graphql::Enum, Copy, Clone, Eq, PartialEq, Hash)]
pub enum LanguageGQL {
    Rust,
    Python,
    TypeScript,
    JavaScript,
    Go,
    Java,
}

impl From<spy_core::Language> for LanguageGQL {
    fn from(lang: spy_core::Language) -> Self {
        match lang {
            spy_core::Language::Rust => LanguageGQL::Rust,
            spy_core::Language::Python => LanguageGQL::Python,
            spy_core::Language::TypeScript => LanguageGQL::TypeScript,
            spy_core::Language::JavaScript => LanguageGQL::JavaScript,
            spy_core::Language::Go => LanguageGQL::Go,
            spy_core::Language::Java => LanguageGQL::Java,
        }
    }
}

#[derive(async_graphql::Enum, Copy, Clone, Eq, PartialEq)]
pub enum EdgeKindGQL {
    Calls,
    Imports,
    References,
    #[graphql(name = "INHERITS_FROM")]
    InheritsFrom,
    #[graphql(name = "IMPLEMENTS")]
    Implements,
    #[graphql(name = "DEPENDS_ON")]
    DependsOn,
}

impl From<spy_core::EdgeKind> for EdgeKindGQL {
    fn from(kind: spy_core::EdgeKind) -> Self {
        match kind {
            spy_core::EdgeKind::Calls => EdgeKindGQL::Calls,
            spy_core::EdgeKind::Imports => EdgeKindGQL::Imports,
            spy_core::EdgeKind::References => EdgeKindGQL::References,
            spy_core::EdgeKind::InheritsFrom => EdgeKindGQL::InheritsFrom,
            spy_core::EdgeKind::Implements => EdgeKindGQL::Implements,
            spy_core::EdgeKind::DependsOn => EdgeKindGQL::DependsOn,
        }
    }
}

// ---------------------------------------------------------------------------
// GQL object types
// ---------------------------------------------------------------------------

#[derive(SimpleObject)]
pub struct Param {
    name: String,
    #[graphql(name = "type")]
    type_: Option<String>,
}

impl From<spy_core::Param> for Param {
    fn from(p: spy_core::Param) -> Self {
        Param {
            name: p.name,
            type_: p.type_,
        }
    }
}

#[derive(SimpleObject)]
pub struct Signature {
    params: Vec<Param>,
    returns: Option<String>,
}

impl From<spy_core::Signature> for Signature {
    fn from(s: spy_core::Signature) -> Self {
        Signature {
            params: s.params.into_iter().map(Into::into).collect(),
            returns: s.returns,
        }
    }
}

pub struct Node {
    id: String,
    kind: NodeKindGQL,
    name: String,
    description: Option<String>,
    signatures: Vec<Signature>,
    language: LanguageGQL,
    file_path: String,
    start_line: i32,
    end_line: i32,
    git_sha: Option<String>,
    renamed_from: Option<String>,
}

#[Object]
impl Node {
    async fn id(&self) -> &str {
        &self.id
    }

    async fn kind(&self) -> NodeKindGQL {
        self.kind
    }

    async fn name(&self) -> &str {
        &self.name
    }

    async fn description(&self) -> Option<&str> {
        self.description.as_deref()
    }

    async fn signatures(&self) -> &[Signature] {
        &self.signatures
    }

    async fn language(&self) -> LanguageGQL {
        self.language
    }

    #[graphql(name = "filePath")]
    async fn file_path(&self) -> &str {
        &self.file_path
    }

    #[graphql(name = "startLine")]
    async fn start_line(&self) -> i32 {
        self.start_line
    }

    #[graphql(name = "endLine")]
    async fn end_line(&self) -> i32 {
        self.end_line
    }

    #[graphql(name = "gitSha")]
    async fn git_sha(&self) -> Option<&str> {
        self.git_sha.as_deref()
    }

    #[graphql(name = "renamedFrom")]
    async fn renamed_from(&self) -> Option<&str> {
        self.renamed_from.as_deref()
    }

    #[graphql(name = "sourceCode")]
    async fn source_code(&self) -> Option<String> {
        read_file_snippet(&self.file_path, self.start_line, self.end_line).ok()
    }

    async fn tests(&self, ctx: &Context<'_>) -> async_graphql::Result<Vec<Node>> {
        let state = ctx.data::<Arc<GraphState>>()?;
        let storage = state.storage.lock().unwrap();

        let all_nodes = storage
            .get_all_nodes()
            .map_err(|e| async_graphql::Error::new(e.to_string()))?;
        let mut associated_tests = Vec::new();
        let target_lower = self.name.to_lowercase();

        for node in all_nodes {
            if node.kind == spy_core::NodeKind::Function {
                let node_name_lower = node.name.to_lowercase();
                if node_name_lower.contains("test")
                    && (node_name_lower.contains(&target_lower) || node.file_path.contains("test"))
                {
                    associated_tests.push(node.into());
                }
            }
        }

        Ok(associated_tests)
    }

    async fn coverage(&self) -> Option<f64> {
        get_line_coverage(&self.file_path, self.start_line, self.end_line)
    }

    async fn bases(&self, ctx: &Context<'_>) -> async_graphql::Result<Vec<Node>> {
        let state = ctx.data::<Arc<GraphState>>()?;
        let storage = state.storage.lock().unwrap();

        let mut base_nodes = Vec::new();
        if let Ok(edges) = storage.get_edges(&self.id, spy_core::EdgeKind::InheritsFrom) {
            for edge in edges {
                if let Ok(Some(node)) = storage.get_node(edge.to_id.as_str()) {
                    base_nodes.push(node.into());
                }
            }
        }
        if let Ok(edges) = storage.get_edges(&self.id, spy_core::EdgeKind::Implements) {
            for edge in edges {
                if let Ok(Some(node)) = storage.get_node(edge.to_id.as_str()) {
                    base_nodes.push(node.into());
                }
            }
        }
        Ok(base_nodes)
    }

    async fn derived(&self, ctx: &Context<'_>) -> async_graphql::Result<Vec<Node>> {
        let state = ctx.data::<Arc<GraphState>>()?;
        let storage = state.storage.lock().unwrap();

        let mut derived_nodes = Vec::new();
        if let Ok(edges) = storage.get_incoming_edges(&self.id, spy_core::EdgeKind::InheritsFrom) {
            for edge in edges {
                if let Ok(Some(node)) = storage.get_node(edge.from_id.as_str()) {
                    derived_nodes.push(node.into());
                }
            }
        }
        if let Ok(edges) = storage.get_incoming_edges(&self.id, spy_core::EdgeKind::Implements) {
            for edge in edges {
                if let Ok(Some(node)) = storage.get_node(edge.from_id.as_str()) {
                    derived_nodes.push(node.into());
                }
            }
        }
        Ok(derived_nodes)
    }
}

fn get_line_coverage(file_path: &str, start_line: i32, end_line: i32) -> Option<f64> {
    let repo_root = std::env::current_dir().unwrap_or_else(|_| PathBuf::from("."));
    let lcov_paths = [
        repo_root.join("coverage/lcov.info"),
        repo_root.join("lcov.info"),
    ];

    for lcov_path in &lcov_paths {
        if lcov_path.exists() {
            if let Ok(content) = std::fs::read_to_string(lcov_path) {
                let mut in_file = false;
                let mut total_lines = 0;
                let mut covered_lines = 0;

                for line in content.lines() {
                    let line = line.trim();
                    if let Some(sf_path) = line.strip_prefix("SF:") {
                        in_file = sf_path.contains(file_path) || file_path.contains(sf_path);
                    } else if in_file && line.starts_with("DA:") {
                        let parts: Vec<&str> = line[3..].split(',').collect();
                        if parts.len() >= 2 {
                            if let Ok(line_num) = parts[0].parse::<i32>() {
                                if line_num >= start_line && line_num <= end_line {
                                    total_lines += 1;
                                    if let Ok(count) = parts[1].parse::<i32>() {
                                        if count > 0 {
                                            covered_lines += 1;
                                        }
                                    }
                                }
                            }
                        }
                    } else if line == "end_of_record" {
                        if in_file {
                            if total_lines > 0 {
                                return Some((covered_lines as f64) / (total_lines as f64));
                            }
                            return Some(0.0);
                        }
                        in_file = false;
                    }
                }
            }
        }
    }
    None
}

impl From<spy_core::Node> for Node {
    fn from(n: spy_core::Node) -> Self {
        let kind = match n.kind {
            NodeKind::Function => NodeKindGQL::Function,
            NodeKind::Class => NodeKindGQL::Class,
            NodeKind::Constant => NodeKindGQL::Constant,
            NodeKind::Dependency => NodeKindGQL::Dependency,
        };

        Node {
            id: n.node_id.to_string(),
            kind,
            name: n.name,
            description: n.description,
            signatures: n.signatures.into_iter().map(Into::into).collect(),
            language: n.language.into(),
            file_path: n.file_path,
            start_line: n.start_line as i32,
            end_line: n.end_line as i32,
            git_sha: n.git_sha,
            renamed_from: n.renamed_from.map(|id| id.to_string()),
        }
    }
}

#[derive(SimpleObject)]
pub struct Edge {
    from_id: String,
    to_id: String,
    kind: EdgeKindGQL,
    confidence: f64,
}

impl From<spy_core::Edge> for Edge {
    fn from(e: spy_core::Edge) -> Self {
        Edge {
            from_id: e.from_id.to_string(),
            to_id: e.to_id.to_string(),
            kind: e.kind.into(),
            confidence: e.confidence,
        }
    }
}

#[derive(SimpleObject)]
pub struct SearchResult {
    node: Node,
    score: f64,
}

#[derive(SimpleObject)]
pub struct IndexStatsGQL {
    node_count: i32,
    edge_count: i32,
    file_count: i32,
    last_indexed: Option<String>,
    last_git_sha: Option<String>,
}

#[derive(SimpleObject)]
pub struct EmbeddingStatusGQL {
    total_nodes: i32,
    processed_nodes: i32,
    status: String,
    started_at: i64,
    completed_at: Option<i64>,
}

#[derive(SimpleObject)]
pub struct GraphData {
    nodes: Vec<Node>,
    edges: Vec<Edge>,
}

#[derive(async_graphql::InputObject, Default)]
pub struct GraphFilterInput {
    file_path: Option<String>,
    node_kinds: Option<Vec<NodeKindGQL>>,
    languages: Option<Vec<LanguageGQL>>,
    edge_kinds: Option<Vec<EdgeKindGQL>>,
}

fn read_file_snippet(file_path: &str, start_line: i32, end_line: i32) -> std::io::Result<String> {
    use std::fs::File;
    use std::io::{BufRead, BufReader};

    let file = File::open(file_path)?;
    let reader = BufReader::new(file);

    let mut snippet_lines = Vec::new();
    for (idx, line) in reader.lines().enumerate() {
        let line_num = (idx + 1) as i32;
        if line_num >= start_line && line_num <= end_line {
            snippet_lines.push(line?);
        }
        if line_num > end_line {
            break;
        }
    }
    Ok(snippet_lines.join("\n"))
}

fn skeletonize(
    file_content: &str,
    language: spy_core::Language,
    functions: &[spy_core::Node],
) -> String {
    let mut lines: Vec<String> = file_content.lines().map(|s| s.to_string()).collect();

    // Sort functions by start_line descending to avoid index shifting issues
    let mut sorted_funcs = functions.to_vec();
    sorted_funcs.sort_by_key(|b| std::cmp::Reverse(b.start_line));

    for func in sorted_funcs {
        let start_idx = (func.start_line as usize).saturating_sub(1);
        let end_idx = (func.end_line as usize).saturating_sub(1);

        if start_idx >= lines.len() || end_idx >= lines.len() {
            continue;
        }

        match language {
            spy_core::Language::Python => {
                if func.end_line > func.start_line {
                    let first_line = &lines[start_idx];
                    let indent = first_line
                        .chars()
                        .take_while(|c| c.is_whitespace())
                        .collect::<String>();
                    let placeholder = format!("{}    pass", indent);
                    lines.splice((start_idx + 1)..=end_idx, std::iter::once(placeholder));
                }
            }
            _ => {
                if func.end_line > func.start_line + 1 {
                    let first_line = &lines[start_idx];
                    let indent = first_line
                        .chars()
                        .take_while(|c| c.is_whitespace())
                        .collect::<String>();
                    let placeholder = format!("{}    // ...", indent);
                    lines.splice((start_idx + 1)..end_idx, std::iter::once(placeholder));
                }
            }
        }
    }

    lines.join("\n")
}

#[cfg(test)]
mod tests {
    use super::*;
    use spy_core::{Language, Node, NodeId, NodeKind};

    #[test]
    fn test_read_file_snippet() {
        let path = "test_read_file_snippet_temp.txt";
        std::fs::write(path, "line 1\nline 2\nline 3\nline 4").unwrap();

        let snippet = read_file_snippet(path, 2, 3).unwrap();
        assert_eq!(snippet, "line 2\nline 3");

        let single = read_file_snippet(path, 1, 1).unwrap();
        assert_eq!(single, "line 1");

        let _ = std::fs::remove_file(path);
    }

    #[test]
    fn test_skeletonize_rust() {
        let content =
            "fn add(a: i32, b: i32) -> i32 {\n    let sum = a + b;\n    sum\n}\n\nfn sub() {}";
        let nodes = vec![
            Node {
                node_id: NodeId::new("src", "lib.rs", "_", "add").unwrap(),
                kind: NodeKind::Function,
                name: "add".to_string(),
                description: None,
                signatures: vec![],
                language: Language::Rust,
                file_path: "src/lib.rs".to_string(),
                start_line: 1,
                end_line: 4,
                content_hash: "".to_string(),
                git_sha: None,
                renamed_from: None,
            },
            Node {
                node_id: NodeId::new("src", "lib.rs", "_", "sub").unwrap(),
                kind: NodeKind::Function,
                name: "sub".to_string(),
                description: None,
                signatures: vec![],
                language: Language::Rust,
                file_path: "src/lib.rs".to_string(),
                start_line: 6,
                end_line: 6,
                content_hash: "".to_string(),
                git_sha: None,
                renamed_from: None,
            },
        ];

        let skeleton = skeletonize(content, Language::Rust, &nodes);
        let expected = "fn add(a: i32, b: i32) -> i32 {\n    // ...\n}\n\nfn sub() {}";
        assert_eq!(skeleton, expected);
    }

    #[test]
    fn test_skeletonize_python() {
        let content = "def hello():\n    print(\"hello\")\n    return 42";
        let nodes = vec![Node {
            node_id: NodeId::new("src", "main.py", "_", "hello").unwrap(),
            kind: NodeKind::Function,
            name: "hello".to_string(),
            description: None,
            signatures: vec![],
            language: Language::Python,
            file_path: "src/main.py".to_string(),
            start_line: 1,
            end_line: 3,
            content_hash: "".to_string(),
            git_sha: None,
            renamed_from: None,
        }];

        let skeleton = skeletonize(content, Language::Python, &nodes);
        let expected = "def hello():\n    pass";
        assert_eq!(skeleton, expected);
    }
}
