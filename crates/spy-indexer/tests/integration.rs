use anyhow::Result;
use spy_core::{Config, Language, LanguageConfig};
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

fn write_file(path: &Path, contents: &str) {
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent).unwrap();
    }
    std::fs::write(path, contents).unwrap();
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

#[test]
fn test_index_respects_language_enable_flags() -> Result<()> {
    let project = TempDir::new()?;
    write_file(&project.path().join("src/lib.rs"), "fn keep() {}");
    write_file(&project.path().join("src/app.py"), "def skip():\n    pass\n");

    let (_dir, storage) = make_storage();
    let mut config = Config::default();
    config.languages.python = Some(LanguageConfig {
        enabled: false,
        ..LanguageConfig::default()
    });
    config.languages.typescript = Some(LanguageConfig {
        enabled: false,
        ..LanguageConfig::default()
    });
    config.languages.go = Some(LanguageConfig {
        enabled: false,
        ..LanguageConfig::default()
    });

    let mut indexer = Indexer::new(storage, config);
    let stats = indexer.index(project.path(), true)?;

    assert_eq!(stats.files_parsed, 1);
    Ok(())
}

#[test]
fn test_index_respects_configured_roots_and_ignores() -> Result<()> {
    let project = TempDir::new()?;
    write_file(&project.path().join("src/lib.rs"), "fn keep() {}");
    write_file(&project.path().join("generated/skip.rs"), "fn skip() {}");
    write_file(
        &project.path().join("src/ignored/nested.rs"),
        "fn ignored() {}",
    );

    let (_dir, storage) = make_storage();
    let mut config = Config::default();
    config.languages.rust = Some(LanguageConfig {
        roots: vec!["src".to_string()],
        ignore: vec!["src/ignored/".to_string()],
        ..LanguageConfig::default()
    });
    config.languages.python = Some(LanguageConfig {
        enabled: false,
        ..LanguageConfig::default()
    });
    config.languages.typescript = Some(LanguageConfig {
        enabled: false,
        ..LanguageConfig::default()
    });
    config.languages.go = Some(LanguageConfig {
        enabled: false,
        ..LanguageConfig::default()
    });

    let mut indexer = Indexer::new(storage, config);
    let stats = indexer.index(project.path(), true)?;

    assert_eq!(stats.files_parsed, 1);
    Ok(())
}
