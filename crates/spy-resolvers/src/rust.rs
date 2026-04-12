use anyhow::Result;
use spy_core::{
    Edge, EdgeKind, FileContext, Language, Node, NodeId, NodeKind, Param, ProjectScope, Resolver,
    Signature,
};
use tree_sitter::Node as TSNode;

pub struct RustResolver;

impl Resolver for RustResolver {
    fn language(&self) -> Language {
        Language::Rust
    }

    fn extensions(&self) -> &[&str] {
        &["rs"]
    }

    fn extract_nodes(&self, ctx: &FileContext) -> Result<Vec<Node>> {
        let mut nodes = Vec::new();
        let root = ctx.tree.root_node();

        let dir = ctx
            .path
            .parent()
            .and_then(|p| p.to_str())
            .unwrap_or(".");
        let file = ctx.path.file_name().and_then(|f| f.to_str()).unwrap_or("_");

        walk_nodes(&root, &ctx.source, dir, file, "_", &mut nodes, &ctx)?;

        Ok(nodes)
    }

    fn extract_edges(&self, ctx: &FileContext, scope: &ProjectScope) -> Result<Vec<Edge>> {
        let mut edges = Vec::new();
        let root = ctx.tree.root_node();

        walk_for_edges(&root, &ctx.source, ctx, scope, &mut edges)?;

        Ok(edges)
    }
}

fn walk_nodes(
    node: &TSNode,
    source: &[u8],
    dir: &str,
    file: &str,
    parent_class: &str,
    nodes: &mut Vec<Node>,
    ctx: &FileContext,
) -> Result<()> {
    match node.kind() {
        "function_item" => {
            if let Some(name_node) = node.child_by_field_name("name") {
                let name = node_text(&name_node, source);
                let description = extract_doc_comment(node, source);
                let signatures = extract_function_signature(node, source);
                let content_hash = compute_hash(node, source);

                let node_id = NodeId::new(dir, file, parent_class, name)?;

                nodes.push(Node {
                    node_id,
                    kind: NodeKind::Function,
                    name: name.to_string(),
                    description,
                    signatures: vec![signatures],
                    language: Language::Rust,
                    file_path: ctx.path.to_string_lossy().to_string(),
                    start_line: node.start_position().row as u32 + 1,
                    end_line: node.end_position().row as u32 + 1,
                    content_hash,
                    git_sha: None,
                    renamed_from: None,
                });
            }
        }
        "struct_item" | "enum_item" | "trait_item" => {
            if let Some(name_node) = node.child_by_field_name("name") {
                let name = node_text(&name_node, source);
                let description = extract_doc_comment(node, source);
                let content_hash = compute_hash(node, source);

                let node_id = NodeId::new(dir, file, "_", name)?;

                nodes.push(Node {
                    node_id,
                    kind: NodeKind::Class,
                    name: name.to_string(),
                    description,
                    signatures: vec![],
                    language: Language::Rust,
                    file_path: ctx.path.to_string_lossy().to_string(),
                    start_line: node.start_position().row as u32 + 1,
                    end_line: node.end_position().row as u32 + 1,
                    content_hash,
                    git_sha: None,
                    renamed_from: None,
                });
            }
        }
        "impl_item" => {
            if let Some(type_node) = node.child_by_field_name("type") {
                let type_name = node_text(&type_node, source);
                let class_name = if let Some(trait_node) = node.child_by_field_name("trait") {
                    let trait_name = node_text(&trait_node, source);
                    format!("{}<{}>", type_name, trait_name)
                } else {
                    type_name.to_string()
                };

                if let Some(body) = node.child_by_field_name("body") {
                    let mut cursor = body.walk();
                    for child in body.children(&mut cursor) {
                        if child.kind() == "function_item" {
                            if let Some(name_node) = child.child_by_field_name("name") {
                                let name = node_text(&name_node, source);
                                let description = extract_doc_comment(&child, source);
                                let signatures = extract_function_signature(&child, source);
                                let content_hash = compute_hash(&child, source);

                                let node_id = NodeId::new(dir, file, &class_name, name)?;

                                nodes.push(Node {
                                    node_id,
                                    kind: NodeKind::Function,
                                    name: name.to_string(),
                                    description,
                                    signatures: vec![signatures],
                                    language: Language::Rust,
                                    file_path: ctx.path.to_string_lossy().to_string(),
                                    start_line: child.start_position().row as u32 + 1,
                                    end_line: child.end_position().row as u32 + 1,
                                    content_hash,
                                    git_sha: None,
                                    renamed_from: None,
                                });
                            }
                        }
                    }
                }
            }
        }
        "const_item" | "static_item" => {
            if let Some(name_node) = node.child_by_field_name("name") {
                let name = node_text(&name_node, source);
                let description = extract_doc_comment(node, source);
                let content_hash = compute_hash(node, source);

                let node_id = NodeId::new(dir, file, "_", name)?;

                nodes.push(Node {
                    node_id,
                    kind: NodeKind::Constant,
                    name: name.to_string(),
                    description,
                    signatures: vec![],
                    language: Language::Rust,
                    file_path: ctx.path.to_string_lossy().to_string(),
                    start_line: node.start_position().row as u32 + 1,
                    end_line: node.end_position().row as u32 + 1,
                    content_hash,
                    git_sha: None,
                    renamed_from: None,
                });
            }
        }
        _ => {}
    }

    let mut cursor = node.walk();
    for child in node.children(&mut cursor) {
        walk_nodes(&child, source, dir, file, parent_class, nodes, ctx)?;
    }

    Ok(())
}

fn walk_for_edges(
    node: &TSNode,
    source: &[u8],
    ctx: &FileContext,
    scope: &ProjectScope,
    edges: &mut Vec<Edge>,
) -> Result<()> {
    if node.kind() == "call_expression" {
        if let Some(func_node) = node.child_by_field_name("function") {
            let func_text = node_text(&func_node, source);
            let from_id = infer_containing_function(node, source, ctx)?;

            if let Some(from_id) = from_id {
                let candidates = scope.find_nodes_by_name(func_text);
                if candidates.len() == 1 {
                    edges.push(Edge {
                        from_id,
                        to_id: candidates[0].node_id.clone(),
                        kind: EdgeKind::Calls,
                        confidence: 1.0,
                    });
                } else if !candidates.is_empty() {
                    edges.push(Edge {
                        from_id,
                        to_id: candidates[0].node_id.clone(),
                        kind: EdgeKind::Calls,
                        confidence: 0.4,
                    });
                }
            }
        }
    }

    let mut cursor = node.walk();
    for child in node.children(&mut cursor) {
        walk_for_edges(&child, source, ctx, scope, edges)?;
    }

    Ok(())
}

fn infer_containing_function(
    node: &TSNode,
    source: &[u8],
    ctx: &FileContext,
) -> Result<Option<NodeId>> {
    let mut current = node.parent();
    let dir = ctx
        .path
        .parent()
        .and_then(|p| p.to_str())
        .unwrap_or(".");
    let file = ctx.path.file_name().and_then(|f| f.to_str()).unwrap_or("_");

    while let Some(parent) = current {
        if parent.kind() == "function_item" {
            if let Some(name_node) = parent.child_by_field_name("name") {
                let name = node_text(&name_node, source);
                return Ok(Some(NodeId::new(dir, file, "_", name)?));
            }
        } else if parent.kind() == "impl_item" {
            if let Some(type_node) = parent.child_by_field_name("type") {
                let type_name = node_text(&type_node, source);

                if let Some(body) = parent.child_by_field_name("body") {
                    let mut func_parent = *node;
                    while let Some(p) = func_parent.parent() {
                        if p.kind() == "function_item" && p.parent().map(|pp| pp.id()) == Some(body.id()) {
                            if let Some(name_node) = p.child_by_field_name("name") {
                                let name = node_text(&name_node, source);
                                return Ok(Some(NodeId::new(dir, file, type_name, name)?));
                            }
                        }
                        func_parent = p;
                    }
                }
            }
        }
        current = parent.parent();
    }

    Ok(None)
}

fn extract_doc_comment(node: &TSNode, source: &[u8]) -> Option<String> {
    let mut comments = Vec::new();
    let start_row = node.start_position().row;

    if let Some(parent) = node.parent() {
        let mut cursor = parent.walk();
        for sibling in parent.children(&mut cursor) {
            if sibling.kind() == "line_comment" && sibling.end_position().row < start_row {
                let text = node_text(&sibling, source);
                if let Some(stripped) = text.strip_prefix("///") {
                    comments.push(stripped.trim().to_string());
                }
            }
        }
    }

    if comments.is_empty() {
        None
    } else {
        Some(comments.join(" "))
    }
}

fn extract_function_signature(node: &TSNode, source: &[u8]) -> Signature {
    let mut params = Vec::new();

    if let Some(params_node) = node.child_by_field_name("parameters") {
        let mut cursor = params_node.walk();
        for child in params_node.children(&mut cursor) {
            if child.kind() == "parameter" {
                let name = child
                    .child_by_field_name("pattern")
                    .map(|n| node_text(&n, source).to_string())
                    .unwrap_or_else(|| "_".to_string());

                let type_ = child
                    .child_by_field_name("type")
                    .map(|n| node_text(&n, source).to_string());

                params.push(Param { name, type_ });
            }
        }
    }

    let returns = node
        .child_by_field_name("return_type")
        .map(|n| node_text(&n, source).to_string());

    Signature { params, returns }
}

fn compute_hash(node: &TSNode, source: &[u8]) -> String {
    let start = node.start_byte();
    let end = node.end_byte();
    let slice = &source[start..end];
    let hash = blake3::hash(slice);
    hash.to_hex().to_string()
}

fn node_text<'a>(node: &TSNode, source: &'a [u8]) -> &'a str {
    node.utf8_text(source).unwrap_or("")
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::Path;

    #[test]
    fn test_extract_function() -> Result<()> {
        let source = b"fn test() {}";
        let ctx = spy_parser::parse_file(Path::new("test.rs"), source.to_vec(), Language::Rust)?;

        let resolver = RustResolver;
        let nodes = resolver.extract_nodes(&ctx)?;

        assert_eq!(nodes.len(), 1);
        assert_eq!(nodes[0].name, "test");
        assert_eq!(nodes[0].kind, NodeKind::Function);

        Ok(())
    }

    #[test]
    fn test_extract_struct() -> Result<()> {
        let source = b"struct Foo {}";
        let ctx = spy_parser::parse_file(Path::new("test.rs"), source.to_vec(), Language::Rust)?;

        let resolver = RustResolver;
        let nodes = resolver.extract_nodes(&ctx)?;

        assert_eq!(nodes.len(), 1);
        assert_eq!(nodes[0].name, "Foo");
        assert_eq!(nodes[0].kind, NodeKind::Class);

        Ok(())
    }
}
