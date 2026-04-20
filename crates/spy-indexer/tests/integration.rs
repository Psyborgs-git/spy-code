use anyhow::Result;
use spy_core::{Config, EdgeKind, Language, LanguageConfig};
use spy_indexer::{detect_language, Indexer};
use spy_storage::Storage;
use std::path::Path;
use tempfile::TempDir;

fn make_storage() -> (TempDir, Storage) {
    let dir = TempDir::new().unwrap();
    let db_path = dir.path().join("graph.db");
    let storage = Storage::open(&db_path).unwrap();
    (dir, storage)
}

fn fixtures_path(lang: &str) -> std::path::PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent().unwrap()
        .parent().unwrap()
        .join("tests/fixtures")
        .join(lang)
}

#[test]
fn test_detect_language() {
    assert_eq!(detect_language(Path::new("foo.rs")), Some(Language::Rust));
    assert_eq!(detect_language(Path::new("foo.py")), Some(Language::Python));
    assert_eq!(detect_language(Path::new("foo.ts")), Some(Language::TypeScript));
    assert_eq!(detect_language(Path::new("foo.tsx")), Some(Language::TypeScript));
    assert_eq!(detect_language(Path::new("foo.js")), Some(Language::JavaScript));
    assert_eq!(detect_language(Path::new("foo.go")), Some(Language::Go));
    assert_eq!(detect_language(Path::new("foo.txt")), None);
}

#[test]
fn test_index_rust_fixtures() -> Result<()> {
    let root = fixtures_path("rust_sample");
    if !root.exists() {
        return Ok(()); // skip if fixtures not present
    }
    let (_dir, storage) = make_storage();
    let mut indexer = Indexer::new(storage, Config::default());
    let stats = indexer.index(&root, true)?;
    // math.rs: add, subtract, MAX_VALUE = at least 3 nodes
    // traits.rs: Animal (trait), Dog (struct), multiple methods
    assert!(stats.nodes_extracted >= 3, "Expected >=3 nodes, got {}", stats.nodes_extracted);
    Ok(())
}

#[test]
fn test_index_python_fixtures() -> Result<()> {
    let root = fixtures_path("python_sample");
    if !root.exists() {
        return Ok(());
    }
    let (_dir, storage) = make_storage();
    let mut indexer = Indexer::new(storage, Config::default());
    let stats = indexer.index(&root, true)?;
    assert!(stats.nodes_extracted >= 4, "Expected >=4 nodes, got {}", stats.nodes_extracted);
    Ok(())
}

#[test]
fn test_index_typescript_fixtures() -> Result<()> {
    let root = fixtures_path("ts_sample");
    if !root.exists() {
        return Ok(());
    }
    let (_dir, storage) = make_storage();
    let mut indexer = Indexer::new(storage, Config::default());
    let stats = indexer.index(&root, true)?;
    // add, subtract, MAX_VALUE, Animal, Dog, speak x2, constructor
    assert!(stats.nodes_extracted >= 4, "Expected >=4 nodes, got {}", stats.nodes_extracted);
    Ok(())
}

#[test]
fn test_index_go_fixtures() -> Result<()> {
    let root = fixtures_path("go_sample");
    if !root.exists() {
        return Ok(());
    }
    let (_dir, storage) = make_storage();
    let mut indexer = Indexer::new(storage, Config::default());
    let stats = indexer.index(&root, true)?;
    // Add, Subtract, MaxValue, Animal, Dog, Speak x2
    assert!(stats.nodes_extracted >= 4, "Expected >=4 nodes, got {}", stats.nodes_extracted);
    Ok(())
}

#[test]
fn test_incremental_index_skips_unchanged() -> Result<()> {
    let root = fixtures_path("rust_sample");
    if !root.exists() {
        return Ok(());
    }
    let (_dir, storage) = make_storage();
    let mut indexer = Indexer::new(storage, Config::default());

    // First index
    let stats1 = indexer.index(&root, true)?;
    assert!(stats1.files_parsed > 0);

    // Second incremental index — nothing changed, should parse 0 files
    let stats2 = indexer.index(&root, false)?;
    assert_eq!(
        stats2.files_parsed, 0,
        "Incremental index should parse 0 unchanged files"
    );
    Ok(())
}

// ---------------------------------------------------------------------------
// Python imports and references edge extraction
// ---------------------------------------------------------------------------

#[test]
fn test_python_imports_edges_across_files() -> Result<()> {
    let root = fixtures_path("python_sample");
    if !root.exists() {
        return Ok(());
    }

    let (_dir, storage) = make_storage();
    let mut indexer = Indexer::new(storage, Config::default());
    let stats = indexer.index(&root, true)?;

    // zoo.py imports `add`, `Animal`, `Dog` from sibling files
    // → at least some import edges should have been extracted
    assert!(
        stats.edges_extracted > 0,
        "Expected >0 edges (imports/calls), got {}",
        stats.edges_extracted
    );
    Ok(())
}

#[test]
fn test_python_imports_edges_stored() -> Result<()> {
    let root = fixtures_path("python_sample");
    if !root.exists() {
        return Ok(());
    }

    let tmp = TempDir::new()?;
    let db_path = tmp.path().join("graph.db");
    {
        let storage = Storage::open(&db_path)?;
        let mut indexer = Indexer::new(storage, Config::default());
        indexer.index(&root, true)?;
    }

    let storage = Storage::open(&db_path)?;

    // Find all nodes named "add" (from math.py)
    let results = storage.search_nodes("add", 10)?;
    let add_node = results.iter().find(|(n, _)| n.name == "add");
    assert!(add_node.is_some(), "Expected 'add' node to be indexed");

    let add_id = add_node.unwrap().0.node_id.as_str().to_string();
    let importers = storage.get_incoming_edges(&add_id, EdgeKind::Imports)?;
    assert!(
        !importers.is_empty(),
        "Expected zoo.py nodes to import 'add' from math.py; got no import edges"
    );

    Ok(())
}

// ---------------------------------------------------------------------------
// Config roots and ignore
// ---------------------------------------------------------------------------

#[test]
fn test_config_ignore_excludes_files() -> Result<()> {
    let root = fixtures_path("python_sample");
    if !root.exists() {
        return Ok(());
    }

    // Configure Python to ignore zoo.py
    let mut config = Config::default();
    config.languages.python = Some(LanguageConfig {
        enabled: true,
        roots: vec!["./".to_string()],
        ignore: vec!["zoo.py".to_string()],
        resolver: "builtin".to_string(),
        tsconfig: None,
    });

    let tmp = TempDir::new()?;
    let db_path = tmp.path().join("graph.db");
    let storage = Storage::open(&db_path)?;
    let mut indexer = Indexer::new(storage, config);
    indexer.index(&root, true)?;

    let storage = Storage::open(&db_path)?;
    // Zoo class and create_zoo should NOT be indexed
    let results = storage.search_nodes("Zoo", 10)?;
    let zoo_nodes: Vec<_> = results.iter().filter(|(n, _)| n.name == "Zoo").collect();
    assert!(
        zoo_nodes.is_empty(),
        "Expected 'Zoo' to be excluded by ignore pattern but it was indexed"
    );
    Ok(())
}

#[test]
fn test_config_roots_restricts_scan() -> Result<()> {
    let root = fixtures_path("python_sample");
    if !root.exists() {
        return Ok(());
    }

    // This test verifies that when we configure non-existent roots,
    // no Python files from the fixture dir are indexed.
    let mut config = Config::default();
    config.languages.python = Some(LanguageConfig {
        enabled: true,
        roots: vec!["nonexistent_dir/".to_string()],
        ignore: vec![],
        resolver: "builtin".to_string(),
        tsconfig: None,
    });

    let tmp = TempDir::new()?;
    let db_path = tmp.path().join("graph.db");
    let storage = Storage::open(&db_path)?;
    let mut indexer = Indexer::new(storage, config);
    let stats = indexer.index(&root, true)?;

    assert_eq!(
        stats.nodes_extracted, 0,
        "Expected 0 Python nodes when roots restrict to nonexistent dir"
    );
    Ok(())
}

// ---------------------------------------------------------------------------
// Path normalization: absolute paths stored consistently
// ---------------------------------------------------------------------------

#[test]
fn test_indexed_file_paths_are_absolute() -> Result<()> {
    let root = fixtures_path("python_sample");
    if !root.exists() {
        return Ok(());
    }

    let tmp = TempDir::new()?;
    let db_path = tmp.path().join("graph.db");
    let storage = Storage::open(&db_path)?;
    let mut indexer = Indexer::new(storage, Config::default());
    // Index using a relative-style path if the fixture path is absolute,
    // otherwise use as-is; either way all stored paths must be absolute.
    indexer.index(&root, true)?;

    let storage = Storage::open(&db_path)?;
    let files = storage.list_files()?;
    assert!(!files.is_empty(), "Expected indexed files");
    for f in &files {
        assert!(
            std::path::Path::new(f).is_absolute(),
            "Stored file path should be absolute, got: {}",
            f
        );
    }
    Ok(())
}
