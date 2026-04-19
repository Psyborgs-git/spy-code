use anyhow::{Context, Result};
use rusqlite::{params, Connection, OptionalExtension};
use spy_core::{Edge, EdgeKind, Node, NodeId};
use std::path::Path;

pub struct Storage {
    conn: Connection,
}

impl Storage {
    pub fn open<P: AsRef<Path>>(path: P) -> Result<Self> {
        let path = path.as_ref();
        if let Some(parent) = path.parent() {
            std::fs::create_dir_all(parent)?;
        }
        
        let conn = Connection::open(path)
            .context("Failed to open database")?;
        
        let mut storage = Storage { conn };
        storage.migrate()?;
        Ok(storage)
    }
    
    pub fn open_in_memory() -> Result<Self> {
        let conn = Connection::open_in_memory()?;
        let mut storage = Storage { conn };
        storage.migrate()?;
        Ok(storage)
    }
    
    fn migrate(&mut self) -> Result<()> {
        self.conn.execute_batch("PRAGMA foreign_keys = ON;")?;
        self.conn.execute_batch(
            r#"
            CREATE TABLE IF NOT EXISTS nodes (
                node_id       TEXT PRIMARY KEY,
                kind          TEXT NOT NULL,
                name          TEXT NOT NULL,
                description   TEXT,
                signatures    TEXT NOT NULL,
                language      TEXT NOT NULL,
                file_path     TEXT NOT NULL,
                start_line    INTEGER NOT NULL,
                end_line      INTEGER NOT NULL,
                content_hash  TEXT NOT NULL,
                git_sha       TEXT,
                renamed_from  TEXT
            );
            
            CREATE INDEX IF NOT EXISTS idx_nodes_name ON nodes(name);
            CREATE INDEX IF NOT EXISTS idx_nodes_file ON nodes(file_path);
            CREATE INDEX IF NOT EXISTS idx_nodes_kind ON nodes(kind);
            
            CREATE TABLE IF NOT EXISTS edges_calls (
                from_id     TEXT NOT NULL,
                to_id       TEXT NOT NULL,
                confidence  REAL NOT NULL DEFAULT 1.0,
                PRIMARY KEY (from_id, to_id),
                FOREIGN KEY (from_id) REFERENCES nodes(node_id) ON DELETE CASCADE
            );
            
            CREATE INDEX IF NOT EXISTS idx_calls_to ON edges_calls(to_id);
            
            CREATE TABLE IF NOT EXISTS edges_imports (
                from_id     TEXT NOT NULL,
                to_id       TEXT NOT NULL,
                confidence  REAL NOT NULL DEFAULT 1.0,
                PRIMARY KEY (from_id, to_id),
                FOREIGN KEY (from_id) REFERENCES nodes(node_id) ON DELETE CASCADE
            );
            
            CREATE INDEX IF NOT EXISTS idx_imports_to ON edges_imports(to_id);
            
            CREATE TABLE IF NOT EXISTS edges_references (
                from_id     TEXT NOT NULL,
                to_id       TEXT NOT NULL,
                confidence  REAL NOT NULL DEFAULT 1.0,
                PRIMARY KEY (from_id, to_id),
                FOREIGN KEY (from_id) REFERENCES nodes(node_id) ON DELETE CASCADE
            );
            
            CREATE INDEX IF NOT EXISTS idx_refs_to ON edges_references(to_id);
            
            CREATE TABLE IF NOT EXISTS files (
                path           TEXT PRIMARY KEY,
                language       TEXT NOT NULL,
                content_hash   TEXT NOT NULL,
                last_indexed   INTEGER NOT NULL,
                git_sha        TEXT
            );
            
            CREATE TABLE IF NOT EXISTS index_meta (
                key TEXT PRIMARY KEY,
                value TEXT NOT NULL
            );
            "#,
        )?;
        
        self.setup_fts()?;
        Ok(())
    }
    
    fn setup_fts(&mut self) -> Result<()> {
        let fts_exists: bool = self.conn.query_row(
            "SELECT COUNT(*) > 0 FROM sqlite_master WHERE type='table' AND name='nodes_fts'",
            [],
            |row| row.get(0)
        )?;
        
        if !fts_exists {
            self.conn.execute_batch(
                r#"
                CREATE VIRTUAL TABLE nodes_fts USING fts5(
                    node_id UNINDEXED,
                    name,
                    description,
                    content=nodes,
                    content_rowid=rowid
                );
                
                INSERT INTO nodes_fts(rowid, node_id, name, description)
                SELECT rowid, node_id, name, description FROM nodes;
                
                 CREATE TRIGGER nodes_ai AFTER INSERT ON nodes BEGIN
                     INSERT INTO nodes_fts(rowid, node_id, name, description)
                     VALUES (NEW.rowid, NEW.node_id, NEW.name, NEW.description);
                 END;
                 
                 CREATE TRIGGER nodes_ad AFTER DELETE ON nodes BEGIN
                     INSERT INTO nodes_fts(nodes_fts, rowid, node_id, name, description)
                     VALUES ('delete', OLD.rowid, OLD.node_id, OLD.name, OLD.description);
                 END;
                 
                 CREATE TRIGGER nodes_au AFTER UPDATE ON nodes BEGIN
                     INSERT INTO nodes_fts(nodes_fts, rowid, node_id, name, description)
                     VALUES ('delete', OLD.rowid, OLD.node_id, OLD.name, OLD.description);
                     INSERT INTO nodes_fts(rowid, node_id, name, description)
                     VALUES (NEW.rowid, NEW.node_id, NEW.name, NEW.description);
                 END;
                 "#,
            )?;
        }
        
        Ok(())
    }
    
    pub fn upsert_node(&mut self, node: &Node) -> Result<()> {
        let signatures = serde_json::to_string(&node.signatures)?;
        
        self.conn.execute(
            r#"
            INSERT INTO nodes (
                node_id, kind, name, description, signatures, language,
                file_path, start_line, end_line, content_hash, git_sha, renamed_from
            ) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12)
            ON CONFLICT(node_id) DO UPDATE SET
                kind = excluded.kind,
                name = excluded.name,
                description = excluded.description,
                signatures = excluded.signatures,
                language = excluded.language,
                file_path = excluded.file_path,
                start_line = excluded.start_line,
                end_line = excluded.end_line,
                content_hash = excluded.content_hash,
                git_sha = excluded.git_sha,
                renamed_from = excluded.renamed_from
            "#,
            params![
                node.node_id.as_str(),
                node.kind.as_str(),
                &node.name,
                node.description.as_ref(),
                signatures,
                node.language.as_str(),
                &node.file_path,
                node.start_line,
                node.end_line,
                &node.content_hash,
                node.git_sha.as_ref(),
                node.renamed_from.as_ref().map(|id| id.as_str()),
            ],
        )?;
        
        Ok(())
    }
    
    pub fn upsert_edge(&mut self, edge: &Edge) -> Result<()> {
        let table = edge.kind.table_name();
        let query = format!(
            "INSERT INTO {} (from_id, to_id, confidence) VALUES (?1, ?2, ?3)
             ON CONFLICT(from_id, to_id) DO UPDATE SET confidence = excluded.confidence",
            table
        );
        
        self.conn.execute(
            &query,
            params![
                edge.from_id.as_str(),
                edge.to_id.as_str(),
                edge.confidence,
            ],
        )?;
        
        Ok(())
    }
    
    pub fn get_node(&self, node_id: &str) -> Result<Option<Node>> {
        let result = self.conn.query_row(
            "SELECT node_id, kind, name, description, signatures, language, 
                    file_path, start_line, end_line, content_hash, git_sha, renamed_from
             FROM nodes WHERE node_id = ?1",
            params![node_id],
            |row| {
                let signatures_str: String = row.get(4)?;
                let signatures = serde_json::from_str(&signatures_str)
                    .map_err(|_e| rusqlite::Error::InvalidQuery)?;
                
                let kind_str: String = row.get(1)?;
                let kind = match kind_str.as_str() {
                    "function" => spy_core::NodeKind::Function,
                    "class" => spy_core::NodeKind::Class,
                    "constant" => spy_core::NodeKind::Constant,
                    _ => return Err(rusqlite::Error::InvalidQuery),
                };
                
                let lang_str: String = row.get(5)?;
                let language = match lang_str.as_str() {
                    "rust" => spy_core::Language::Rust,
                    "python" => spy_core::Language::Python,
                    "typescript" => spy_core::Language::TypeScript,
                    "javascript" => spy_core::Language::JavaScript,
                    "go" => spy_core::Language::Go,
                    _ => return Err(rusqlite::Error::InvalidQuery),
                };
                
                let renamed_from_str: Option<String> = row.get(11)?;
                let renamed_from = renamed_from_str
                    .map(|s| NodeId::from_string(s))
                    .transpose()
                    .map_err(|_| rusqlite::Error::InvalidQuery)?;
                
                Ok(Node {
                    node_id: NodeId::from_string(row.get(0)?)
                        .map_err(|_| rusqlite::Error::InvalidQuery)?,
                    kind,
                    name: row.get(2)?,
                    description: row.get(3)?,
                    signatures,
                    language,
                    file_path: row.get(6)?,
                    start_line: row.get(7)?,
                    end_line: row.get(8)?,
                    content_hash: row.get(9)?,
                    git_sha: row.get(10)?,
                    renamed_from,
                })
            },
        ).optional()?;
        
        Ok(result)
    }
    
    pub fn delete_nodes_for_file(&mut self, file_path: &str) -> Result<()> {
        // Delete stale edges before deleting nodes (also ensures cleanup even if FK cascade
        // is not active on this SQLite build)
        self.conn.execute(
            "DELETE FROM edges_calls WHERE from_id IN (SELECT node_id FROM nodes WHERE file_path = ?1)",
            params![file_path],
        )?;
        self.conn.execute(
            "DELETE FROM edges_imports WHERE from_id IN (SELECT node_id FROM nodes WHERE file_path = ?1)",
            params![file_path],
        )?;
        self.conn.execute(
            "DELETE FROM edges_references WHERE from_id IN (SELECT node_id FROM nodes WHERE file_path = ?1)",
            params![file_path],
        )?;
        self.conn.execute(
            "DELETE FROM nodes WHERE file_path = ?1",
            params![file_path],
        )?;
        Ok(())
    }
    
    pub fn search_nodes(&self, query: &str, limit: usize) -> Result<Vec<(Node, f64)>> {
        let mut stmt = self.conn.prepare(
            "SELECT n.node_id, n.kind, n.name, n.description, n.signatures, n.language,
                    n.file_path, n.start_line, n.end_line, n.content_hash, n.git_sha, n.renamed_from,
                    rank
             FROM nodes_fts
             JOIN nodes n ON nodes_fts.rowid = n.rowid
             WHERE nodes_fts MATCH ?1
             ORDER BY rank
             LIMIT ?2"
        )?;
        
        let rows = stmt.query_map(params![query, limit], |row| {
            let signatures_str: String = row.get(4)?;
            let signatures = serde_json::from_str(&signatures_str)
                .map_err(|_| rusqlite::Error::InvalidQuery)?;
            
            let kind_str: String = row.get(1)?;
            let kind = match kind_str.as_str() {
                "function" => spy_core::NodeKind::Function,
                "class" => spy_core::NodeKind::Class,
                "constant" => spy_core::NodeKind::Constant,
                _ => return Err(rusqlite::Error::InvalidQuery),
            };
            
            let lang_str: String = row.get(5)?;
            let language = match lang_str.as_str() {
                "rust" => spy_core::Language::Rust,
                "python" => spy_core::Language::Python,
                "typescript" => spy_core::Language::TypeScript,
                "javascript" => spy_core::Language::JavaScript,
                "go" => spy_core::Language::Go,
                _ => return Err(rusqlite::Error::InvalidQuery),
            };
            
            let renamed_from_str: Option<String> = row.get(11)?;
            let renamed_from = renamed_from_str
                .map(|s| NodeId::from_string(s))
                .transpose()
                .map_err(|_| rusqlite::Error::InvalidQuery)?;
            
            let rank: f64 = row.get(12)?;
            
            Ok((Node {
                node_id: NodeId::from_string(row.get(0)?)
                    .map_err(|_| rusqlite::Error::InvalidQuery)?,
                kind,
                name: row.get(2)?,
                description: row.get(3)?,
                signatures,
                language,
                file_path: row.get(6)?,
                start_line: row.get(7)?,
                end_line: row.get(8)?,
                content_hash: row.get(9)?,
                git_sha: row.get(10)?,
                renamed_from,
            }, rank))
        })?;
        
        let mut results = Vec::new();
        for row in rows {
            results.push(row?);
        }
        Ok(results)
    }
    
    pub fn get_file(&self, path: &str) -> Result<Option<FileRecord>> {
        let result = self.conn.query_row(
            "SELECT path, language, content_hash, last_indexed, git_sha FROM files WHERE path = ?1",
            params![path],
            |row| {
                Ok(FileRecord {
                    path: row.get(0)?,
                    language: row.get(1)?,
                    content_hash: row.get(2)?,
                    last_indexed: row.get(3)?,
                    git_sha: row.get(4)?,
                })
            },
        ).optional()?;
        
        Ok(result)
    }
    
    pub fn upsert_file(&mut self, file: &FileRecord) -> Result<()> {
        self.conn.execute(
            r#"
            INSERT INTO files (path, language, content_hash, last_indexed, git_sha)
            VALUES (?1, ?2, ?3, ?4, ?5)
            ON CONFLICT(path) DO UPDATE SET
                language = excluded.language,
                content_hash = excluded.content_hash,
                last_indexed = excluded.last_indexed,
                git_sha = excluded.git_sha
            "#,
            params![
                &file.path,
                &file.language,
                &file.content_hash,
                file.last_indexed,
                file.git_sha.as_ref(),
            ],
        )?;
        Ok(())
    }
    
    pub fn get_meta(&self, key: &str) -> Result<Option<String>> {
        let result = self.conn.query_row(
            "SELECT value FROM index_meta WHERE key = ?1",
            params![key],
            |row| row.get(0),
        ).optional()?;
        
        Ok(result)
    }
    
    pub fn set_meta(&mut self, key: &str, value: &str) -> Result<()> {
        self.conn.execute(
            "INSERT INTO index_meta (key, value) VALUES (?1, ?2)
             ON CONFLICT(key) DO UPDATE SET value = excluded.value",
            params![key, value],
        )?;
        Ok(())
    }
    
    pub fn get_edges(&self, from_id: &str, kind: EdgeKind) -> Result<Vec<Edge>> {
        let table = kind.table_name();
        let query = format!(
            "SELECT from_id, to_id, confidence FROM {} WHERE from_id = ?1",
            table
        );
        
        let mut stmt = self.conn.prepare(&query)?;
        let rows = stmt.query_map(params![from_id], |row| {
            Ok(Edge {
                from_id: NodeId::from_string(row.get(0)?)
                    .map_err(|_| rusqlite::Error::InvalidQuery)?,
                to_id: NodeId::from_string(row.get(1)?)
                    .map_err(|_| rusqlite::Error::InvalidQuery)?,
                kind,
                confidence: row.get(2)?,
            })
        })?;
        
        let mut results = Vec::new();
        for row in rows {
            results.push(row?);
        }
        Ok(results)
    }
    
    pub fn get_incoming_edges(&self, to_id: &str, kind: EdgeKind) -> Result<Vec<Edge>> {
        let table = kind.table_name();
        let query = format!(
            "SELECT from_id, to_id, confidence FROM {} WHERE to_id = ?1",
            table
        );
        
        let mut stmt = self.conn.prepare(&query)?;
        let rows = stmt.query_map(params![to_id], |row| {
            Ok(Edge {
                from_id: NodeId::from_string(row.get(0)?)
                    .map_err(|_| rusqlite::Error::InvalidQuery)?,
                to_id: NodeId::from_string(row.get(1)?)
                    .map_err(|_| rusqlite::Error::InvalidQuery)?,
                kind,
                confidence: row.get(2)?,
            })
        })?;
        
        let mut results = Vec::new();
        for row in rows {
            results.push(row?);
        }
        Ok(results)
    }
    
    pub fn list_files(&self) -> Result<Vec<String>> {
        let mut stmt = self.conn.prepare("SELECT path FROM files ORDER BY path")?;
        let rows = stmt.query_map([], |row| row.get(0))?;
        let mut paths = Vec::new();
        for row in rows {
            paths.push(row?);
        }
        Ok(paths)
    }

    pub fn get_nodes_for_files(&self, file_paths: &[String]) -> Result<Vec<Node>> {
        if file_paths.is_empty() {
            return Ok(vec![]);
        }
        let placeholders: String = file_paths
            .iter()
            .enumerate()
            .map(|(i, _)| format!("?{}", i + 1))
            .collect::<Vec<_>>()
            .join(", ");
        let query = format!(
            "SELECT node_id, kind, name, description, signatures, language,
                    file_path, start_line, end_line, content_hash, git_sha, renamed_from
             FROM nodes WHERE file_path IN ({})",
            placeholders
        );
        let mut stmt = self.conn.prepare(&query)?;
        let rows = stmt.query_map(
            rusqlite::params_from_iter(file_paths.iter()),
            |row| {
                let signatures_str: String = row.get(4)?;
                let signatures = serde_json::from_str(&signatures_str)
                    .map_err(|_| rusqlite::Error::InvalidQuery)?;
                let kind_str: String = row.get(1)?;
                let kind = match kind_str.as_str() {
                    "function" => spy_core::NodeKind::Function,
                    "class" => spy_core::NodeKind::Class,
                    "constant" => spy_core::NodeKind::Constant,
                    _ => return Err(rusqlite::Error::InvalidQuery),
                };
                let lang_str: String = row.get(5)?;
                let language = match lang_str.as_str() {
                    "rust" => spy_core::Language::Rust,
                    "python" => spy_core::Language::Python,
                    "typescript" => spy_core::Language::TypeScript,
                    "javascript" => spy_core::Language::JavaScript,
                    "go" => spy_core::Language::Go,
                    _ => return Err(rusqlite::Error::InvalidQuery),
                };
                let renamed_from_str: Option<String> = row.get(11)?;
                let renamed_from = renamed_from_str
                    .map(|s| NodeId::from_string(s))
                    .transpose()
                    .map_err(|_| rusqlite::Error::InvalidQuery)?;
                Ok(Node {
                    node_id: NodeId::from_string(row.get(0)?)
                        .map_err(|_| rusqlite::Error::InvalidQuery)?,
                    kind,
                    name: row.get(2)?,
                    description: row.get(3)?,
                    signatures,
                    language,
                    file_path: row.get(6)?,
                    start_line: row.get(7)?,
                    end_line: row.get(8)?,
                    content_hash: row.get(9)?,
                    git_sha: row.get(10)?,
                    renamed_from,
                })
            },
        )?;
        let mut results = Vec::new();
        for row in rows {
            results.push(row?);
        }
        Ok(results)
    }

    pub fn get_stats(&self) -> Result<IndexStats> {
        let node_count: i64 = self.conn.query_row("SELECT COUNT(*) FROM nodes", [], |row| row.get(0))?;
        let edge_count: i64 = self.conn.query_row(
            "SELECT (SELECT COUNT(*) FROM edges_calls) +
                    (SELECT COUNT(*) FROM edges_imports) +
                    (SELECT COUNT(*) FROM edges_references)",
            [],
            |row| row.get(0)
        )?;
        let file_count: i64 = self.conn.query_row("SELECT COUNT(*) FROM files", [], |row| row.get(0))?;
        let last_git_sha = self.get_meta("last_git_sha")?;
        
        Ok(IndexStats {
            node_count: node_count as usize,
            edge_count: edge_count as usize,
            file_count: file_count as usize,
            last_git_sha,
        })
    }
}

#[derive(Debug, Clone)]
pub struct FileRecord {
    pub path: String,
    pub language: String,
    pub content_hash: String,
    pub last_indexed: i64,
    pub git_sha: Option<String>,
}

#[derive(Debug, Clone)]
pub struct IndexStats {
    pub node_count: usize,
    pub edge_count: usize,
    pub file_count: usize,
    pub last_git_sha: Option<String>,
}

#[cfg(test)]
mod tests {
    use super::*;
    use spy_core::{Language, NodeKind, Signature};

    fn sample_node(file_path: &str, symbol: &str, name: &str, description: Option<&str>) -> Result<Node> {
        Ok(Node {
            node_id: NodeId::new("src", file_path, "_", symbol)?,
            kind: NodeKind::Function,
            name: name.to_string(),
            description: description.map(str::to_string),
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
    
    #[test]
    fn test_upsert_and_get_node() -> Result<()> {
        let mut storage = Storage::open_in_memory()?;
        
        let node = Node {
            node_id: NodeId::new("src", "lib.rs", "_", "test_fn")?,
            kind: NodeKind::Function,
            name: "test_fn".to_string(),
            description: Some("A test function".to_string()),
            signatures: vec![Signature {
                params: vec![],
                returns: Some("()".to_string()),
            }],
            language: Language::Rust,
            file_path: "src/lib.rs".to_string(),
            start_line: 1,
            end_line: 5,
            content_hash: "abc123".to_string(),
            git_sha: None,
            renamed_from: None,
        };
        
        storage.upsert_node(&node)?;
        
        let retrieved = storage.get_node("src:lib.rs:_:test_fn")?;
        assert!(retrieved.is_some());
        let retrieved = retrieved.unwrap();
        assert_eq!(retrieved.name, "test_fn");
        assert_eq!(retrieved.description, Some("A test function".to_string()));
        
        Ok(())
    }
    
    #[test]
    fn test_search_nodes() -> Result<()> {
        let mut storage = Storage::open_in_memory()?;
        
        let node = Node {
            node_id: NodeId::new("src", "lib.rs", "_", "auth_user")?,
            kind: NodeKind::Function,
            name: "auth_user".to_string(),
            description: Some("Authenticate a user".to_string()),
            signatures: vec![],
            language: Language::Rust,
            file_path: "src/lib.rs".to_string(),
            start_line: 1,
            end_line: 5,
            content_hash: "abc123".to_string(),
            git_sha: None,
            renamed_from: None,
        };
        
        storage.upsert_node(&node)?;
        
        let results = storage.search_nodes("auth", 10)?;
        assert!(!results.is_empty());
        
        Ok(())
    }

    #[test]
    fn test_upsert_node_updates_existing_search_index() -> Result<()> {
        let mut storage = Storage::open_in_memory()?;
        let mut node = sample_node("lib.rs", "auth_user", "auth_user", Some("Authenticate a user"))?;
        storage.upsert_node(&node)?;

        let original_results = storage.search_nodes("Authenticate", 10)?;
        assert_eq!(original_results.len(), 1);

        node.description = Some("Authorize a request".to_string());
        node.content_hash = "updated-hash".to_string();
        storage.upsert_node(&node)?;

        assert!(storage.search_nodes("Authenticate", 10)?.is_empty());
        let updated_results = storage.search_nodes("Authorize", 10)?;
        assert_eq!(updated_results.len(), 1);
        assert_eq!(updated_results[0].0.content_hash, "updated-hash");

        Ok(())
    }

    #[test]
    fn test_edges_files_meta_and_stats() -> Result<()> {
        let mut storage = Storage::open_in_memory()?;
        let node_a = sample_node("a.rs", "a", "alpha", Some("Alpha node"))?;
        let node_b = sample_node("b.rs", "b", "beta", Some("Beta node"))?;

        storage.upsert_node(&node_a)?;
        storage.upsert_node(&node_b)?;

        let edge = Edge {
            from_id: node_a.node_id.clone(),
            to_id: node_b.node_id.clone(),
            kind: EdgeKind::Calls,
            confidence: 0.8,
        };
        storage.upsert_edge(&edge)?;

        let outgoing = storage.get_edges(node_a.node_id.as_str(), EdgeKind::Calls)?;
        assert_eq!(outgoing.len(), 1);
        assert_eq!(outgoing[0].to_id, node_b.node_id);
        assert_eq!(outgoing[0].confidence, 0.8);

        let incoming = storage.get_incoming_edges(node_b.node_id.as_str(), EdgeKind::Calls)?;
        assert_eq!(incoming.len(), 1);
        assert_eq!(incoming[0].from_id, node_a.node_id);

        let file_a = FileRecord {
            path: "src/a.rs".to_string(),
            language: "rust".to_string(),
            content_hash: "hash-a".to_string(),
            last_indexed: 10,
            git_sha: Some("sha-a".to_string()),
        };
        let file_b = FileRecord {
            path: "src/b.rs".to_string(),
            language: "rust".to_string(),
            content_hash: "hash-b".to_string(),
            last_indexed: 20,
            git_sha: None,
        };
        storage.upsert_file(&file_b)?;
        storage.upsert_file(&file_a)?;

        let listed_files = storage.list_files()?;
        assert_eq!(listed_files, vec!["src/a.rs".to_string(), "src/b.rs".to_string()]);

        let stored_file = storage.get_file("src/a.rs")?.unwrap();
        assert_eq!(stored_file.content_hash, "hash-a");
        assert_eq!(stored_file.git_sha.as_deref(), Some("sha-a"));

        let file_nodes = storage.get_nodes_for_files(&listed_files)?;
        assert_eq!(file_nodes.len(), 2);
        assert!(storage.get_nodes_for_files(&[] as &[String])?.is_empty());

        storage.set_meta("last_git_sha", "sha123")?;
        assert_eq!(storage.get_meta("last_git_sha")?, Some("sha123".to_string()));

        let stats = storage.get_stats()?;
        assert_eq!(stats.node_count, 2);
        assert_eq!(stats.edge_count, 1);
        assert_eq!(stats.file_count, 2);
        assert_eq!(stats.last_git_sha.as_deref(), Some("sha123"));

        Ok(())
    }

    #[test]
    fn test_delete_nodes_for_file_removes_matching_nodes() -> Result<()> {
        let mut storage = Storage::open_in_memory()?;
        let keep = sample_node("keep.rs", "keep", "keep", None)?;
        let delete = sample_node("delete.rs", "delete", "delete", None)?;

        storage.upsert_node(&keep)?;
        storage.upsert_node(&delete)?;
        storage.delete_nodes_for_file("src/delete.rs")?;

        assert!(storage.get_node(delete.node_id.as_str())?.is_none());
        assert!(storage.get_node(keep.node_id.as_str())?.is_some());

        Ok(())
    }
}
