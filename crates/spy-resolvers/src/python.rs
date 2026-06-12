use anyhow::Result;
use spy_core::{
    Edge, EdgeKind, FileContext, Language, Node, NodeId, NodeKind, Param, ProjectScope, Resolver,
    Signature,
};
use tree_sitter::Node as TSNode;

pub struct PythonResolver;

impl Resolver for PythonResolver {
    fn language(&self) -> Language {
        Language::Python
    }

    fn extensions(&self) -> &[&str] {
        &["py"]
    }

    fn extract_nodes(&self, ctx: &FileContext) -> Result<Vec<Node>> {
        let mut nodes = Vec::new();
        let root = ctx.tree.as_ref().ok_or_else(|| anyhow::anyhow!("Tree is missing in FileContext"))?.root_node();

        let dir = ctx.path.parent().and_then(|p| p.to_str()).unwrap_or(".");
        let file = ctx.path.file_name().and_then(|f| f.to_str()).unwrap_or("_");

        walk_nodes(&root, &ctx.source, dir, file, "_", &mut nodes, ctx)?;

        // Collapse @overload-decorated functions into a single node with multiple signatures
        collapse_overloads(&mut nodes);

        Ok(nodes)
    }

    fn extract_edges(&self, ctx: &FileContext, scope: &ProjectScope) -> Result<Vec<Edge>> {
        let mut edges = Vec::new();
        let root = ctx.tree.as_ref().ok_or_else(|| anyhow::anyhow!("Tree is missing in FileContext"))?.root_node();

        // Build import map: alias/name → module
        let import_map = build_import_map(&root, &ctx.source);

        walk_for_edges(&root, &ctx.source, ctx, scope, &import_map, &mut edges)?;
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
    parent_class: &str,
    nodes: &mut Vec<Node>,
    ctx: &FileContext,
) -> Result<()> {
    match node.kind() {
        "function_definition" => {
            if let Some(name_node) = node.child_by_field_name("name") {
                let name = node_text(&name_node, source);
                let description = extract_docstring(node, source);
                let sig = extract_function_signature(node, source);
                let content_hash = compute_hash(node, source);
                let is_overload = has_overload_decorator(node, source);

                let node_id = NodeId::new(dir, file, parent_class, name)?;
                nodes.push(Node {
                    node_id,
                    kind: NodeKind::Function,
                    name: name.to_string(),
                    description,
                    signatures: vec![sig],
                    language: Language::Python,
                    file_path: ctx.path.to_string_lossy().to_string(),
                    start_line: node.start_position().row as u32 + 1,
                    end_line: node.end_position().row as u32 + 1,
                    content_hash: if is_overload {
                        format!("overload:{}", content_hash)
                    } else {
                        content_hash
                    },
                    git_sha: None,
                    renamed_from: None,
                });

                // Walk body for nested functions/classes
                if let Some(body) = node.child_by_field_name("body") {
                    let mut cursor = body.walk();
                    for child in body.children(&mut cursor) {
                        walk_nodes(&child, source, dir, file, parent_class, nodes, ctx)?;
                    }
                }
                return Ok(());
            }
        }
        "class_definition" => {
            if let Some(name_node) = node.child_by_field_name("name") {
                let name = node_text(&name_node, source);
                let description = extract_docstring(node, source);
                let content_hash = compute_hash(node, source);

                let node_id = NodeId::new(dir, file, "_", name)?;
                nodes.push(Node {
                    node_id,
                    kind: NodeKind::Class,
                    name: name.to_string(),
                    description,
                    signatures: vec![],
                    language: Language::Python,
                    file_path: ctx.path.to_string_lossy().to_string(),
                    start_line: node.start_position().row as u32 + 1,
                    end_line: node.end_position().row as u32 + 1,
                    content_hash,
                    git_sha: None,
                    renamed_from: None,
                });

                // Walk class body for methods
                if let Some(body) = node.child_by_field_name("body") {
                    let mut cursor = body.walk();
                    for child in body.children(&mut cursor) {
                        walk_nodes(&child, source, dir, file, name, nodes, ctx)?;
                    }
                }
                return Ok(());
            }
        }
        "expression_statement" => {
            // Module-level assignment of a literal → Constant
            if let Some(assign) = node.named_child(0) {
                if assign.kind() == "assignment" {
                    if let (Some(left), Some(right)) = (
                        assign.child_by_field_name("left"),
                        assign.child_by_field_name("right"),
                    ) {
                        if is_literal(&right) && parent_class == "_" {
                            let name = node_text(&left, source);
                            if is_valid_identifier(name) {
                                let content_hash = compute_hash(node, source);
                                let node_id = NodeId::new(dir, file, "_", name)?;
                                nodes.push(Node {
                                    node_id,
                                    kind: NodeKind::Constant,
                                    name: name.to_string(),
                                    description: None,
                                    signatures: vec![],
                                    language: Language::Python,
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
        _ => {}
    }

    let mut cursor = node.walk();
    for child in node.children(&mut cursor) {
        walk_nodes(&child, source, dir, file, parent_class, nodes, ctx)?;
    }

    Ok(())
}

// ---------------------------------------------------------------------------
// @overload collapse
// ---------------------------------------------------------------------------

/// Merge nodes that represent `@typing.overload` variants into a single node
/// with multiple signatures, keyed by `(file_path, parent_class, name)`.
fn collapse_overloads(nodes: &mut Vec<Node>) {
    use std::collections::HashMap;

    // Group overload variants
    let mut overload_map: HashMap<String, Vec<usize>> = HashMap::new();
    for (i, node) in nodes.iter().enumerate() {
        if node.content_hash.starts_with("overload:") {
            let key = format!("{}:{}:{}", node.file_path, node.kind.as_str(), node.name);
            overload_map.entry(key).or_default().push(i);
        }
    }

    // For each group, keep the first occurrence and merge signatures
    let mut to_remove = Vec::new();
    for indices in overload_map.values() {
        if indices.len() < 2 {
            continue;
        }
        let first = indices[0];
        let extra_sigs: Vec<Signature> = indices[1..]
            .iter()
            .flat_map(|&i| nodes[i].signatures.clone())
            .collect();
        nodes[first].signatures.extend(extra_sigs);
        nodes[first].content_hash = nodes[first]
            .content_hash
            .trim_start_matches("overload:")
            .to_string();
        to_remove.extend_from_slice(&indices[1..]);
    }

    // Remove merged duplicates (highest index first to preserve positions)
    to_remove.sort_unstable_by(|a, b| b.cmp(a));
    to_remove.dedup();
    for idx in to_remove {
        nodes.remove(idx);
    }
}

// ---------------------------------------------------------------------------
// Edge extraction
// ---------------------------------------------------------------------------

/// Build a map from imported name/alias → module string.
fn build_import_map(root: &TSNode, source: &[u8]) -> std::collections::HashMap<String, String> {
    let mut map = std::collections::HashMap::new();
    let mut cursor = root.walk();

    for child in root.children(&mut cursor) {
        match child.kind() {
            "import_statement" => {
                // import foo, import foo as bar
                let mut c2 = child.walk();
                for name_node in child.children(&mut c2) {
                    if name_node.kind() == "dotted_name" {
                        let name = node_text(&name_node, source);
                        map.insert(
                            name.split('.').next_back().unwrap_or(name).to_string(),
                            name.to_string(),
                        );
                    } else if name_node.kind() == "aliased_import" {
                        if let (Some(n), Some(a)) = (
                            name_node.child_by_field_name("name"),
                            name_node.child_by_field_name("alias"),
                        ) {
                            map.insert(
                                node_text(&a, source).to_string(),
                                node_text(&n, source).to_string(),
                            );
                        }
                    }
                }
            }
            "import_from_statement" => {
                // from foo import bar, baz
                let module = child
                    .child_by_field_name("module_name")
                    .map(|n| node_text(&n, source).to_string())
                    .unwrap_or_default();

                let mut c2 = child.walk();
                for name_node in child.children(&mut c2) {
                    if name_node.kind() == "dotted_name" {
                        let name = node_text(&name_node, source);
                        map.insert(name.to_string(), format!("{}.{}", module, name));
                    } else if name_node.kind() == "aliased_import" {
                        if let Some(a) = name_node.child_by_field_name("alias") {
                            map.insert(node_text(&a, source).to_string(), module.clone());
                        }
                    }
                }
            }
            _ => {}
        }
    }
    map
}

fn walk_for_edges(
    node: &TSNode,
    source: &[u8],
    ctx: &FileContext,
    scope: &ProjectScope,
    import_map: &std::collections::HashMap<String, String>,
    edges: &mut Vec<Edge>,
) -> Result<()> {
    match node.kind() {
        "call" => {
            if let Some(func_node) = node.child_by_field_name("function") {
                let func_text = node_text(&func_node, source);
                // Strip attribute access: foo.bar → try "bar"
                let bare_name = func_text.split('.').next_back().unwrap_or(func_text);
                let from_id = infer_containing_function(node, source, ctx)?;
                if let Some(from_id) = from_id {
                    let candidates = scope.find_nodes_by_name(bare_name);
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
                    let _ = import_map; // used for future module-qualified resolution
                }
            }
        }
        "class_definition" => {
            if let Some(superclasses_node) = node.child_by_field_name("superclasses") {
                let name_node = node.child_by_field_name("name");
                if let Some(name_node) = name_node {
                    let class_name = node_text(&name_node, source);
                    let dir = ctx.path.parent().and_then(|p| p.to_str()).unwrap_or(".");
                    let file = ctx.path.file_name().and_then(|f| f.to_str()).unwrap_or("_");
                    if let Ok(from_id) = NodeId::new(dir, file, "_", class_name) {
                        let mut cursor = superclasses_node.walk();
                        for child in superclasses_node.children(&mut cursor) {
                            if child.kind() == "identifier" || child.kind() == "attribute" {
                                let base_name = node_text(&child, source);
                                let bare_name =
                                    base_name.split('.').next_back().unwrap_or(base_name);
                                let candidates = scope.find_nodes_by_name(bare_name);
                                if candidates.len() == 1 {
                                    edges.push(Edge {
                                        from_id: from_id.clone(),
                                        to_id: candidates[0].node_id.clone(),
                                        kind: EdgeKind::InheritsFrom,
                                        confidence: 1.0,
                                    });
                                } else if !candidates.is_empty() {
                                    edges.push(Edge {
                                        from_id: from_id.clone(),
                                        to_id: candidates[0].node_id.clone(),
                                        kind: EdgeKind::InheritsFrom,
                                        confidence: 0.4,
                                    });
                                }
                            }
                        }
                    }
                }
            }
        }
        "import_statement" | "import_from_statement" => {
            // Emit import edges from a file-level sentinel node (future work)
            // For now, skip.
        }
        _ => {}
    }

    let mut cursor = node.walk();
    for child in node.children(&mut cursor) {
        walk_for_edges(&child, source, ctx, scope, import_map, edges)?;
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
    let mut class_name = "_";

    while let Some(parent) = current {
        if parent.kind() == "class_definition" {
            if let Some(n) = parent.child_by_field_name("name") {
                class_name = node_text(&n, source);
            }
        }
        if parent.kind() == "function_definition" {
            if let Some(name_node) = parent.child_by_field_name("name") {
                let name = node_text(&name_node, source);
                return Ok(Some(NodeId::new(dir, file, class_name, name)?));
            }
        }
        current = parent.parent();
    }

    Ok(None)
}

// ---------------------------------------------------------------------------
// Helpers
// ---------------------------------------------------------------------------

fn extract_docstring(node: &TSNode, source: &[u8]) -> Option<String> {
    // First child of the body that is an expression_statement containing a string
    let body = node.child_by_field_name("body")?;
    let mut cursor = body.walk();
    if let Some(child) = body.children(&mut cursor).next() {
        if child.kind() == "expression_statement" {
            if let Some(expr) = child.named_child(0) {
                if expr.kind() == "string" {
                    let raw = node_text(&expr, source);
                    // Strip quotes
                    let stripped = raw
                        .trim_start_matches("\"\"\"")
                        .trim_end_matches("\"\"\"")
                        .trim_start_matches("'''")
                        .trim_end_matches("'''")
                        .trim_start_matches('"')
                        .trim_end_matches('"')
                        .trim_start_matches('\'')
                        .trim_end_matches('\'')
                        .trim();
                    if !stripped.is_empty() {
                        return Some(stripped.to_string());
                    }
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
                    // positional param without annotation
                    let name = node_text(&child, source);
                    if name != "self" && name != "cls" {
                        params.push(Param {
                            name: name.to_string(),
                            type_: None,
                        });
                    }
                }
                "typed_parameter" => {
                    let name = child
                        .child_by_field_name("name")
                        .or_else(|| child.named_child(0))
                        .map(|n| node_text(&n, source).to_string())
                        .unwrap_or_default();
                    let type_ = child
                        .child_by_field_name("type")
                        .map(|n| node_text(&n, source).to_string());
                    if !name.is_empty() && name != "self" && name != "cls" {
                        params.push(Param { name, type_ });
                    }
                }
                "default_parameter" | "typed_default_parameter" => {
                    let name = child
                        .child_by_field_name("name")
                        .map(|n| node_text(&n, source).to_string())
                        .unwrap_or_default();
                    let type_ = child
                        .child_by_field_name("type")
                        .map(|n| node_text(&n, source).to_string());
                    if !name.is_empty() && name != "self" && name != "cls" {
                        params.push(Param { name, type_ });
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

fn has_overload_decorator(node: &TSNode, source: &[u8]) -> bool {
    if let Some(parent) = node.parent() {
        let mut cursor = parent.walk();
        for sibling in parent.children(&mut cursor) {
            if sibling.kind() == "decorator" {
                let text = node_text(&sibling, source);
                if text.contains("overload") {
                    return true;
                }
            }
        }
    }
    false
}

fn is_literal(node: &TSNode) -> bool {
    matches!(
        node.kind(),
        "integer" | "float" | "string" | "true" | "false" | "none" | "concatenated_string"
    )
}

fn is_valid_identifier(s: &str) -> bool {
    !s.is_empty()
        && s.chars()
            .next()
            .map(|c| c.is_alphabetic() || c == '_')
            .unwrap_or(false)
        && s.chars().all(|c| c.is_alphanumeric() || c == '_')
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
        spy_parser::parse_file(Path::new("test.py"), source.to_vec(), Language::Python).unwrap()
    }

    #[test]
    fn test_extract_function() {
        let ctx = parse(b"def hello(x, y): pass");
        let nodes = PythonResolver.extract_nodes(&ctx).unwrap();
        assert_eq!(nodes.len(), 1);
        assert_eq!(nodes[0].name, "hello");
        assert_eq!(nodes[0].kind, NodeKind::Function);
        assert_eq!(nodes[0].signatures[0].params.len(), 2);
    }

    #[test]
    fn test_extract_class() {
        let ctx = parse(b"class Foo:\n    def bar(self): pass");
        let nodes = PythonResolver.extract_nodes(&ctx).unwrap();
        assert!(nodes
            .iter()
            .any(|n| n.name == "Foo" && n.kind == NodeKind::Class));
        assert!(nodes
            .iter()
            .any(|n| n.name == "bar" && n.kind == NodeKind::Function));
    }

    #[test]
    fn test_extract_constant() {
        let ctx = parse(b"MAX = 100");
        let nodes = PythonResolver.extract_nodes(&ctx).unwrap();
        assert!(nodes
            .iter()
            .any(|n| n.name == "MAX" && n.kind == NodeKind::Constant));
    }
}
