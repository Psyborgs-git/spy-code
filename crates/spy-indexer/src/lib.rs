use anyhow::{Context, Result};
use spy_core::{Config, Language, ProjectScope};
use spy_storage::{FileRecord, Storage};
use std::path::Path;
use walkdir::WalkDir;

pub struct Indexer {
    storage: Storage,
    #[allow(dead_code)]
    config: Config,
}

impl Indexer {
    pub fn new(storage: Storage, config: Config) -> Self {
        Indexer { storage, config }
    }

    pub fn index(&mut self, root_path: &Path, full: bool) -> Result<IndexStats> {
        let mut stats = IndexStats::default();
        let mut scope = ProjectScope::new();

        let files = self.discover_files(root_path)?;
        stats.files_scanned = files.len();

        for file_path in &files {
            if let Some(lang) = self.detect_language(file_path) {
                let should_parse = if full {
                    true
                } else {
                    self.should_reparse(file_path)?
                };

                if should_parse {
                    stats.files_parsed += 1;

                    let source = std::fs::read(file_path)?;
                    let content_hash = compute_file_hash(&source);

                    match self.parse_and_extract_nodes(file_path, source.clone(), lang) {
                        Ok(nodes) => {
                            for node in nodes {
                                scope.add_node(node.clone());
                                self.storage.upsert_node(&node)?;
                                stats.nodes_extracted += 1;
                            }

                            let now = std::time::SystemTime::now()
                                .duration_since(std::time::UNIX_EPOCH)?
                                .as_secs() as i64;

                            self.storage.upsert_file(&FileRecord {
                                path: file_path.to_string_lossy().to_string(),
                                language: lang.as_str().to_string(),
                                content_hash,
                                last_indexed: now,
                                git_sha: None,
                            })?;
                        }
                        Err(e) => {
                            eprintln!("Failed to parse {}: {}", file_path.display(), e);
                            stats.files_failed += 1;
                        }
                    }
                }
            }
        }

        for file_path in &files {
            if let Some(lang) = self.detect_language(file_path) {
                let source = std::fs::read(file_path)?;
                match self.extract_edges(file_path, source, lang, &scope) {
                    Ok(edges) => {
                        for edge in edges {
                            self.storage.upsert_edge(&edge)?;
                            stats.edges_extracted += 1;
                        }
                    }
                    Err(e) => {
                        eprintln!("Failed to extract edges from {}: {}", file_path.display(), e);
                    }
                }
            }
        }

        Ok(stats)
    }

    fn discover_files(&self, root: &Path) -> Result<Vec<std::path::PathBuf>> {
        let mut files = Vec::new();

        for entry in WalkDir::new(root).follow_links(false) {
            let entry = entry?;
            if entry.file_type().is_file() {
                if let Some(ext) = entry.path().extension() {
                    if ext == "rs" {
                        files.push(entry.path().to_path_buf());
                    }
                }
            }
        }

        Ok(files)
    }

    fn detect_language(&self, path: &Path) -> Option<Language> {
        path.extension()
            .and_then(|ext| ext.to_str())
            .and_then(|ext| match ext {
                "rs" => Some(Language::Rust),
                "py" => Some(Language::Python),
                "ts" => Some(Language::TypeScript),
                "js" => Some(Language::JavaScript),
                "go" => Some(Language::Go),
                _ => None,
            })
    }

    fn should_reparse(&self, path: &Path) -> Result<bool> {
        let source = std::fs::read(path)?;
        let current_hash = compute_file_hash(&source);

        if let Some(file_record) = self.storage.get_file(&path.to_string_lossy())? {
            Ok(file_record.content_hash != current_hash)
        } else {
            Ok(true)
        }
    }

    fn parse_and_extract_nodes(
        &self,
        path: &Path,
        source: Vec<u8>,
        lang: Language,
    ) -> Result<Vec<spy_core::Node>> {
        let ctx = spy_parser::parse_file(path, source, lang)?;

        let resolver = spy_resolvers::get_resolver(lang)
            .context("No resolver available for language")?;

        resolver.extract_nodes(&ctx)
    }

    fn extract_edges(
        &self,
        path: &Path,
        source: Vec<u8>,
        lang: Language,
        scope: &ProjectScope,
    ) -> Result<Vec<spy_core::Edge>> {
        let ctx = spy_parser::parse_file(path, source, lang)?;

        let resolver = spy_resolvers::get_resolver(lang)
            .context("No resolver available for language")?;

        resolver.extract_edges(&ctx, scope)
    }
}

fn compute_file_hash(source: &[u8]) -> String {
    let hash = blake3::hash(source);
    hash.to_hex().to_string()
}

#[derive(Debug, Default, Clone)]
pub struct IndexStats {
    pub files_scanned: usize,
    pub files_parsed: usize,
    pub files_failed: usize,
    pub nodes_extracted: usize,
    pub edges_extracted: usize,
}
