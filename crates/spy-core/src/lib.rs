use serde::de::{self, Visitor};
use serde::{Deserialize, Deserializer, Serialize, Serializer};
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
    pub fn new(dir: &str, file: &str, class: &str, symbol: &str) -> Result<Self> {
        let dir = if dir.is_empty() { "_" } else { dir };
        let file = if file.is_empty() { "_" } else { file };
        let class = if class.is_empty() { "_" } else { class };
        let symbol = if symbol.is_empty() { "_" } else { symbol };

        let id = format!("{}:{}:{}:{}", dir, file, class, symbol);

        if id.len() > 512 {
            return Err(SpyError::NodeIdTooLong(id));
        }

        Ok(NodeId(id))
    }

    pub fn from_string(s: String) -> Result<Self> {
        if s.len() > 512 {
            return Err(SpyError::NodeIdTooLong(s));
        }
        let parts: Vec<&str> = s.split(':').collect();
        if parts.len() != 4 {
            return Err(SpyError::InvalidNodeId(s));
        }
        Ok(NodeId(s))
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }

    pub fn parts(&self) -> (&str, &str, &str, &str) {
        let parts: Vec<&str> = self.0.split(':').collect();
        (parts[0], parts[1], parts[2], parts[3])
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
}

impl NodeKind {
    pub fn as_str(&self) -> &str {
        match self {
            NodeKind::Function => "function",
            NodeKind::Class => "class",
            NodeKind::Constant => "constant",
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
}

impl EdgeKind {
    pub fn as_str(&self) -> &str {
        match self {
            EdgeKind::Calls => "calls",
            EdgeKind::Imports => "imports",
            EdgeKind::References => "references",
        }
    }

    pub fn table_name(&self) -> &str {
        match self {
            EdgeKind::Calls => "edges_calls",
            EdgeKind::Imports => "edges_imports",
            EdgeKind::References => "edges_references",
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

pub trait Resolver: Send + Sync {
    fn language(&self) -> Language;
    fn extensions(&self) -> &[&str];
    fn extract_nodes(&self, ctx: &FileContext) -> anyhow::Result<Vec<Node>>;
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
    pub fn new() -> Self {
        ProjectScope {
            nodes: std::collections::HashMap::new(),
        }
    }

    pub fn add_node(&mut self, node: Node) {
        self.nodes.insert(node.node_id.to_string(), node);
    }

    pub fn get_node(&self, node_id: &str) -> Option<&Node> {
        self.nodes.get(node_id)
    }

    pub fn find_nodes_by_name(&self, name: &str) -> Vec<&Node> {
        self.nodes.values().filter(|n| n.name == name).collect()
    }

    pub fn all_nodes(&self) -> impl Iterator<Item = &Node> {
        self.nodes.values()
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

fn default_version() -> u32 {
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

fn default_enabled() -> bool {
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
        LanguageConfig {
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
        GitConfig {
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

fn default_max_file_size() -> u64 {
    2048
}
fn default_parallelism() -> ParallelismConfig {
    ParallelismConfig::Auto
}

impl Default for IndexingConfig {
    fn default() -> Self {
        IndexingConfig {
            max_file_size_kb: 2048,
            parallelism: ParallelismConfig::Auto,
            fail_fast: false,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ParallelismConfig {
    Auto,
    Threads(usize),
}

impl Serialize for ParallelismConfig {
    fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        match self {
            ParallelismConfig::Auto => serializer.serialize_str("auto"),
            ParallelismConfig::Threads(threads) => serializer.serialize_u64(*threads as u64),
        }
    }
}

impl<'de> Deserialize<'de> for ParallelismConfig {
    fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        struct ParallelismVisitor;

        impl<'de> Visitor<'de> for ParallelismVisitor {
            type Value = ParallelismConfig;

            fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
                formatter.write_str(r#""auto" or a positive integer"#)
            }

            fn visit_str<E>(self, value: &str) -> std::result::Result<Self::Value, E>
            where
                E: de::Error,
            {
                match value {
                    "auto" => Ok(ParallelismConfig::Auto),
                    other => Err(E::custom(format!(
                        "invalid parallelism value '{other}', expected \"auto\""
                    ))),
                }
            }

            fn visit_u64<E>(self, value: u64) -> std::result::Result<Self::Value, E>
            where
                E: de::Error,
            {
                if value == 0 {
                    Err(E::custom(
                        "parallelism thread count must be greater than zero",
                    ))
                } else {
                    Ok(ParallelismConfig::Threads(value as usize))
                }
            }
        }

        deserializer.deserialize_any(ParallelismVisitor)
    }
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
        SearchConfig {
            fts_tokenizer: "unicode61".to_string(),
        }
    }
}

impl Default for Config {
    fn default() -> Self {
        Config {
            version: 1,
            db_path: ".spy-code/graph.db".to_string(),
            languages: LanguagesConfig {
                rust: Some(LanguageConfig::default()),
                python: Some(LanguageConfig::default()),
                typescript: Some(LanguageConfig::default()),
                go: Some(LanguageConfig::default()),
            },
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

    #[test]
    fn test_default_config_serializes_enabled_languages() {
        let config = Config::default();
        let json = serde_json::to_value(config).unwrap();

        assert_eq!(json["languages"]["rust"]["enabled"], true);
        assert_eq!(json["languages"]["python"]["enabled"], true);
        assert_eq!(json["languages"]["typescript"]["enabled"], true);
        assert_eq!(json["languages"]["go"]["enabled"], true);
        assert_eq!(json["indexing"]["parallelism"], "auto");
    }

    #[test]
    fn test_parallelism_config_deserializes_string_and_int() {
        let auto: ParallelismConfig = serde_json::from_str(r#""auto""#).unwrap();
        let threads: ParallelismConfig = serde_json::from_str("4").unwrap();

        assert_eq!(auto, ParallelismConfig::Auto);
        assert_eq!(threads, ParallelismConfig::Threads(4));
    }
}
