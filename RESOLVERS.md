# Resolvers

Per-language modules under `spy-resolvers/`. Each implements the `Resolver` trait from `spy-core`.

## Trait

```rust
pub trait Resolver: Send + Sync {
    fn language(&self) -> Language;
    fn extensions(&self) -> &[&str];

    /// Walk the tree-sitter AST and emit nodes.
    fn extract_nodes(&self, ctx: &FileContext) -> Result<Vec<Node>>;

    /// Resolve calls/imports/references against project scope.
    /// May return edges with confidence < 1.0 when ambiguous.
    fn extract_edges(&self, ctx: &FileContext, scope: &ProjectScope) -> Result<Vec<Edge>>;
}
```

`FileContext` carries the parsed tree, source bytes, file path, and resolver-specific config from `spy.config.json`.

`ProjectScope` is a read-only index of all known nodes built in pass 1, used in pass 2 for cross-file resolution.

## Two-pass indexing

1. **Pass 1 (extract)**: every file → `extract_nodes`. Build `ProjectScope`.
2. **Pass 2 (resolve)**: every file → `extract_edges`, using `ProjectScope` to resolve names.

This guarantees forward references resolve correctly.

## Per-language resolvers

### Rust (`resolvers/rust.rs`)

- Use `tree-sitter-rust`.
- Nodes: `function_item`, `struct_item`, `enum_item`, `trait_item`, `impl_item` (methods), `const_item`, `static_item`.
- Description: `///` and `//!` doc comments.
- Edges:
  - `calls`: `call_expression` → resolve via `use` imports + same-module + `crate::` paths.
  - `imports`: `use_declaration`.
  - `references`: type references in signatures, struct field types.
- Trait impl disambiguation: when an impl is `impl Trait for Type`, qualify class as `Type<Trait>`.

### Python (`resolvers/python.rs`)

- Use `tree-sitter-python`.
- Nodes: `function_definition`, `class_definition`, module-level assignments where RHS is literal.
- Description: first triple-quoted string in body.
- Edges:
  - `calls`: `call` nodes; resolve by walking `import` / `from ... import` map for the file.
  - `imports`: `import_statement`, `import_from_statement`.
  - `references`: type annotations in params and return.
- `@overload` collapses into one node with multi-signature.

### TypeScript / JavaScript (`resolvers/ts.rs`)

- Use `tree-sitter-typescript` and `tree-sitter-javascript`.
- Nodes: `function_declaration`, `method_definition`, `class_declaration`, `lexical_declaration` for top-level `const`.
- Description: `/** */` JSDoc preceding the node.
- Edges:
  - `calls`: `call_expression`.
  - `imports`: `import_statement`. Resolve relative imports against file path; alias paths via `tsconfig.json` → readable from config.
  - `references`: type annotations.
- Function overloads collapse into one node.

### Go (`resolvers/go.rs`)

- Use `tree-sitter-go`.
- Nodes: `function_declaration`, `method_declaration`, `type_declaration` (for structs/interfaces), `const_declaration` at package level.
- Description: line comments immediately preceding declaration.
- Edges:
  - `calls`: `call_expression`.
  - `imports`: `import_declaration`.
  - `references`: type names in signatures.

## Confidence scoring

- `1.0`: name resolved unambiguously to exactly one node in scope.
- `0.7`: name resolved to one node by best-guess heuristic (e.g. same package, no explicit import).
- `0.4`: name matches multiple nodes; edge points to most likely candidate.
- `< 0.4`: drop edge.

## Config-driven resolver behavior

Resolvers read their section from `spy.config.json`:

```json
{
  "languages": {
    "rust": { "roots": ["src/", "crates/"], "ignore": ["target/"] },
    "python": { "roots": ["./"], "ignore": [".venv/", "__pycache__/"] }
  }
}
```

See `CONFIG.md` for full schema.
