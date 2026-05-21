# spy-code

A GraphQL-style compiler for codebases. Parses source via tree-sitter, builds a graph of every function/class/constant (nodes) and their calls/imports/references (edges), stores it in SQLite, and exposes the graph via a real GraphQL API over CLI and MCP.

Built for AI systems that need precise, queryable context about a codebase — not for runtime introspection.

## Core idea

- **Nodes**: functions, classes, global constants. Each stores `name`, `description` (from doc comments), `params`, `returns`, `kind`, `node_id`.
- **Edges**: `calls`, `imports`, `references`. Stored in separate tables, one per relation.
- **Node ID**: `dir:file:class:fn` path-based identity.
- **Storage**: single SQLite DB per repo at `.spy-code/graph.db`.
- **Query**: real GraphQL via `async-graphql`.
- **Interfaces**: `spy-code` CLI binary + MCP server (stdio).
- **Incremental**: content-hash per file, skip unchanged. Git-aware: only re-index files changed since last indexed commit.
- **Embeddings**: vector embeddings for semantic search using open-source models (optional).
- **Graph Visualization**: interactive React-based graph UI for exploring code relationships.

## Supported languages (v1)

Rust, Python, TypeScript/JavaScript, Go. All via tree-sitter grammars + per-language resolver modules.

## Document index

- `docs/ARCHITECTURE.md` — system layout, crates, data flow
- `docs/NODE_ID_SPEC.md` — ID format, collision rules
- `docs/SCHEMA.md` — SQLite tables + GraphQL schema
- `docs/RESOLVERS.md` — per-language resolver contract
- `docs/CONFIG.md` — `spy.config.json` spec
- `docs/CLI_MCP.md` — CLI commands + MCP tool surface
- `docs/GIT_INTEGRATION.md` — change tracking
- `docs/ROADMAP.md` — milestones
- `docs/TESTING.md` — test strategy
- `docs/EMBEDDINGS.md` — vector embeddings for semantic search

## Quick start

```bash
# Initialize
spy-code init

# Index your codebase
spy-code index

# Query the graph
spy-code search "auth"
spy-code callers src:auth.rs:_:login

# Generate embeddings for semantic search
spy-code embed

# Ask natural language questions
spy-code ask "how do I authenticate users?"

# Start GraphQL server with graph visualization
spy-code serve --http
# Visit http://localhost:4000/graph for interactive graph UI
```
