# Roadmap

Milestones are sequential. Each ends with a working binary on `main`.

## M0 — Skeleton (week 1)

- Cargo workspace, all 9 crates wired with empty types.
- `spy-core`: `Node`, `Edge`, `NodeId`, `Language`, `EdgeKind`, `Signature`, `Param`.
- CI: fmt, clippy, tests.
- `spy-code --version` works.

## M1 — Storage layer (week 2)

- `spy-storage`: SQLite schema + migrations.
- CRUD: upsert node, upsert edge, fetch node, fetch by file, cascade delete.
- FTS5 virtual table + triggers.
- Unit tests with in-memory SQLite.

## M2 — Rust resolver (week 3)

- `spy-parser`: tree-sitter wrapper for Rust.
- `spy-resolvers/rust`: extract nodes + edges.
- Two-pass indexing in `spy-indexer`.
- `spy-code init` and `spy-code index` work end-to-end on a Rust repo.
- Smoke test: index `spy-code` itself, assert >0 nodes/edges.

## M3 — GraphQL + CLI (week 4)

- `spy-graph`: full schema from `SCHEMA.md`, async-graphql resolvers.
- `spy-cli`: `query`, `get`, `search`, `callers`, `callees`, `stats`, `serve --http`.
- Snapshot tests on canonical queries against fixture repos.

## M4 — Git integration (week 5)

- `spy-git`: diff against `last_git_sha`, content-hash gate, rename tracking.
- `changedSince` query.
- `spy-code changed <ref>` CLI.
- Dirty tree handling, fallback to full index on unreachable refs.

## M5 — Python resolver (week 6)

- `spy-resolvers/python` with tree-sitter-python.
- `@overload` collapse.
- Cross-test: index a mid-size Python repo (e.g. `requests`).

## M6 — TypeScript/JS resolver (week 7)

- `spy-resolvers/ts` with tree-sitter-typescript + javascript.
- `tsconfig.json` path alias resolution.
- Function overload collapse.
- Cross-test on a real TS repo.

## M7 — Go resolver (week 8)

- `spy-resolvers/go` with tree-sitter-go.
- Package-level scope resolution.
- Cross-test on a small Go module.

## M8 — MCP server (week 9)

- `spy-mcp`: stdio JSON-RPC, tools listed in `CLI_MCP.md`.
- Resource endpoints for schema/stats/config.
- Manual smoke test against an MCP client.

## M9 — Hardening (week 10)

- Error messages, logging (`tracing`).
- Parallel indexing via `rayon`.
- Benchmarks: index a 50k LOC repo in < 10s warm, < 30s cold.
- Documentation polish.

## M10 — v1.0 (week 11)

- Crates.io release.
- Pre-built binaries for macOS / Linux / Windows.
- README + quickstart.

## Out of scope for v1

- Custom resolver plugins (dynamic load).
- Semantic-level rename detection.
- Multi-repo / monorepo federation.
- Web UI.
- Editor integrations (LSP).
- Mutations / write API.
- Submodule traversal.
