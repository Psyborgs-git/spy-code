use anyhow::{Context, Result};
use spy_core::{Config, Language, ProjectScope};
use spy_git::{FileChangeStatus, GitRepo};
use spy_storage::{FileRecord, Storage};
use std::path::Path;
use walkdir::WalkDir;

pub struct Indexer {
    storage: Storage,
    config: Config,
}

impl Indexer {
    pub fn new(storage: Storage, config: Config) -> Self {
        Indexer { storage, config }
    }

    pub fn index(&mut self, root_path: &Path, full: bool) -> Result<IndexStats> {
        let mut stats = IndexStats::default();

        // Config hash invalidation: force full re-index when config changes.
        let config_json = serde_json::to_string(&self.config).unwrap_or_default();
        let config_hash = blake3::hash(config_json.as_bytes()).to_hex().to_string();
        let stored_config_hash = self.storage.get_meta("last_config_hash")?;
        let full = if stored_config_hash.as_deref() != Some(config_hash.as_str()) {
            true
        } else {
            full
        };

        // Capture current HEAD SHA once for all file records in this run.
        let current_git_sha = if let Ok(Some(repo)) = GitRepo::discover(root_path) {
            repo.current_sha()
        } else {
            None
        };

        // Determine which files to parse (either all, or just the diff from git)
        let files_to_parse = if full {
            let all = self.discover_files(root_path)?;
            stats.files_scanned = all.len();
            all
        } else {
            self.incremental_files(root_path, &mut stats)?
        };

        // Pass 1 — extract nodes from each file and build project scope
        let mut scope = ProjectScope::new();
        for file_path in &files_to_parse {
            if let Some(lang) = detect_language(file_path) {
                stats.files_parsed += 1;
                let source = std::fs::read(file_path)?;
                let content_hash = compute_file_hash(&source);

                match self.parse_and_extract_nodes(file_path, source.clone(), lang) {
                    Ok(nodes) => {
                        // Remove stale nodes for this file then insert fresh ones
                        self.storage
                            .delete_nodes_for_file(&file_path.to_string_lossy())?;

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
                            git_sha: current_git_sha.clone(),
                        })?;
                    }
                    Err(e) => {
                        eprintln!("Failed to parse {}: {}", file_path.display(), e);
                        if self.config.indexing.fail_fast {
                            return Err(e);
                        }
                        stats.files_failed += 1;
                    }
                }
            }
        }

        // Pass 2 — extract edges now that the full scope is known
        for file_path in &files_to_parse {
            if let Some(lang) = detect_language(file_path) {
                let source = match std::fs::read(file_path) {
                    Ok(s) => s,
                    Err(_) => continue,
                };
                match self.extract_edges(file_path, source, lang, &scope) {
                    Ok(edges) => {
                        for edge in edges {
                            self.storage.upsert_edge(&edge)?;
                            stats.edges_extracted += 1;
                        }
                    }
                    Err(e) => {
                        eprintln!(
                            "Failed to extract edges from {}: {}",
                            file_path.display(),
                            e
                        );
                    }
                }
            }
        }

        // Persist the HEAD SHA so the next incremental run can diff against it
        if let Ok(Some(repo)) = GitRepo::discover(root_path) {
            if let Some(sha) = repo.current_sha() {
                let stored_sha = if repo.is_dirty() {
                    format!("{}+dirty", sha)
                } else {
                    sha
                };
                self.storage.set_meta("last_git_sha", &stored_sha)?;
            }
        }

        // Persist config hash so we can detect config changes on next run
        self.storage.set_meta("last_config_hash", &config_hash)?;

        Ok(stats)
    }

    // -----------------------------------------------------------------------
    // File discovery
    // -----------------------------------------------------------------------

    /// For incremental mode: use git diff when available, fall back to
    /// content-hash comparison.
    fn incremental_files(
        &mut self,
        root_path: &Path,
        stats: &mut IndexStats,
    ) -> Result<Vec<std::path::PathBuf>> {
        let all_files = self.discover_files(root_path)?;
        stats.files_scanned = all_files.len();

        // Try git-based incremental first
        if self.config.git.enabled {
            if let Ok(Some(repo)) = GitRepo::discover(root_path) {
                if let Some(last_sha) = self.storage.get_meta("last_git_sha")? {
                    // Warn when working tree is dirty
                    if repo.is_dirty() {
                        eprintln!("Warning: working tree is dirty; indexed state may not match HEAD");
                    }

                    // Strip the +dirty suffix before passing to git diff
                    let clean_sha = last_sha.trim_end_matches("+dirty").to_string();

                    match repo.diff_files_since(&clean_sha) {
                        Ok(diffs) => {
                            return self.apply_git_diff(diffs, root_path, &all_files);
                        }
                        Err(e) => {
                            eprintln!(
                                "Warning: git diff failed ({}), falling back to full scan",
                                e
                            );
                        }
                    }
                }
            }
        }

        // Fall back: only re-parse files whose content hash changed
        let mut changed = Vec::new();
        for path in all_files {
            if self.should_reparse(&path)? {
                changed.push(path);
            }
        }
        Ok(changed)
    }

    /// Process deleted files from the diff and return the set of files to
    /// (re-)parse.
    fn apply_git_diff(
        &mut self,
        diffs: Vec<spy_git::FileDiff>,
        workdir: &Path,
        _all_files: &[std::path::PathBuf],
    ) -> Result<Vec<std::path::PathBuf>> {
        let mut to_parse = Vec::new();

        for diff in diffs {
            let abs = workdir.join(&diff.path);
            match &diff.status {
                FileChangeStatus::Deleted => {
                    self.storage
                        .delete_nodes_for_file(&abs.to_string_lossy())?;
                }
                FileChangeStatus::Renamed { old_path } => {
                    let old_abs = workdir.join(old_path);
                    self.storage
                        .delete_nodes_for_file(&old_abs.to_string_lossy())?;
                    if detect_language(&abs).is_some() {
                        to_parse.push(abs);
                    }
                }
                FileChangeStatus::Added | FileChangeStatus::Modified => {
                    if detect_language(&abs).is_some() && abs.exists() {
                        // Content-hash gating: skip if file hasn't actually changed
                        if self.should_reparse(&abs)? {
                            to_parse.push(abs);
                        }
                    }
                }
            }
        }

        Ok(to_parse)
    }

    fn discover_files(&self, root: &Path) -> Result<Vec<std::path::PathBuf>> {
        let ignore_dirs: &[&str] = &[
            "target",
            ".git",
            "__pycache__",
            ".venv",
            "node_modules",
            ".mypy_cache",
            "dist",
            "build",
        ];

        let mut files = Vec::new();

        for entry in WalkDir::new(root).follow_links(self.config.git.follow_symlinks) {
            let entry = entry?;
            if entry.file_type().is_dir() {
                continue;
            }

            // Skip files inside ignored directories. WalkDir has no built-in skip-dir
            // API, so we check each file's path ancestors against ignore_dirs.
            let path = entry.path();
            let in_ignored = path.ancestors().any(|a| {
                a.file_name()
                    .map(|n| ignore_dirs.iter().any(|d| *d == n.to_string_lossy().as_ref()))
                    .unwrap_or(false)
            });
            if in_ignored {
                continue;
            }

            if let Some(lang) = detect_language(path) {
                if !self.is_language_enabled(lang) {
                    continue;
                }
                let file_size = entry.metadata().map(|m| m.len()).unwrap_or(0);
                let max_bytes = self.config.indexing.max_file_size_kb * 1024;
                if file_size <= max_bytes {
                    files.push(path.to_path_buf());
                }
            }
        }

        Ok(files)
    }

    fn is_language_enabled(&self, lang: Language) -> bool {
        match lang {
            Language::Rust => self.config.languages.rust.as_ref().map(|c| c.enabled).unwrap_or(true),
            Language::Python => self.config.languages.python.as_ref().map(|c| c.enabled).unwrap_or(true),
            Language::TypeScript | Language::JavaScript => {
                self.config.languages.typescript.as_ref().map(|c| c.enabled).unwrap_or(true)
            }
            Language::Go => self.config.languages.go.as_ref().map(|c| c.enabled).unwrap_or(true),
        }
    }

    fn should_reparse(&self, path: &Path) -> Result<bool> {
        let source = std::fs::read(path)?;
        let current_hash = compute_file_hash(&source);
        if let Some(rec) = self.storage.get_file(&path.to_string_lossy())? {
            Ok(rec.content_hash != current_hash)
        } else {
            Ok(true)
        }
    }

    // -----------------------------------------------------------------------
    // Parse / edge helpers
    // -----------------------------------------------------------------------

    fn parse_and_extract_nodes(
        &self,
        path: &Path,
        source: Vec<u8>,
        lang: Language,
    ) -> Result<Vec<spy_core::Node>> {
        let ctx = spy_parser::parse_file(path, source, lang)?;
        let resolver =
            spy_resolvers::get_resolver(lang).context("No resolver available for language")?;
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
        let resolver =
            spy_resolvers::get_resolver(lang).context("No resolver available for language")?;
        resolver.extract_edges(&ctx, scope)
    }
}

// ---------------------------------------------------------------------------
// Helpers
// ---------------------------------------------------------------------------

pub fn detect_language(path: &Path) -> Option<Language> {
    path.extension()
        .and_then(|ext| ext.to_str())
        .and_then(|ext| match ext {
            "rs" => Some(Language::Rust),
            "py" => Some(Language::Python),
            "ts" | "tsx" => Some(Language::TypeScript),
            "js" | "jsx" | "mjs" | "cjs" => Some(Language::JavaScript),
            "go" => Some(Language::Go),
            _ => None,
        })
}

fn compute_file_hash(source: &[u8]) -> String {
    blake3::hash(source).to_hex().to_string()
}

#[derive(Debug, Default, Clone)]
pub struct IndexStats {
    pub files_scanned: usize,
    pub files_parsed: usize,
    pub files_failed: usize,
    pub nodes_extracted: usize,
    pub edges_extracted: usize,
}

