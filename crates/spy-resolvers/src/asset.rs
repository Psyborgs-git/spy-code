use anyhow::Result;
use spy_core::{Edge, FileContext, Language, Node, NodeId, NodeKind, ProjectScope, Resolver};

pub struct AssetResolver {
    lang: Language,
}

impl AssetResolver {
    pub fn new(lang: Language) -> Self {
        Self { lang }
    }
}

impl Resolver for AssetResolver {
    fn language(&self) -> Language {
        self.lang
    }

    fn extensions(&self) -> &[&str] {
        &[]
    }

    fn extract_nodes(&self, ctx: &FileContext) -> Result<Vec<Node>> {
        let dir = ctx.path.parent().and_then(|p| p.to_str()).unwrap_or(".");
        let file = ctx.path.file_name().and_then(|f| f.to_str()).unwrap_or("_");

        let node_id = NodeId::new(dir, file, "_", file)?;

        // For text files, the description could be the actual text.
        // For other files, it could be empty or extracted by a plugin/api later.
        // The indexer's job is just to extract the node, so we use the raw bytes as description
        // if it's text-like and valid UTF-8. Otherwise just a generic description.
        let description = if matches!(self.lang, Language::Markdown | Language::Text) {
            String::from_utf8(ctx.source.clone()).ok()
        } else {
            None
        };

        let content_hash = blake3::hash(&ctx.source).to_hex().to_string();

        let node = Node {
            node_id,
            kind: NodeKind::Asset,
            name: file.to_string(),
            description,
            signatures: vec![],
            language: self.lang,
            file_path: ctx.path.to_string_lossy().to_string(),
            start_line: 1,
            end_line: 1,
            content_hash,
            git_sha: None,
            renamed_from: None,
        };

        Ok(vec![node])
    }

    fn extract_edges(&self, _ctx: &FileContext, _scope: &ProjectScope) -> Result<Vec<Edge>> {
        // Assets don't currently have outgoing edges (they don't import or call things)
        Ok(vec![])
    }
}
