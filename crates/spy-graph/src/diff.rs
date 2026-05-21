use async_graphql::*;
use spy_core::{Language, Node};
use std::path::Path;

#[derive(Enum, Copy, Clone, Eq, PartialEq)]
pub enum ChangeType {
    Added,
    Deleted,
    SignatureModified,
    BodyModified,
}

#[derive(SimpleObject)]
pub struct SignatureDiff {
    pub change_type: ChangeType,
    pub name: String,
    pub old_signature: Option<String>,
    pub new_signature: Option<String>,
}

fn format_signature(node: &Node) -> String {
    if node.signatures.is_empty() {
        return "()".to_string();
    }
    let sig = &node.signatures[0];
    let params = sig
        .params
        .iter()
        .map(|p| {
            if let Some(t) = &p.type_ {
                format!("{}: {}", p.name, t)
            } else {
                p.name.clone()
            }
        })
        .collect::<Vec<_>>()
        .join(", ");

    if let Some(ret) = &sig.returns {
        format!("({}) -> {}", params, ret)
    } else {
        format!("({})", params)
    }
}

pub fn compute_diff(
    path: &Path,
    language: Language,
    old_source: &[u8],
    new_source: &[u8],
) -> anyhow::Result<Vec<SignatureDiff>> {
    let resolver = spy_resolvers::get_resolver(language)
        .ok_or_else(|| anyhow::anyhow!("Unsupported language"))?;

    let old_ctx = spy_parser::parse_file(path, old_source.to_vec(), language)?;
    let new_ctx = spy_parser::parse_file(path, new_source.to_vec(), language)?;

    let old_nodes = resolver.extract_nodes(&old_ctx)?;
    let new_nodes = resolver.extract_nodes(&new_ctx)?;

    let mut diffs = Vec::new();

    for new_node in &new_nodes {
        let old_match = old_nodes.iter().find(|n| n.node_id == new_node.node_id);

        if let Some(old_node) = old_match {
            let old_sig = format_signature(old_node);
            let new_sig = format_signature(new_node);

            let change_type = if old_sig != new_sig {
                ChangeType::SignatureModified
            } else if old_node.content_hash != new_node.content_hash {
                ChangeType::BodyModified
            } else {
                continue;
            };

            diffs.push(SignatureDiff {
                change_type,
                name: new_node.name.clone(),
                old_signature: Some(old_sig),
                new_signature: Some(new_sig),
            });
        } else {
            diffs.push(SignatureDiff {
                change_type: ChangeType::Added,
                name: new_node.name.clone(),
                old_signature: None,
                new_signature: Some(format_signature(new_node)),
            });
        }
    }

    for old_node in &old_nodes {
        let new_match = new_nodes.iter().find(|n| n.node_id == old_node.node_id);
        if new_match.is_none() {
            diffs.push(SignatureDiff {
                change_type: ChangeType::Deleted,
                name: old_node.name.clone(),
                old_signature: Some(format_signature(old_node)),
                new_signature: None,
            });
        }
    }

    Ok(diffs)
}
