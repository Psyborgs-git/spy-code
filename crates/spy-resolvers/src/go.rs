use anyhow::Result;
use spy_core::{
    Edge, EdgeKind, FileContext, Language, Node, NodeId, NodeKind, Param, ProjectScope, Resolver,
    Signature,
};
use tree_sitter::Node as TSNode;

pub struct GoResolver;

impl Resolver for GoResolver {
    fn language(&self) -> Language {
        Language::Go
    }

    fn extensions(&self) -> &[&str] {
        &["go"]
    }

    fn extract_nodes(&self, ctx: &FileContext) -> Result<Vec<Node>> {
        let mut nodes = Vec::new();
        let root = ctx.tree.as_ref().unwrap().root_node();

        let dir = ctx.path.parent().and_then(|p| p.to_str()).unwrap_or(".");
        let file = ctx.path.file_name().and_then(|f| f.to_str()).unwrap_or("_");

        walk_nodes(&root, &ctx.source, dir, file, &mut nodes, ctx)?;
        Ok(nodes)
    }

    fn extract_edges(&self, ctx: &FileContext, scope: &ProjectScope) -> Result<Vec<Edge>> {
        let mut edges = Vec::new();
        let root = ctx.tree.as_ref().unwrap().root_node();
        walk_for_edges(&root, &ctx.source, ctx, scope, &mut edges)?;
        Ok(edges)
    }
}

// ---------------------------------------------------------------------------
// Node extraction
// ---------------------------------------------------------------------------

fn walk_nodes(
    node: &TSNode,
    source: &[u8],
    dir: &str,
    file: &str,
    nodes: &mut Vec<Node>,
    ctx: &FileContext,
) -> Result<()> {
    match node.kind() {
        "function_declaration" => {
            if let Some(name_node) = node.child_by_field_name("name") {
                let name = node_text(&name_node, source);
                let description = extract_preceding_comments(node, source);
                let sig = extract_function_signature(node, source);
                let content_hash = compute_hash(node, source);
                let node_id = NodeId::new(dir, file, "_", name)?;
                nodes.push(Node {
                    node_id,
                    kind: NodeKind::Function,
                    name: name.to_string(),
                    description,
                    signatures: vec![sig],
                    language: Language::Go,
                    file_path: ctx.path.to_string_lossy().to_string(),
                    start_line: node.start_position().row as u32 + 1,
                    end_line: node.end_position().row as u32 + 1,
                    content_hash,
                    git_sha: None,
                    renamed_from: None,
                });
            }
        }
        "method_declaration" => {
            if let Some(name_node) = node.child_by_field_name("name") {
                let name = node_text(&name_node, source);
                let description = extract_preceding_comments(node, source);
                let sig = extract_function_signature(node, source);
                let content_hash = compute_hash(node, source);

                // Receiver type becomes the "class" component
                let receiver_type = extract_receiver_type(node, source);
                let node_id = NodeId::new(dir, file, &receiver_type, name)?;
                nodes.push(Node {
                    node_id,
                    kind: NodeKind::Function,
                    name: name.to_string(),
                    description,
                    signatures: vec![sig],
                    language: Language::Go,
                    file_path: ctx.path.to_string_lossy().to_string(),
                    start_line: node.start_position().row as u32 + 1,
                    end_line: node.end_position().row as u32 + 1,
                    content_hash,
                    git_sha: None,
                    renamed_from: None,
                });
            }
        }
        "type_declaration" => {
            // type Foo struct {...} or type Bar interface {...}
            let mut cursor = node.walk();
            for child in node.children(&mut cursor) {
                if child.kind() == "type_spec" {
                    if let Some(name_node) = child.child_by_field_name("name") {
                        let name = node_text(&name_node, source);
                        if let Some(type_node) = child.child_by_field_name("type") {
                            if matches!(type_node.kind(), "struct_type" | "interface_type") {
                                let description = extract_preceding_comments(node, source);
                                let content_hash = compute_hash(node, source);
                                let node_id = NodeId::new(dir, file, "_", name)?;
                                nodes.push(Node {
                                    node_id,
                                    kind: NodeKind::Class,
                                    name: name.to_string(),
                                    description,
                                    signatures: vec![],
                                    language: Language::Go,
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
            }
        }
        "const_declaration" => {
            // Package-level const
            let mut cursor = node.walk();
            for spec in node.children(&mut cursor) {
                if spec.kind() == "const_spec" {
                    if let Some(name_node) = spec.child_by_field_name("name") {
                        let name = node_text(&name_node, source);
                        let description = extract_preceding_comments(node, source);
                        let content_hash = compute_hash(node, source);
                        let node_id = NodeId::new(dir, file, "_", name)?;
                        nodes.push(Node {
                            node_id,
                            kind: NodeKind::Constant,
                            name: name.to_string(),
                            description,
                            signatures: vec![],
                            language: Language::Go,
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
        walk_nodes(&child, source, dir, file, nodes, ctx)?;
    }

    Ok(())
}

// ---------------------------------------------------------------------------
// Edge extraction
// ---------------------------------------------------------------------------

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
            // Strip package qualifier: fmt.Println → Println
            let bare = func_text.split('.').next_back().unwrap_or(func_text);
            if let Some(from_id) = infer_containing_function(node, source, ctx)? {
                let candidates = scope.find_nodes_by_name(bare);
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
    let dir = ctx.path.parent().and_then(|p| p.to_str()).unwrap_or(".");
    let file = ctx.path.file_name().and_then(|f| f.to_str()).unwrap_or("_");

    let mut current = node.parent();
    while let Some(parent) = current {
        match parent.kind() {
            "function_declaration" => {
                if let Some(n) = parent.child_by_field_name("name") {
                    let name = node_text(&n, source);
                    return Ok(Some(NodeId::new(dir, file, "_", name)?));
                }
            }
            "method_declaration" => {
                if let Some(n) = parent.child_by_field_name("name") {
                    let name = node_text(&n, source);
                    let receiver = extract_receiver_type(&parent, source);
                    return Ok(Some(NodeId::new(dir, file, &receiver, name)?));
                }
            }
            _ => {}
        }
        current = parent.parent();
    }

    Ok(None)
}

// ---------------------------------------------------------------------------
// Helpers
// ---------------------------------------------------------------------------

fn extract_receiver_type(node: &TSNode, source: &[u8]) -> String {
    if let Some(recv) = node.child_by_field_name("receiver") {
        // parameter_list → parameter_declaration → type
        let mut cursor = recv.walk();
        for child in recv.children(&mut cursor) {
            if child.kind() == "parameter_declaration" {
                if let Some(t) = child.child_by_field_name("type") {
                    let raw = node_text(&t, source);
                    // Strip pointer: *Foo → Foo
                    return raw.trim_start_matches('*').to_string();
                }
            }
        }
    }
    "_".to_string()
}

fn extract_preceding_comments(node: &TSNode, source: &[u8]) -> Option<String> {
    let start_row = node.start_position().row;
    let parent = node.parent()?;
    let mut comments = Vec::new();

    let mut cursor = parent.walk();
    for sibling in parent.children(&mut cursor) {
        if sibling.kind() == "comment" && sibling.end_position().row < start_row {
            let text = node_text(&sibling, source);
            let stripped = text.trim_start_matches("//").trim();
            if !stripped.is_empty() {
                comments.push(stripped.to_string());
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
            if child.kind() == "parameter_declaration" {
                let type_ = child
                    .child_by_field_name("type")
                    .map(|n| node_text(&n, source).to_string());

                // A parameter_declaration can have multiple names: a, b int
                let mut c2 = child.walk();
                for nc in child.children(&mut c2) {
                    if nc.kind() == "identifier" {
                        params.push(Param {
                            name: node_text(&nc, source).to_string(),
                            type_: type_.clone(),
                        });
                    }
                }
            }
        }
    }

    // Return type(s): could be a single type or (type, error)
    let returns = node
        .child_by_field_name("result")
        .map(|n| node_text(&n, source).to_string());

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
        spy_parser::parse_file(Path::new("test.go"), source.to_vec(), Language::Go).unwrap()
    }

    #[test]
    fn test_go_function() {
        let ctx = parse(b"package main\nfunc Hello(name string) string { return name }");
        let nodes = GoResolver.extract_nodes(&ctx).unwrap();
        assert!(nodes
            .iter()
            .any(|n| n.name == "Hello" && n.kind == NodeKind::Function));
    }

    #[test]
    fn test_go_method() {
        let ctx = parse(b"package main\ntype Foo struct{}\nfunc (f *Foo) Bar() {}");
        let nodes = GoResolver.extract_nodes(&ctx).unwrap();
        assert!(nodes
            .iter()
            .any(|n| n.name == "Bar" && n.kind == NodeKind::Function));
    }

    #[test]
    fn test_go_struct() {
        let ctx = parse(b"package main\ntype Server struct { addr string }");
        let nodes = GoResolver.extract_nodes(&ctx).unwrap();
        assert!(nodes
            .iter()
            .any(|n| n.name == "Server" && n.kind == NodeKind::Class));
    }
}
