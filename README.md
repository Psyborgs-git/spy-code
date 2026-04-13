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
