use anyhow::Context;
use serde::{Deserialize, Serialize};
use std::fmt;
use std::path::PathBuf;
use thiserror::Error;

#[derive(Debug, Error)]
pub enum SpyError {
    #[error("Invalid node ID: {0}")]
    InvalidNodeId(String),
    #[error("Node ID too long (max 512 chars): {0}")]
    NodeIdTooLong(String),
    #[error("Parse error: {0}")]
    ParseError(String),
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),
    #[error("Other error: {0}")]
    Other(String),
}

pub type Result<T> = std::result::Result<T, SpyError>;

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct NodeId(String);

impl NodeId {
    /// Create a new NodeId from components
    /// # Errors
    /// Returns an error if the resulting ID is too long
    pub fn new(dir: &str, file: &str, class: &str, symbol: &str) -> Result<Self> {
        let dir = if dir.is_empty() { "_" } else { dir };
        let file = if file.is_empty() { "_" } else { file };
        let class = if class.is_empty() { "_" } else { class };
        let symbol = if symbol.is_empty() { "_" } else { symbol };

        let id = format!("{dir}:{file}:{class}:{symbol}");

        if id.len() > 512 {
            return Err(SpyError::NodeIdTooLong(id));
        }

        Ok(Self(id))
    }

    /// Create a NodeId from a string
    /// # Errors
    /// Returns an error if the string is invalid or too long
    pub fn from_string(s: String) -> Result<Self> {
        if s.len() > 512 {
            return Err(SpyError::NodeIdTooLong(s));
        }
        if s.split(':').count() != 4 {
            return Err(SpyError::InvalidNodeId(s));
        }
        Ok(Self(s))
    }

    #[must_use]
    pub fn as_str(&self) -> &str {
        &self.0
    }

    #[must_use]
    pub fn parts(&self) -> (&str, &str, &str, &str) {
        let mut parts = self.0.split(':');
        (
            parts.next().unwrap_or(""),
            parts.next().unwrap_or(""),
            parts.next().unwrap_or(""),
            parts.next().unwrap_or(""),
        )
    }
}

impl fmt::Display for NodeId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl From<NodeId> for String {
    fn from(id: NodeId) -> String {
        id.0
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Language {
    Rust,
    Python,
    TypeScript,
    JavaScript,
    Go,
}

impl Language {
    pub fn as_str(&self) -> &str {
        match self {
            Language::Rust => "rust",
            Language::Python => "python",
            Language::TypeScript => "typescript",
            Language::JavaScript => "javascript",
            Language::Go => "go",
        }
    }
}

impl fmt::Display for Language {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.as_str())
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum NodeKind {
    Function,
    Class,
    Constant,
    Dependency,
}

impl NodeKind {
    pub fn as_str(&self) -> &str {
        match self {
            NodeKind::Function => "function",
            NodeKind::Class => "class",
            NodeKind::Constant => "constant",
            NodeKind::Dependency => "dependency",
        }
    }
}

impl fmt::Display for NodeKind {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.as_str())
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum EdgeKind {
    Calls,
    Imports,
    References,
    InheritsFrom,
    Implements,
    DependsOn,
}

impl EdgeKind {
    pub fn as_str(&self) -> &str {
        match self {
            EdgeKind::Calls => "calls",
            EdgeKind::Imports => "imports",
            EdgeKind::References => "references",
            EdgeKind::InheritsFrom => "inherits_from",
            EdgeKind::Implements => "implements",
            EdgeKind::DependsOn => "depends_on",
        }
    }

    pub fn table_name(&self) -> &str {
        match self {
            EdgeKind::Calls => "edges_calls",
            EdgeKind::Imports => "edges_imports",
            EdgeKind::References => "edges_references",
            EdgeKind::InheritsFrom => "edges_inherits_from",
            EdgeKind::Implements => "edges_implements",
            EdgeKind::DependsOn => "edges_depends_on",
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Param {
    pub name: String,
    #[serde(rename = "type")]
    pub type_: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Signature {
    pub params: Vec<Param>,
    pub returns: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Node {
    pub node_id: NodeId,
    pub kind: NodeKind,
    pub name: String,
    pub description: Option<String>,
    pub signatures: Vec<Signature>,
    pub language: Language,
    pub file_path: String,
    pub start_line: u32,
    pub end_line: u32,
    pub content_hash: String,
    pub git_sha: Option<String>,
    pub renamed_from: Option<NodeId>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Edge {
    pub from_id: NodeId,
    pub to_id: NodeId,
    pub kind: EdgeKind,
    pub confidence: f64,
}

/// Trait for embedding model implementations
pub trait EmbeddingModel: Send + Sync {
    /// Generate embedding for a single text
    fn embed(&self, text: &str) -> anyhow::Result<Vec<f32>>;

    /// Generate embeddings for multiple texts (batch processing)
    fn embed_batch(&self, texts: &[&str]) -> anyhow::Result<Vec<Vec<f32>>> {
        texts.iter().map(|t| self.embed(t)).collect()
    }

    /// Return the dimension of the embedding vectors
    fn dimension(&self) -> usize;

    /// Return the model identifier
    fn model_name(&self) -> &str;
}

/// Model type enum for configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum ModelType {
    Local,
    Python,
    Remote,
}

impl ModelType {
    pub fn as_str(&self) -> &str {
        match self {
            ModelType::Local => "local",
            ModelType::Python => "python",
            ModelType::Remote => "remote",
        }
    }
}

pub trait Resolver: Send + Sync {
    fn language(&self) -> Language;
    fn extensions(&self) -> &[&str];
    /// Extract nodes from the file context
    /// # Errors
    /// Returns an error if parsing fails
    fn extract_nodes(&self, ctx: &FileContext) -> anyhow::Result<Vec<Node>>;
    /// Extract edges from the file context
    /// # Errors
    /// Returns an error if edge extraction fails
    fn extract_edges(&self, ctx: &FileContext, scope: &ProjectScope) -> anyhow::Result<Vec<Edge>>;
}

pub struct FileContext {
    pub tree: tree_sitter::Tree,
    pub source: Vec<u8>,
    pub path: PathBuf,
    pub language: Language,
}

pub struct ProjectScope {
    nodes: std::collections::HashMap<String, Node>,
}

impl ProjectScope {
    #[must_use]
    pub fn new() -> Self {
        Self {
            nodes: std::collections::HashMap::new(),
        }
    }

    pub fn add_node(&mut self, node: Node) {
        self.nodes.insert(node.node_id.to_string(), node);
    }

    #[must_use]
    pub fn get_node(&self, node_id: &str) -> Option<&Node> {
        self.nodes.get(node_id)
    }

    #[must_use]
    pub fn find_nodes_by_name(&self, name: &str) -> Vec<&Node> {
        self.nodes.values().filter(|n| n.name == name).collect()
    }

    pub fn all_nodes(&self) -> impl Iterator<Item = &Node> {
        self.nodes.values()
    }

    pub fn remove_nodes_for_file(&mut self, file_path: &str) {
        self.nodes.retain(|_, node| node.file_path != file_path);
    }
}

impl Default for ProjectScope {
    fn default() -> Self {
        Self::new()
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Config {
    #[serde(default = "default_version")]
    pub version: u32,
    #[serde(default = "default_db_path")]
    pub db_path: String,
    #[serde(default)]
    pub languages: LanguagesConfig,
    #[serde(default)]
    pub git: GitConfig,
    #[serde(default)]
    pub indexing: IndexingConfig,
    #[serde(default)]
    pub search: SearchConfig,
}

const fn default_version() -> u32 {
    1
}
fn default_db_path() -> String {
    ".spy-code/graph.db".to_string()
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
#[serde(deny_unknown_fields)]
pub struct LanguagesConfig {
    #[serde(default)]
    pub rust: Option<LanguageConfig>,
    #[serde(default)]
    pub python: Option<LanguageConfig>,
    #[serde(default)]
    pub typescript: Option<LanguageConfig>,
    #[serde(default)]
    pub go: Option<LanguageConfig>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct LanguageConfig {
    #[serde(default = "default_enabled")]
    pub enabled: bool,
    #[serde(default = "default_roots")]
    pub roots: Vec<String>,
    #[serde(default)]
    pub ignore: Vec<String>,
    #[serde(default = "default_resolver")]
    pub resolver: String,
    #[serde(default)]
    pub tsconfig: Option<String>,
}

const fn default_enabled() -> bool {
    true
}
fn default_roots() -> Vec<String> {
    vec!["./".to_string()]
}
fn default_resolver() -> String {
    "builtin".to_string()
}

impl Default for LanguageConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            roots: vec!["./".to_string()],
            ignore: vec![],
            resolver: "builtin".to_string(),
            tsconfig: None,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct GitConfig {
    #[serde(default = "default_enabled")]
    pub enabled: bool,
    #[serde(default = "default_enabled")]
    pub track_renames: bool,
    #[serde(default)]
    pub follow_symlinks: bool,
}

impl Default for GitConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            track_renames: true,
            follow_symlinks: false,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct IndexingConfig {
    #[serde(default = "default_max_file_size")]
    pub max_file_size_kb: u64,
    #[serde(default = "default_parallelism")]
    pub parallelism: ParallelismConfig,
    #[serde(default)]
    pub fail_fast: bool,
}

const fn default_max_file_size() -> u64 {
    2048
}
const fn default_parallelism() -> ParallelismConfig {
    ParallelismConfig::Auto
}

impl Default for IndexingConfig {
    fn default() -> Self {
        Self {
            max_file_size_kb: 2048,
            parallelism: ParallelismConfig::Auto,
            fail_fast: false,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(untagged)]
pub enum ParallelismConfig {
    Auto,
    Threads(usize),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct SearchConfig {
    #[serde(default = "default_tokenizer")]
    pub fts_tokenizer: String,
}

fn default_tokenizer() -> String {
    "unicode61".to_string()
}

impl Default for SearchConfig {
    fn default() -> Self {
        Self {
            fts_tokenizer: "unicode61".to_string(),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct EmbeddingConfig {
    #[serde(default = "default_embedding_version")]
    pub version: u32,
    #[serde(default = "default_embedding_model")]
    pub default_model: String,
    #[serde(default)]
    pub models: std::collections::HashMap<String, ModelConfig>,
}

const fn default_embedding_version() -> u32 {
    1
}

fn default_embedding_model() -> String {
    "simple-tfidf".to_string()
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ModelConfig {
    #[serde(rename = "type")]
    pub model_type: ModelType,
    #[serde(default)]
    pub implementation: Option<String>,
    #[serde(default)]
    pub model_path: Option<String>,
    #[serde(default)]
    pub download_url: Option<String>,
    pub dimension: usize,
    #[serde(default)]
    pub provider: Option<String>,
    #[serde(default)]
    pub api_key_env: Option<String>,
}

impl Default for EmbeddingConfig {
    fn default() -> Self {
        let mut models = std::collections::HashMap::new();
        models.insert(
            "simple-tfidf".to_string(),
            ModelConfig {
                model_type: ModelType::Local,
                implementation: Some("tfidf".to_string()),
                model_path: None,
                download_url: None,
                dimension: 100,
                provider: None,
                api_key_env: None,
            },
        );

        Self {
            version: 1,
            default_model: "simple-tfidf".to_string(),
            models,
        }
    }
}

impl EmbeddingConfig {
    pub fn load_from_file(path: &std::path::Path) -> anyhow::Result<Self> {
        let config_str = std::fs::read_to_string(path)
            .with_context(|| format!("Failed to read embedding config from {}", path.display()))?;
        let config: EmbeddingConfig = serde_json::from_str(&config_str)
            .with_context(|| format!("Failed to parse embedding config from {}", path.display()))?;
        Ok(config)
    }

    pub fn load_or_default() -> Self {
        let path = std::path::Path::new(".spy-code/embedding.config.json");
        if path.exists() {
            Self::load_from_file(path).unwrap_or_default()
        } else {
            Self::default()
        }
    }
}

impl Default for Config {
    fn default() -> Self {
        Self {
            version: 1,
            db_path: ".spy-code/graph.db".to_string(),
            languages: LanguagesConfig::default(),
            git: GitConfig::default(),
            indexing: IndexingConfig::default(),
            search: SearchConfig::default(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_node_id_format() {
        let id = NodeId::new("src", "lib.rs", "_", "parse").unwrap();
        assert_eq!(id.as_str(), "src:lib.rs:_:parse");
    }

    #[test]
    fn test_node_id_empty_to_underscore() {
        let id = NodeId::new("", "lib.rs", "", "parse").unwrap();
        assert_eq!(id.as_str(), "_:lib.rs:_:parse");
    }

    #[test]
    fn test_node_id_max_length() {
        let long_name = "a".repeat(600);
        let result = NodeId::new(&long_name, "b", "c", "d");
        assert!(result.is_err());
    }

    #[test]
    fn test_node_id_parts() {
        let id = NodeId::new("src/foo", "bar.rs", "Baz", "qux").unwrap();
        let (dir, file, class, symbol) = id.parts();
        assert_eq!(dir, "src/foo");
        assert_eq!(file, "bar.rs");
        assert_eq!(class, "Baz");
        assert_eq!(symbol, "qux");
    }
}
