# Architecture

## Crate layout

Cargo workspace under `spy-code/`:

```
spy-code/
├── Cargo.toml                  # workspace
├── crates/
│   ├── spy-core/               # types, node id, errors, traits
│   ├── spy-parser/             # tree-sitter wrappers, AST walking
│   ├── spy-resolvers/          # per-lang resolvers (rust, py, ts, go)
│   ├── spy-storage/            # SQLite schema, migrations, queries
│   ├── spy-graph/              # GraphQL schema, resolvers (async-graphql)
│   ├── spy-git/                # git diff + hash tracking (gix)
│   ├── spy-indexer/            # orchestrates parse → resolve → store
│   ├── spy-mcp/                # MCP server (stdio JSON-RPC)
│   └── spy-cli/                # binary entrypoint
└── docs/
```

## Data flow

```
[source files]
     │
     ▼
[spy-git] ── unchanged files? skip ──┐
     │                                │
     ▼                                │
[spy-parser] tree-sitter parse        │
     │                                │
     ▼                                │
[spy-resolvers] per-lang: extract     │
   nodes (fn/class/const) + edges     │
   (calls/imports/refs)               │
     │                                │
     ▼                                ▼
[spy-indexer] dedupe + upsert ◄───────┘
     │
     ▼
[spy-storage] SQLite write
     │
     ▼
   .spy-code/graph.db
     │
     ├──► [spy-graph] GraphQL queries ──► [spy-cli query]
     └──► [spy-mcp] MCP tools ──────────► AI clients
```

## Key principles

1. **Static only**: no execution, no runtime introspection.
2. **Best-effort resolution**: resolvers may emit edges with `confidence` < 1.0 when scope resolution is ambiguous.
3. **Incremental**: per-file content hash gates re-parse. Git diff narrows the candidate set further.
4. **Graphs are plural**: edges live in separate tables per relation kind so we can grow new relation types without schema churn on existing ones.
5. **No source storage**: nodes describe shape, not body. Doc comments only.

## External dependencies

- `tree-sitter` + per-language grammars
- `async-graphql` for GraphQL server
- `rusqlite` (bundled) for storage
- `std::process::Command` shelling out to `git`
- `serde` / `serde_json` for config + MCP
- `clap` for CLI
- `blake3` for content hashing
