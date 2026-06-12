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
        for node in self.storage.get_all_nodes()? {
            scope.add_node(node);
        }

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
                        scope.remove_nodes_for_file(&file_path.to_string_lossy());

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

        self.index_dependencies(root_path)?;

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
                    self.storage.delete_nodes_for_file(&abs.to_string_lossy())?;
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
                        to_parse.push(abs);
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
                let name = entry.file_name().to_string_lossy();
                if ignore_dirs.iter().any(|d| *d == name.as_ref()) {
                    // walkdir doesn't support skip-dir natively; we filter below
                }
                continue;
            }

            // Skip files inside ignored directories
            let path = entry.path();
            let in_ignored = path.ancestors().any(|a| {
                a.file_name()
                    .map(|n| {
                        ignore_dirs
                            .iter()
                            .any(|d| *d == n.to_string_lossy().as_ref())
                    })
                    .unwrap_or(false)
            });
            if in_ignored {
                continue;
            }

            if detect_language(path).is_some() {
                let file_size = entry.metadata().map(|m| m.len()).unwrap_or(0);
                let max_bytes = self.config.indexing.max_file_size_kb * 1024;
                if file_size <= max_bytes {
                    files.push(path.to_path_buf());
                }
            }
        }

        Ok(files)
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
        let ctx = spy_parser::parse_file(path, source.clone(), lang)?;
        let resolver =
            spy_resolvers::get_resolver(lang).context("No resolver available for language")?;

        let mut nodes = resolver.extract_nodes(&ctx)?;

        if matches!(lang, Language::Image | Language::Pdf | Language::Docx | Language::Video | Language::Svg | Language::Other) {
            for node in &mut nodes {
                // Try plugin first
                let extracted_text = self.extract_with_plugin(path).or_else(|| self.extract_with_api(path, lang));
                if let Some(text) = extracted_text {
                    node.description = Some(text);
                }
            }
        }

        Ok(nodes)
    }

    fn extract_with_plugin(&self, path: &Path) -> Option<String> {
        if let Some(cmd) = &self.config.assets.plugin_command {
            let parts: Vec<&str> = cmd.split_whitespace().collect();
            if parts.is_empty() { return None; }

            let mut command = std::process::Command::new(parts[0]);
            for arg in &parts[1..] {
                command.arg(arg);
            }
            command.arg(path);

            if let Ok(output) = command.output() {
                if output.status.success() {
                    return String::from_utf8(output.stdout).ok();
                }
            }
        }
        None
    }

    fn extract_with_api(&self, path: &Path, lang: Language) -> Option<String> {
        // Fallback to OpenAI API for images and audio/video if API key exists
        let api_key = std::env::var("OPENAI_API_KEY").ok()?;
        let client = reqwest::blocking::Client::new();
        use base64::Engine;

        match lang {
            Language::Image => {
                let bytes = std::fs::read(path).ok()?;
                let base64 = base64::engine::general_purpose::STANDARD.encode(&bytes);
                let mime = match path.extension().and_then(|e| e.to_str()) {
                    Some("png") => "image/png",
                    Some("jpeg") | Some("jpg") => "image/jpeg",
                    Some("webp") => "image/webp",
                    Some("gif") => "image/gif",
                    _ => "image/jpeg",
                };

                let data_url = format!("data:{};base64,{}", mime, base64);
                let response = client.post("https://api.openai.com/v1/chat/completions")
                    .bearer_auth(&api_key)
                    .json(&serde_json::json!({
                        "model": "gpt-4-vision-preview",
                        "messages": [
                            {
                                "role": "user",
                                "content": [
                                    { "type": "text", "text": "Describe the contents of this image." },
                                    { "type": "image_url", "image_url": { "url": data_url } }
                                ]
                            }
                        ],
                        "max_tokens": 500
                    }))
                    .send()
                    .ok()?;

                let json: serde_json::Value = response.json().ok()?;
                json["choices"][0]["message"]["content"].as_str().map(|s: &str| s.to_string())
            },
            Language::Video => {
                // OpenAI whisper API for audio extraction.
                // We assume small files or already extracted audio since we just pass the file.
                // In a real production system, we would need to convert video to audio first.
                // Here we make a best effort with the direct API.
                let form = reqwest::blocking::multipart::Form::new()
                    .text("model", "whisper-1")
                    .file("file", path).ok()?;

                let response = client.post("https://api.openai.com/v1/audio/transcriptions")
                    .bearer_auth(&api_key)
                    .multipart(form)
                    .send()
                    .ok()?;

                let json: serde_json::Value = response.json().ok()?;
                json["text"].as_str().map(|s: &str| s.to_string())
            },
            _ => None
        }
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

    fn index_dependencies(&mut self, root_path: &Path) -> Result<()> {
        let cargo_toml = root_path.join("Cargo.toml");
        if cargo_toml.exists() {
            if let Ok(content) = std::fs::read_to_string(&cargo_toml) {
                let file_path = cargo_toml
                    .strip_prefix(root_path)
                    .unwrap_or(&cargo_toml)
                    .to_string_lossy()
                    .to_string();
                self.parse_cargo_dependencies(&content, &file_path)?;
            }
        }

        let pkg_json = root_path.join("package.json");
        if pkg_json.exists() {
            if let Ok(content) = std::fs::read_to_string(&pkg_json) {
                let file_path = pkg_json
                    .strip_prefix(root_path)
                    .unwrap_or(&pkg_json)
                    .to_string_lossy()
                    .to_string();
                self.parse_package_dependencies(&content, &file_path)?;
            }
        }

        let go_mod = root_path.join("go.mod");
        if go_mod.exists() {
            if let Ok(content) = std::fs::read_to_string(&go_mod) {
                let file_path = go_mod
                    .strip_prefix(root_path)
                    .unwrap_or(&go_mod)
                    .to_string_lossy()
                    .to_string();
                self.parse_go_dependencies(&content, &file_path)?;
            }
        }

        let req_txt = root_path.join("requirements.txt");
        if req_txt.exists() {
            if let Ok(content) = std::fs::read_to_string(&req_txt) {
                let file_path = req_txt
                    .strip_prefix(root_path)
                    .unwrap_or(&req_txt)
                    .to_string_lossy()
                    .to_string();
                self.parse_req_dependencies(&content, &file_path)?;
            }
        }

        Ok(())
    }

    fn add_dependency_node(&mut self, dep_name: &str, file_path: &str) -> Result<()> {
        use spy_core::{Language, Node, NodeId, NodeKind};

        let node_id = NodeId::new("dependency", file_path, "", dep_name)?;

        let node = Node {
            node_id,
            kind: NodeKind::Dependency,
            name: dep_name.to_string(),
            description: Some(format!("Dependency imported via {}", file_path)),
            signatures: vec![],
            language: Language::Rust,
            file_path: file_path.to_string(),
            start_line: 1,
            end_line: 1,
            content_hash: "".to_string(),
            git_sha: None,
            renamed_from: None,
        };

        self.storage.upsert_node(&node)?;
        Ok(())
    }

    fn parse_cargo_dependencies(&mut self, content: &str, file_path: &str) -> Result<()> {
        let mut in_deps = false;
        for line in content.lines() {
            let line = line.trim();
            if line.starts_with('[') {
                in_deps = line.contains("dependencies");
                continue;
            }
            if in_deps && !line.is_empty() && !line.starts_with('#') {
                if let Some(dep_name) = line.split('=').next() {
                    let dep_name = dep_name.trim();
                    if !dep_name.is_empty() {
                        self.add_dependency_node(dep_name, file_path)?;
                    }
                }
            }
        }
        Ok(())
    }

    fn parse_package_dependencies(&mut self, content: &str, file_path: &str) -> Result<()> {
        if let Ok(val) = serde_json::from_str::<serde_json::Value>(content) {
            if let Some(deps) = val.get("dependencies").and_then(|d| d.as_object()) {
                for dep_name in deps.keys() {
                    self.add_dependency_node(dep_name, file_path)?;
                }
            }
            if let Some(dev_deps) = val.get("devDependencies").and_then(|d| d.as_object()) {
                for dep_name in dev_deps.keys() {
                    self.add_dependency_node(dep_name, file_path)?;
                }
            }
        }
        Ok(())
    }

    fn parse_req_dependencies(&mut self, content: &str, file_path: &str) -> Result<()> {
        for line in content.lines() {
            let line = line.trim();
            if !line.is_empty() && !line.starts_with('#') {
                let dep_name = line
                    .split(&['=', '>', '<', '~', '@'][..])
                    .next()
                    .unwrap_or("")
                    .trim();
                if !dep_name.is_empty() {
                    self.add_dependency_node(dep_name, file_path)?;
                }
            }
        }
        Ok(())
    }

    fn parse_go_dependencies(&mut self, content: &str, file_path: &str) -> Result<()> {
        let mut in_require = false;
        for line in content.lines() {
            let line = line.trim();
            if line.starts_with("require (") {
                in_require = true;
                continue;
            }
            if in_require && line.starts_with(')') {
                in_require = false;
                continue;
            }
            if line.starts_with("require ") {
                let parts: Vec<&str> = line.split_whitespace().collect();
                if parts.len() >= 2 {
                    // Stripping any quotes around import path
                    let dep = parts[1].trim_matches('"');
                    self.add_dependency_node(dep, file_path)?;
                }
            } else if in_require && !line.is_empty() {
                let parts: Vec<&str> = line.split_whitespace().collect();
                if !parts.is_empty() {
                    let dep = parts[0].trim_matches('"');
                    self.add_dependency_node(dep, file_path)?;
                }
            }
        }
        Ok(())
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
