use anyhow::Result;
use spy_core::{
    Edge, EdgeKind, FileContext, Language, Node, NodeId, NodeKind, Param, ProjectScope, Resolver,
    Signature,
};
use tree_sitter::Node as TSNode;

pub struct TypeScriptResolver;
pub struct JavaScriptResolver;

impl Resolver for TypeScriptResolver {
    fn language(&self) -> Language {
        Language::TypeScript
    }
    fn extensions(&self) -> &[&str] {
        &["ts", "tsx"]
    }
    fn extract_nodes(&self, ctx: &FileContext) -> Result<Vec<Node>> {
        extract_js_nodes(ctx)
    }
    fn extract_edges(&self, ctx: &FileContext, scope: &ProjectScope) -> Result<Vec<Edge>> {
        extract_js_edges(ctx, scope)
    }
}

impl Resolver for JavaScriptResolver {
    fn language(&self) -> Language {
        Language::JavaScript
    }
    fn extensions(&self) -> &[&str] {
        &["js", "jsx", "mjs", "cjs"]
    }
    fn extract_nodes(&self, ctx: &FileContext) -> Result<Vec<Node>> {
        extract_js_nodes(ctx)
    }
    fn extract_edges(&self, ctx: &FileContext, scope: &ProjectScope) -> Result<Vec<Edge>> {
        extract_js_edges(ctx, scope)
    }
}

// ---------------------------------------------------------------------------
// Node extraction
// ---------------------------------------------------------------------------

fn extract_js_nodes(ctx: &FileContext) -> Result<Vec<Node>> {
    let mut nodes = Vec::new();
    let root = ctx.tree.root_node();

    let dir = ctx
        .path
        .parent()
        .and_then(|p| p.to_str())
        .unwrap_or(".");
    let file = ctx.path.file_name().and_then(|f| f.to_str()).unwrap_or("_");

    walk_nodes(&root, &ctx.source, dir, file, "_", &mut nodes, ctx)?;

    // Collapse TypeScript function overloads (same name, consecutive declarations)
    collapse_overloads(&mut nodes);

    Ok(nodes)
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
        "function_declaration" | "generator_function_declaration" => {
            if let Some(name_node) = node.child_by_field_name("name") {
                let name = node_text(&name_node, source);
                let description = extract_jsdoc(node, source);
                let sig = extract_function_signature(node, source);
                let content_hash = compute_hash(node, source);
                let node_id = NodeId::new(dir, file, parent_class, name)?;
                nodes.push(Node {
                    node_id,
                    kind: NodeKind::Function,
                    name: name.to_string(),
                    description,
                    signatures: vec![sig],
                    language: ctx.language,
                    file_path: ctx.path.to_string_lossy().to_string(),
                    start_line: node.start_position().row as u32 + 1,
                    end_line: node.end_position().row as u32 + 1,
                    content_hash,
                    git_sha: None,
                    renamed_from: None,
                });
            }
        }
        "class_declaration" | "abstract_class_declaration" => {
            if let Some(name_node) = node.child_by_field_name("name") {
                let name = node_text(&name_node, source);
                let description = extract_jsdoc(node, source);
                let content_hash = compute_hash(node, source);
                let node_id = NodeId::new(dir, file, "_", name)?;
                nodes.push(Node {
                    node_id,
                    kind: NodeKind::Class,
                    name: name.to_string(),
                    description,
                    signatures: vec![],
                    language: ctx.language,
                    file_path: ctx.path.to_string_lossy().to_string(),
                    start_line: node.start_position().row as u32 + 1,
                    end_line: node.end_position().row as u32 + 1,
                    content_hash,
                    git_sha: None,
                    renamed_from: None,
                });
                // Walk class body
                if let Some(body) = node.child_by_field_name("body") {
                    let mut cursor = body.walk();
                    for child in body.children(&mut cursor) {
                        walk_nodes(&child, source, dir, file, name, nodes, ctx)?;
                    }
                }
                return Ok(());
            }
        }
        "method_definition" => {
            if let Some(name_node) = node.child_by_field_name("name") {
                let name = node_text(&name_node, source);
                let description = extract_jsdoc(node, source);
                let sig = extract_function_signature(node, source);
                let content_hash = compute_hash(node, source);
                let node_id = NodeId::new(dir, file, parent_class, name)?;
                nodes.push(Node {
                    node_id,
                    kind: NodeKind::Function,
                    name: name.to_string(),
                    description,
                    signatures: vec![sig],
                    language: ctx.language,
                    file_path: ctx.path.to_string_lossy().to_string(),
                    start_line: node.start_position().row as u32 + 1,
                    end_line: node.end_position().row as u32 + 1,
                    content_hash,
                    git_sha: None,
                    renamed_from: None,
                });
            }
        }
        "lexical_declaration" | "variable_declaration" => {
            // const FOO = ... or const foo = function() {} / () => {}
            let mut cursor = node.walk();
            for decl in node.children(&mut cursor) {
                if decl.kind() == "variable_declarator" {
                    if let Some(name_node) = decl.child_by_field_name("name") {
                        let name = node_text(&name_node, source);
                        if let Some(value) = decl.child_by_field_name("value") {
                            match value.kind() {
                                "arrow_function" | "function" | "generator_function" => {
                                    let description = extract_jsdoc(node, source);
                                    let sig = extract_function_signature(&value, source);
                                    let content_hash = compute_hash(&value, source);
                                    let node_id = NodeId::new(dir, file, parent_class, name)?;
                                    nodes.push(Node {
                                        node_id,
                                        kind: NodeKind::Function,
                                        name: name.to_string(),
                                        description,
                                        signatures: vec![sig],
                                        language: ctx.language,
                                        file_path: ctx.path.to_string_lossy().to_string(),
                                        start_line: node.start_position().row as u32 + 1,
                                        end_line: node.end_position().row as u32 + 1,
                                        content_hash,
                                        git_sha: None,
                                        renamed_from: None,
                                    });
                                }
                                _ => {
                                    // Only emit as Constant if it's a literal and top-level
                                    if parent_class == "_" && is_literal(&value) {
                                        let description = extract_jsdoc(node, source);
                                        let content_hash = compute_hash(node, source);
                                        let node_id = NodeId::new(dir, file, "_", name)?;
                                        nodes.push(Node {
                                            node_id,
                                            kind: NodeKind::Constant,
                                            name: name.to_string(),
                                            description,
                                            signatures: vec![],
                                            language: ctx.language,
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

fn collapse_overloads(nodes: &mut Vec<Node>) {
    use std::collections::HashMap;

    let mut groups: HashMap<String, Vec<usize>> = HashMap::new();
    for (i, n) in nodes.iter().enumerate() {
        // Two consecutive declarations with the same name and no body are TS overloads
        let key = format!("{}:{}:{}", n.file_path, n.kind.as_str(), n.name);
        groups.entry(key).or_default().push(i);
    }

    let mut to_remove = Vec::new();
    for indices in groups.values() {
        if indices.len() < 2 {
            continue;
        }
        let first = indices[0];
        let extra_sigs: Vec<Signature> = indices[1..]
            .iter()
            .flat_map(|&i| nodes[i].signatures.clone())
            .collect();
        nodes[first].signatures.extend(extra_sigs);
        to_remove.extend_from_slice(&indices[1..]);
    }

    to_remove.sort_unstable_by(|a, b| b.cmp(a));
    to_remove.dedup();
    for idx in to_remove {
        nodes.remove(idx);
    }
}

// ---------------------------------------------------------------------------
// Edge extraction
// ---------------------------------------------------------------------------

fn extract_js_edges(ctx: &FileContext, scope: &ProjectScope) -> Result<Vec<Edge>> {
    let mut edges = Vec::new();
    let root = ctx.tree.root_node();
    walk_for_edges(&root, &ctx.source, ctx, scope, &mut edges)?;
    Ok(edges)
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
            let bare = func_text.split('.').last().unwrap_or(func_text);
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
    let dir = ctx
        .path
        .parent()
        .and_then(|p| p.to_str())
        .unwrap_or(".");
    let file = ctx.path.file_name().and_then(|f| f.to_str()).unwrap_or("_");

    let mut current = node.parent();
    let mut class_name = "_".to_string();

    while let Some(parent) = current {
        if matches!(parent.kind(), "class_declaration" | "abstract_class_declaration") {
            if let Some(n) = parent.child_by_field_name("name") {
                class_name = node_text(&n, source).to_string();
            }
        }
        if matches!(
            parent.kind(),
            "function_declaration"
                | "arrow_function"
                | "function"
                | "method_definition"
                | "generator_function_declaration"
        ) {
            if let Some(name_node) = parent.child_by_field_name("name") {
                let name = node_text(&name_node, source);
                return Ok(Some(NodeId::new(dir, file, &class_name, name)?));
            }
        }
        current = parent.parent();
    }

    Ok(None)
}

// ---------------------------------------------------------------------------
// Helpers
// ---------------------------------------------------------------------------

fn extract_jsdoc(node: &TSNode, source: &[u8]) -> Option<String> {
    let start_row = node.start_position().row;
    let parent = node.parent()?;
    let mut cursor = parent.walk();

    for sibling in parent.children(&mut cursor) {
        if sibling.kind() == "comment" && sibling.end_position().row < start_row {
            let text = node_text(&sibling, source);
            if text.starts_with("/**") {
                let stripped = text
                    .trim_start_matches("/**")
                    .trim_end_matches("*/")
                    .lines()
                    .map(|l| l.trim_start_matches('*').trim())
                    .filter(|l| !l.is_empty())
                    .collect::<Vec<_>>()
                    .join(" ");
                if !stripped.is_empty() {
                    return Some(stripped);
                }
            }
        }
    }
    None
}

fn extract_function_signature(node: &TSNode, source: &[u8]) -> Signature {
    let mut params = Vec::new();

    if let Some(params_node) = node.child_by_field_name("parameters") {
        let mut cursor = params_node.walk();
        for child in params_node.children(&mut cursor) {
            match child.kind() {
                "identifier" => {
                    params.push(Param {
                        name: node_text(&child, source).to_string(),
                        type_: None,
                    });
                }
                "required_parameter" | "optional_parameter" => {
                    let name = child
                        .child_by_field_name("pattern")
                        .map(|n| node_text(&n, source).to_string())
                        .unwrap_or_default();
                    let type_ = child
                        .child_by_field_name("type")
                        .map(|n| node_text(&n, source).to_string());
                    if !name.is_empty() {
                        params.push(Param { name, type_ });
                    }
                }
                "assignment_pattern" => {
                    if let Some(left) = child.child_by_field_name("left") {
                        params.push(Param {
                            name: node_text(&left, source).to_string(),
                            type_: None,
                        });
                    }
                }
                _ => {}
            }
        }
    }

    let returns = node
        .child_by_field_name("return_type")
        .map(|n| node_text(&n, source).to_string());

    Signature { params, returns }
}

fn is_literal(node: &TSNode) -> bool {
    matches!(
        node.kind(),
        "number" | "string" | "template_string" | "true" | "false" | "null" | "undefined"
    )
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

    fn parse_ts(source: &[u8]) -> FileContext {
        spy_parser::parse_file(Path::new("test.ts"), source.to_vec(), Language::TypeScript)
            .unwrap()
    }

    fn parse_js(source: &[u8]) -> FileContext {
        spy_parser::parse_file(Path::new("test.js"), source.to_vec(), Language::JavaScript)
            .unwrap()
    }

    #[test]
    fn test_ts_function() {
        let ctx = parse_ts(b"function greet(name: string): void {}");
        let nodes = TypeScriptResolver.extract_nodes(&ctx).unwrap();
        assert_eq!(nodes.len(), 1);
        assert_eq!(nodes[0].name, "greet");
        assert_eq!(nodes[0].kind, NodeKind::Function);
    }

    #[test]
    fn test_ts_class() {
        let ctx = parse_ts(b"class Foo { bar(): void {} }");
        let nodes = TypeScriptResolver.extract_nodes(&ctx).unwrap();
        assert!(nodes.iter().any(|n| n.name == "Foo" && n.kind == NodeKind::Class));
        assert!(nodes.iter().any(|n| n.name == "bar" && n.kind == NodeKind::Function));
    }

    #[test]
    fn test_js_arrow_function() {
        let ctx = parse_js(b"const add = (a, b) => a + b;");
        let nodes = JavaScriptResolver.extract_nodes(&ctx).unwrap();
        assert!(nodes.iter().any(|n| n.name == "add" && n.kind == NodeKind::Function));
    }
}
