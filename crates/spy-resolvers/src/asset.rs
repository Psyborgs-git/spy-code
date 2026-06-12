use anyhow::Result;
use spy_core::{Edge, FileContext, Language, Node, NodeId, NodeKind, ProjectScope, Resolver};

pub struct AssetResolver {
    language: Language,
}

impl AssetResolver {
    pub fn new(language: Language) -> Self {
        Self { language }
    }
}

impl Resolver for AssetResolver {
    fn language(&self) -> Language {
        self.language
    }

    fn extensions(&self) -> &[&str] {
        &[]
    }

    fn extract_nodes(&self, ctx: &FileContext) -> Result<Vec<Node>> {
        let dir = ctx
            .path
            .parent()
            .and_then(|p| p.to_str())
            .unwrap_or("");

        let file_name = ctx
            .path
            .file_name()
            .and_then(|n| n.to_str())
            .unwrap_or("");

        let extension = ctx
            .path
            .extension()
            .and_then(|e| e.to_str())
            .unwrap_or("");

        let node_id = NodeId::new(dir, file_name, "asset", extension)?;

        let mut description = None;
        if self.language == Language::Markdown || self.language == Language::Text {
            if let Ok(text) = std::str::from_utf8(&ctx.source) {
                description = Some(text.to_string());
            }
        }

        let node = Node {
            node_id,
            kind: NodeKind::Asset,
            name: file_name.to_string(),
            description,
            signatures: vec![],
            language: self.language,
            file_path: ctx.path.to_string_lossy().to_string(),
            start_line: 1,
            end_line: 1,
            content_hash: "".to_string(),
            git_sha: None,
            renamed_from: None,
        };

        Ok(vec![node])
    }

    fn extract_edges(&self, _ctx: &FileContext, _scope: &ProjectScope) -> Result<Vec<Edge>> {
        Ok(vec![])
    }
}
