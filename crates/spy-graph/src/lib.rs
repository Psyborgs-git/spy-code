use async_graphql::{Context, EmptyMutation, EmptySubscription, Object, Schema, SimpleObject};
use spy_core::{EdgeKind, NodeKind};
use spy_storage::Storage;
use std::sync::{Arc, Mutex};

pub type SpySchema = Schema<QueryRoot, EmptyMutation, EmptySubscription>;

pub fn create_schema(storage: Arc<Mutex<Storage>>) -> SpySchema {
    Schema::build(QueryRoot, EmptyMutation, EmptySubscription)
        .data(storage)
        .finish()
}

pub struct QueryRoot;

#[Object]
impl QueryRoot {
    async fn node(&self, ctx: &Context<'_>, id: String) -> async_graphql::Result<Option<Node>> {
        let storage = ctx.data::<Arc<Mutex<Storage>>>()?;
        let storage = storage.lock().unwrap();

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
        let storage = ctx.data::<Arc<Mutex<Storage>>>()?;
        let storage = storage.lock().unwrap();

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
        let storage = ctx.data::<Arc<Mutex<Storage>>>()?;
        let storage = storage.lock().unwrap();

        let depth = depth.unwrap_or(1).max(1) as usize;
        let edges = collect_incoming_edges(&storage, &id, EdgeKind::Calls, depth)?;
        Ok(edges.into_iter().map(|e| e.into()).collect())
    }

    async fn callees(
        &self,
        ctx: &Context<'_>,
        id: String,
        depth: Option<i32>,
    ) -> async_graphql::Result<Vec<Edge>> {
        let storage = ctx.data::<Arc<Mutex<Storage>>>()?;
        let storage = storage.lock().unwrap();

        let depth = depth.unwrap_or(1).max(1) as usize;
        let edges = collect_outgoing_edges(&storage, &id, EdgeKind::Calls, depth)?;
        Ok(edges.into_iter().map(|e| e.into()).collect())
    }

    async fn stats(&self, ctx: &Context<'_>) -> async_graphql::Result<IndexStatsGQL> {
        let storage = ctx.data::<Arc<Mutex<Storage>>>()?;
        let storage = storage.lock().unwrap();

        let stats = storage.get_stats()?;
        Ok(IndexStatsGQL {
            node_count: stats.node_count as i32,
            edge_count: stats.edge_count as i32,
            file_count: stats.file_count as i32,
            last_indexed: None,
            last_git_sha: stats.last_git_sha,
        })
    }

    async fn files(&self, ctx: &Context<'_>) -> async_graphql::Result<Vec<String>> {
        let storage = ctx.data::<Arc<Mutex<Storage>>>()?;
        let storage = storage.lock().unwrap();
        Ok(storage.list_files()?)
    }

    async fn changed_since(
        &self,
        ctx: &Context<'_>,
        #[graphql(name = "ref")] git_ref: String,
    ) -> async_graphql::Result<Vec<Node>> {
        let storage = ctx.data::<Arc<Mutex<Storage>>>()?;
        let storage = storage.lock().unwrap();

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
}

// ---------------------------------------------------------------------------
// Multi-hop BFS helpers
// ---------------------------------------------------------------------------

fn collect_outgoing_edges(
    storage: &Storage,
    start_id: &str,
    kind: EdgeKind,
    depth: usize,
) -> anyhow::Result<Vec<spy_core::Edge>> {
    let mut all_edges = Vec::new();
    let mut frontier = vec![start_id.to_string()];
    let mut visited = std::collections::HashSet::new();
    visited.insert(start_id.to_string());

    for _ in 0..depth {
        let mut next_frontier = Vec::new();
        for node_id in &frontier {
            let edges = storage.get_edges(node_id, kind)?;
            for e in edges {
                let to = e.to_id.to_string();
                if visited.insert(to.clone()) {
                    next_frontier.push(to);
                }
                all_edges.push(e);
            }
        }
        if next_frontier.is_empty() {
            break;
        }
        frontier = next_frontier;
    }

    Ok(all_edges)
}

fn collect_incoming_edges(
    storage: &Storage,
    start_id: &str,
    kind: EdgeKind,
    depth: usize,
) -> anyhow::Result<Vec<spy_core::Edge>> {
    let mut all_edges = Vec::new();
    let mut frontier = vec![start_id.to_string()];
    let mut visited = std::collections::HashSet::new();
    visited.insert(start_id.to_string());

    for _ in 0..depth {
        let mut next_frontier = Vec::new();
        for node_id in &frontier {
            let edges = storage.get_incoming_edges(node_id, kind)?;
            for e in edges {
                let from = e.from_id.to_string();
                if visited.insert(from.clone()) {
                    next_frontier.push(from);
                }
                all_edges.push(e);
            }
        }
        if next_frontier.is_empty() {
            break;
        }
        frontier = next_frontier;
    }

    Ok(all_edges)
}

// ---------------------------------------------------------------------------
// Helpers
// ---------------------------------------------------------------------------

fn matches_kind(node_kind: &NodeKind, gql_kind: &NodeKindGQL) -> bool {
    match (node_kind, gql_kind) {
        (NodeKind::Function, NodeKindGQL::Function) => true,
        (NodeKind::Class, NodeKindGQL::Class) => true,
        (NodeKind::Constant, NodeKindGQL::Constant) => true,
        _ => false,
    }
}

// ---------------------------------------------------------------------------
// GQL enums
// ---------------------------------------------------------------------------

#[derive(async_graphql::Enum, Copy, Clone, Eq, PartialEq)]
pub enum NodeKindGQL {
    Function,
    Class,
    Constant,
}

#[derive(async_graphql::Enum, Copy, Clone, Eq, PartialEq)]
pub enum LanguageGQL {
    Rust,
    Python,
    TypeScript,
    JavaScript,
    Go,
}

impl From<spy_core::Language> for LanguageGQL {
    fn from(lang: spy_core::Language) -> Self {
        match lang {
            spy_core::Language::Rust => LanguageGQL::Rust,
            spy_core::Language::Python => LanguageGQL::Python,
            spy_core::Language::TypeScript => LanguageGQL::TypeScript,
            spy_core::Language::JavaScript => LanguageGQL::JavaScript,
            spy_core::Language::Go => LanguageGQL::Go,
        }
    }
}

#[derive(async_graphql::Enum, Copy, Clone, Eq, PartialEq)]
pub enum EdgeKindGQL {
    Calls,
    Imports,
    References,
}

impl From<spy_core::EdgeKind> for EdgeKindGQL {
    fn from(kind: spy_core::EdgeKind) -> Self {
        match kind {
            spy_core::EdgeKind::Calls => EdgeKindGQL::Calls,
            spy_core::EdgeKind::Imports => EdgeKindGQL::Imports,
            spy_core::EdgeKind::References => EdgeKindGQL::References,
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

#[derive(SimpleObject)]
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

impl From<spy_core::Node> for Node {
    fn from(n: spy_core::Node) -> Self {
        let kind = match n.kind {
            NodeKind::Function => NodeKindGQL::Function,
            NodeKind::Class => NodeKindGQL::Class,
            NodeKind::Constant => NodeKindGQL::Constant,
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

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::Value;
    use spy_core::{Edge as CoreEdge, Language, Node as CoreNode, NodeId, Signature};

    fn sample_node(file_path: &str, symbol: &str, name: &str) -> anyhow::Result<CoreNode> {
        Ok(CoreNode {
            node_id: NodeId::new("src", file_path, "_", symbol)?,
            kind: spy_core::NodeKind::Function,
            name: name.to_string(),
            description: Some(format!("Description for {}", name)),
            signatures: vec![Signature {
                params: vec![],
                returns: Some("()".to_string()),
            }],
            language: Language::Rust,
            file_path: format!("src/{}", file_path),
            start_line: 1,
            end_line: 5,
            content_hash: format!("hash-{}", symbol),
            git_sha: None,
            renamed_from: None,
        })
    }

    fn sample_edge(from: &CoreNode, to: &CoreNode) -> CoreEdge {
        CoreEdge {
            from_id: from.node_id.clone(),
            to_id: to.node_id.clone(),
            kind: EdgeKind::Calls,
            confidence: 1.0,
        }
    }

    #[test]
    fn test_collect_outgoing_edges_respects_depth_and_cycles() -> anyhow::Result<()> {
        let mut storage = Storage::open_in_memory()?;
        let node_a = sample_node("a.rs", "a", "alpha")?;
        let node_b = sample_node("b.rs", "b", "beta")?;
        let node_c = sample_node("c.rs", "c", "gamma")?;

        for node in [&node_a, &node_b, &node_c] {
            storage.upsert_node(node)?;
        }
        for edge in [
            sample_edge(&node_a, &node_b),
            sample_edge(&node_b, &node_c),
            sample_edge(&node_c, &node_a),
        ] {
            storage.upsert_edge(&edge)?;
        }

        let depth_one = collect_outgoing_edges(&storage, node_a.node_id.as_str(), EdgeKind::Calls, 1)?;
        assert_eq!(depth_one.len(), 1);
        assert_eq!(depth_one[0].to_id, node_b.node_id);

        let depth_two = collect_outgoing_edges(&storage, node_a.node_id.as_str(), EdgeKind::Calls, 2)?;
        assert_eq!(depth_two.len(), 2);
        assert_eq!(depth_two[1].to_id, node_c.node_id);

        let depth_three = collect_outgoing_edges(&storage, node_a.node_id.as_str(), EdgeKind::Calls, 3)?;
        assert_eq!(depth_three.len(), 3);
        assert_eq!(depth_three[2].to_id, node_a.node_id);

        Ok(())
    }

    #[test]
    fn test_collect_incoming_edges_respects_depth() -> anyhow::Result<()> {
        let mut storage = Storage::open_in_memory()?;
        let node_a = sample_node("a.rs", "a", "alpha")?;
        let node_b = sample_node("b.rs", "b", "beta")?;
        let node_c = sample_node("c.rs", "c", "gamma")?;

        for node in [&node_a, &node_b, &node_c] {
            storage.upsert_node(node)?;
        }
        for edge in [
            sample_edge(&node_a, &node_b),
            sample_edge(&node_b, &node_c),
        ] {
            storage.upsert_edge(&edge)?;
        }

        let incoming = collect_incoming_edges(&storage, node_c.node_id.as_str(), EdgeKind::Calls, 2)?;
        assert_eq!(incoming.len(), 2);
        assert_eq!(incoming[0].from_id, node_b.node_id);
        assert_eq!(incoming[1].from_id, node_a.node_id);

        Ok(())
    }

    #[tokio::test]
    async fn test_schema_queries_return_expected_data() -> anyhow::Result<()> {
        let mut storage = Storage::open_in_memory()?;
        let node_a = sample_node("a.rs", "a", "auth_user")?;
        let node_b = sample_node("b.rs", "b", "authorize_user")?;

        storage.upsert_node(&node_a)?;
        storage.upsert_node(&node_b)?;
        storage.upsert_edge(&sample_edge(&node_a, &node_b))?;
        storage.upsert_file(&spy_storage::FileRecord {
            path: "src/a.rs".to_string(),
            language: "rust".to_string(),
            content_hash: "hash-a".to_string(),
            last_indexed: 1,
            git_sha: None,
        })?;
        storage.set_meta("last_git_sha", "sha123")?;

        let schema = create_schema(Arc::new(Mutex::new(storage)));
        let response = schema
            .execute(
                r#"
                {
                  node(id: "src:a.rs:_:a") { id name filePath }
                  files
                  stats { nodeCount edgeCount fileCount lastGitSha }
                  callees(id: "src:a.rs:_:a", depth: 2) { fromId toId confidence }
                }
                "#,
            )
            .await;

        assert!(response.errors.is_empty());
        let data = response.data.into_json()?;

        assert_eq!(data.pointer("/node/name"), Some(&Value::String("auth_user".to_string())));
        assert_eq!(data.pointer("/files/0"), Some(&Value::String("src/a.rs".to_string())));
        assert_eq!(data.pointer("/stats/nodeCount"), Some(&Value::from(2)));
        assert_eq!(data.pointer("/stats/edgeCount"), Some(&Value::from(1)));
        assert_eq!(data.pointer("/stats/fileCount"), Some(&Value::from(1)));
        assert_eq!(data.pointer("/stats/lastGitSha"), Some(&Value::String("sha123".to_string())));
        assert_eq!(data.pointer("/callees/0/toId"), Some(&Value::String("src:b.rs:_:b".to_string())));

        Ok(())
    }
}

#[derive(SimpleObject)]
pub struct IndexStatsGQL {
    node_count: i32,
    edge_count: i32,
    file_count: i32,
    last_indexed: Option<String>,
    last_git_sha: Option<String>,
}
