use anyhow::{Context, Result};
use spy_core::{FileContext, Language};
use std::path::Path;
use tree_sitter::Parser;

pub fn parse_file(path: &Path, source: Vec<u8>, language: Language) -> Result<FileContext> {
    let mut parser = Parser::new();

    let ts_lang = match language {
        Language::Rust => tree_sitter_rust::LANGUAGE.into(),
        Language::Python => tree_sitter_python::LANGUAGE.into(),
        Language::TypeScript => tree_sitter_typescript::LANGUAGE_TYPESCRIPT.into(),
        Language::JavaScript => tree_sitter_javascript::LANGUAGE.into(),
        Language::Go => tree_sitter_go::LANGUAGE.into(),
    };

    parser
        .set_language(&ts_lang)
        .context("Failed to set parser language")?;

    let tree = parser
        .parse(&source, None)
        .context("Failed to parse source")?;

    Ok(FileContext {
        tree,
        source,
        path: path.to_path_buf(),
        language,
    })
}

pub fn node_text<'a>(node: &tree_sitter::Node, source: &'a [u8]) -> &'a str {
    node.utf8_text(source).unwrap_or("")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_rust() -> Result<()> {
        let source = b"fn main() {}";
        let ctx = parse_file(Path::new("test.rs"), source.to_vec(), Language::Rust)?;
        assert_eq!(ctx.language, Language::Rust);
        assert!(ctx.tree.root_node().child_count() > 0);
        Ok(())
    }

    #[test]
    fn test_parse_python() -> Result<()> {
        let source = b"def hello(): pass";
        let ctx = parse_file(Path::new("test.py"), source.to_vec(), Language::Python)?;
        assert_eq!(ctx.language, Language::Python);
        assert!(ctx.tree.root_node().child_count() > 0);
        Ok(())
    }

    #[test]
    fn test_parse_typescript() -> Result<()> {
        let source = b"function greet(): void {}";
        let ctx = parse_file(Path::new("test.ts"), source.to_vec(), Language::TypeScript)?;
        assert_eq!(ctx.language, Language::TypeScript);
        assert!(ctx.tree.root_node().child_count() > 0);
        Ok(())
    }

    #[test]
    fn test_parse_javascript() -> Result<()> {
        let source = b"function foo() {}";
        let ctx = parse_file(Path::new("test.js"), source.to_vec(), Language::JavaScript)?;
        assert_eq!(ctx.language, Language::JavaScript);
        assert!(ctx.tree.root_node().child_count() > 0);
        Ok(())
    }

    #[test]
    fn test_parse_go() -> Result<()> {
        let source = b"package main\nfunc main() {}";
        let ctx = parse_file(Path::new("test.go"), source.to_vec(), Language::Go)?;
        assert_eq!(ctx.language, Language::Go);
        assert!(ctx.tree.root_node().child_count() > 0);
        Ok(())
    }
}
