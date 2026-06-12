use anyhow::Result;
use spy_core::{Edge, EdgeKind, FileContext, Language, Node, NodeId, NodeKind, Param, ProjectScope, Resolver, Signature};
use tree_sitter::Node as TSNode;

pub struct JavaResolver;

impl Resolver for JavaResolver {
    fn language(&self) -> Language {
        Language::Java
    }

    fn extensions(&self) -> &[&str] {
        &["java"]
    }

    fn extract_nodes(&self, ctx: &FileContext) -> Result<Vec<Node>> {
        let mut nodes = Vec::new();
        let source = &ctx.source;
        let root = ctx.tree.as_ref().ok_or_else(|| anyhow::anyhow!("Tree is missing in FileContext"))?.root_node();
        let dir = ctx.path.parent().and_then(|p| p.to_str()).unwrap_or(".");
        let file = ctx.path.file_name().and_then(|f| f.to_str()).unwrap_or("_");

        walk_nodes(&root, source, dir, file, &mut nodes, ctx)?;

        Ok(nodes)
    }

    fn extract_edges(&self, ctx: &FileContext, scope: &ProjectScope) -> Result<Vec<Edge>> {
        let mut edges = Vec::new();
        let source = &ctx.source;
        let root = ctx.tree.as_ref().unwrap().root_node();

        walk_for_edges(&root, source, ctx, scope, &mut edges)?;

        Ok(edges)
    }
}

fn walk_nodes(
    node: &TSNode,
    source: &[u8],
    dir: &str,
    file: &str,
    nodes: &mut Vec<Node>,
    ctx: &FileContext,
) -> Result<()> {
    match node.kind() {
        "class_declaration" | "interface_declaration" | "enum_declaration" => {
            if let Some(name_node) = node.child_by_field_name("name") {
                let name = node_text(&name_node, source);
                let description = extract_preceding_comments(node, source);
                let content_hash = compute_hash(node, source);
                let node_id = NodeId::new(dir, file, "_", name)?;

                let kind = NodeKind::Class;

                nodes.push(Node {
                    node_id,
                    kind,
                    name: name.to_string(),
                    description,
                    signatures: vec![],
                    language: Language::Java,
                    file_path: ctx.path.to_string_lossy().to_string(),
                    start_line: node.start_position().row as u32 + 1,
                    end_line: node.end_position().row as u32 + 1,
                    content_hash,
                    git_sha: None,
                    renamed_from: None,
                });
            }
        }
        "method_declaration" | "constructor_declaration" => {
            if let Some(name_node) = node.child_by_field_name("name") {
                let name = node_text(&name_node, source);
                let parent_class = extract_parent_class(node, source).unwrap_or_else(|| "_".to_string());
                let description = extract_preceding_comments(node, source);
                let signature = extract_function_signature(node, source);
                let content_hash = compute_hash(node, source);
                let node_id = NodeId::new(dir, file, &parent_class, name)?;

                nodes.push(Node {
                    node_id,
                    kind: NodeKind::Function,
                    name: name.to_string(),
                    description,
                    signatures: vec![signature],
                    language: Language::Java,
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
        walk_nodes(&child, source, dir, file, nodes, ctx)?;
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
    if node.kind() == "method_invocation" {
        if let Some(name_node) = node.child_by_field_name("name") {
            let func_name = node_text(&name_node, source);
            if let Some(from_id) = infer_containing_function(node, source, ctx)? {
                let candidates = scope.find_nodes_by_name(func_name);
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
                        confidence: 0.5,
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

fn extract_parent_class(node: &TSNode, source: &[u8]) -> Option<String> {
    let mut current = node.parent();
    while let Some(parent) = current {
        if matches!(parent.kind(), "class_declaration" | "interface_declaration" | "enum_declaration") {
            if let Some(name_node) = parent.child_by_field_name("name") {
                return Some(node_text(&name_node, source).to_string());
            }
        }
        current = parent.parent();
    }
    None
}

fn infer_containing_function(
    node: &TSNode,
    source: &[u8],
    ctx: &FileContext,
) -> Result<Option<NodeId>> {
    let dir = ctx.path.parent().and_then(|p| p.to_str()).unwrap_or(".");
    let file = ctx.path.file_name().and_then(|f| f.to_str()).unwrap_or("_");

    let mut current = node.parent();
    while let Some(parent) = current {
        if matches!(parent.kind(), "method_declaration" | "constructor_declaration") {
            if let Some(n) = parent.child_by_field_name("name") {
                let name = node_text(&n, source);
                let class = extract_parent_class(&parent, source).unwrap_or_else(|| "_".to_string());
                return Ok(Some(NodeId::new(dir, file, &class, name)?));
            }
        }
        current = parent.parent();
    }

    Ok(None)
}

fn extract_preceding_comments(node: &TSNode, source: &[u8]) -> Option<String> {
    let mut comments = Vec::new();
    let mut current = node.prev_sibling();
    while let Some(sibling) = current {
        if sibling.kind() == "line_comment" || sibling.kind() == "block_comment" {
            let text = node_text(&sibling, source);
            let stripped = text.trim_start_matches("//").trim_start_matches("/*").trim_end_matches("*/").trim();
            if !stripped.is_empty() {
                comments.push(stripped.to_string());
            }
            current = sibling.prev_sibling();
        } else {
            break;
        }
    }

    if comments.is_empty() {
        None
    } else {
        comments.reverse();
        Some(comments.join(" "))
    }
}

fn extract_function_signature(node: &TSNode, source: &[u8]) -> Signature {
    let mut params = Vec::new();

    if let Some(params_node) = node.child_by_field_name("parameters") {
        let mut cursor = params_node.walk();
        for child in params_node.children(&mut cursor) {
            if child.kind() == "formal_parameter" {
                let type_ = child.child_by_field_name("type").map(|n| node_text(&n, source).to_string());
                let name = child.child_by_field_name("name").map(|n| node_text(&n, source).to_string()).unwrap_or_default();
                params.push(Param { name, type_ });
            }
        }
    }

    let returns = node.child_by_field_name("type").map(|n| node_text(&n, source).to_string());

    Signature { params, returns }
}

fn compute_hash(node: &TSNode, source: &[u8]) -> String {
    let slice = &source[node.start_byte()..node.end_byte()];
    blake3::hash(slice).to_hex().to_string()
}

fn node_text<'a>(node: &TSNode, source: &'a [u8]) -> &'a str {
    node.utf8_text(source).unwrap_or("")
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::Path;

    fn parse(source: &[u8]) -> FileContext {
        spy_parser::parse_file(Path::new("test.java"), source.to_vec(), Language::Java).unwrap()
    }

    #[test]
    fn test_java_class_and_method() {
        let ctx = parse(b"class Main { public static void main(String[] args) {} }");
        let nodes = JavaResolver.extract_nodes(&ctx).unwrap();
        assert!(nodes.iter().any(|n| n.name == "Main" && n.kind == NodeKind::Class));
        assert!(nodes.iter().any(|n| n.name == "main" && n.kind == NodeKind::Function));
    }
}
