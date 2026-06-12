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
        let root = ctx.tree.root_node();

        let dir = ctx.path.parent().and_then(|p| p.to_str()).unwrap_or(".");
        let file = ctx.path.file_name().and_then(|f| f.to_str()).unwrap_or("_");

        walk_nodes(&root, source, dir, file, "_", &mut nodes, ctx)?;

        Ok(nodes)
    }

    fn extract_edges(&self, ctx: &FileContext, scope: &ProjectScope) -> Result<Vec<Edge>> {
        let mut edges = Vec::new();
        let source = &ctx.source;
        let root = ctx.tree.root_node();

        walk_for_edges(&root, source, ctx, scope, &mut edges)?;

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
        "class_declaration" | "interface_declaration" | "enum_declaration" => {
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
                    language: Language::Java,
                    file_path: ctx.path.to_string_lossy().to_string(),
                    start_line: node.start_position().row as u32 + 1,
                    end_line: node.end_position().row as u32 + 1,
                    content_hash,
                    git_sha: None,
                    renamed_from: None,
                });

                if let Some(body) = node.child_by_field_name("body") {
                    let mut cursor = body.walk();
                    for child in body.children(&mut cursor) {
                        walk_nodes(&child, source, dir, file, name, nodes, ctx)?;
                    }
                }
                return Ok(());
            }
        }
        "method_declaration" => {
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
        "field_declaration" | "constant_declaration" => {
            if is_static_final(node, source) {
                if let Some(declarator) = node.child_by_field_name("declarator") {
                    if let Some(name_node) = declarator.child_by_field_name("name") {
                        let name = node_text(&name_node, source);
                        let description = extract_doc_comment(node, source);
                        let content_hash = compute_hash(node, source);

                        let node_id = NodeId::new(dir, file, parent_class, name)?;

                        nodes.push(Node {
                            node_id,
                            kind: NodeKind::Constant,
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
    if node.kind() == "method_invocation" {
        if let Some(func_node) = node.child_by_field_name("name") {
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
    let dir = ctx.path.parent().and_then(|p| p.to_str()).unwrap_or(".");
    let file = ctx.path.file_name().and_then(|f| f.to_str()).unwrap_or("_");

    let mut method_name = None;
    let mut class_name = "_";

    while let Some(parent) = current {
        if parent.kind() == "method_declaration" {
            if let Some(name_node) = parent.child_by_field_name("name") {
                method_name = Some(node_text(&name_node, source));
            }
        } else if parent.kind() == "class_declaration" || parent.kind() == "interface_declaration" || parent.kind() == "enum_declaration" {
            if let Some(name_node) = parent.child_by_field_name("name") {
                class_name = node_text(&name_node, source);
            }
        }
        current = parent.parent();
    }

    if let Some(name) = method_name {
        return Ok(Some(NodeId::new(dir, file, class_name, name)?));
    }

    Ok(None)
}

fn extract_doc_comment(node: &TSNode, source: &[u8]) -> Option<String> {
    let mut comments = Vec::new();
    let start_row = node.start_position().row;

    if let Some(parent) = node.parent() {
        let mut cursor = parent.walk();
        for sibling in parent.children(&mut cursor) {
            if sibling.kind() == "block_comment" && sibling.end_position().row < start_row {
                let text = node_text(&sibling, source);
                if text.starts_with("/**") {
                    let mut cleaned = String::new();
                    for line in text.lines() {
                        let trimmed = line.trim();
                        let no_star = trimmed.strip_prefix('*').unwrap_or(trimmed).trim();
                        if !no_star.is_empty() && no_star != "/**" && no_star != "/" {
                            cleaned.push_str(no_star);
                            cleaned.push(' ');
                        }
                    }
                    comments.push(cleaned.trim().to_string());
                }
            } else if sibling.kind() == "line_comment" && sibling.end_position().row < start_row {
                let text = node_text(&sibling, source);
                if let Some(stripped) = text.strip_prefix("//") {
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
            if child.kind() == "formal_parameter" {
                let name = child
                    .child_by_field_name("name")
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
        .child_by_field_name("type")
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

fn is_static_final(node: &TSNode, source: &[u8]) -> bool {
    let mut is_static = false;
    let mut is_final = false;

    let mut cursor = node.walk();
    for child in node.children(&mut cursor) {
        if child.kind() == "modifiers" {
            let mut mod_cursor = child.walk();
            for modifier in child.children(&mut mod_cursor) {
                let text = node_text(&modifier, source);
                if text == "static" {
                    is_static = true;
                } else if text == "final" {
                    is_final = true;
                }
            }
        }
    }

    is_static && is_final
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
        let source = b"class Main { void test() {} }";
        let ctx = spy_parser::parse_file(Path::new("test.java"), source.to_vec(), Language::Java)?;

        let resolver = JavaResolver;
        let nodes = resolver.extract_nodes(&ctx)?;

        assert!(nodes.iter().any(|n| n.name == "test" && n.kind == NodeKind::Function));

        Ok(())
    }

    #[test]
    fn test_extract_class() -> Result<()> {
        let source = b"class Foo {}";
        let ctx = spy_parser::parse_file(Path::new("test.java"), source.to_vec(), Language::Java)?;

        let resolver = JavaResolver;
        let nodes = resolver.extract_nodes(&ctx)?;

        assert_eq!(nodes.len(), 1);
        assert_eq!(nodes[0].name, "Foo");
        assert_eq!(nodes[0].kind, NodeKind::Class);

        Ok(())
    }
}
