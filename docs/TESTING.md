# Testing

## Layers

### 1. Unit tests

Per crate, in `src/` with `#[cfg(test)]`. Cover:

- `spy-core`: NodeId parsing, formatting, escape handling, max-length.
- `spy-storage`: every CRUD path, FTS triggers, cascade deletes, migration up/down.
- `spy-parser`: tree-sitter wrapper handles malformed input without panic.
- `spy-resolvers/*`: per-language node extraction on tiny inline source strings.
- `spy-git`: diff parsing, rename detection, dirty tree detection (with `gix` test repos).
- `spy-graph`: resolver functions return correct shapes from a stub DB.

### 2. Integration tests

Under `tests/` in each crate. Use real SQLite (file-backed temp dirs) and real tree-sitter parsers.

- `spy-indexer/tests/index_rust.rs`: index `tests/fixtures/rust_sample/` and assert exact node and edge counts.
- Same for python, ts, go.
- `spy-cli/tests/cli.rs`: spawn the binary with `assert_cmd`, run `init` → `index` → `query`, validate JSON output.

### 3. Fixture repos

`tests/fixtures/` holds tiny canonical repos per language:

```
tests/fixtures/
├── rust_sample/        # 6 files, traits, trait impls, generics
├── python_sample/      # 5 files, classes, @overload, decorators
├── ts_sample/          # 5 files, overloads, namespaces, tsconfig paths
├── go_sample/          # 4 files, package-level fns and methods
└── git_sample/         # pre-built git history with renames + deletes
```

Each fixture has a sibling `expected.json` snapshot of the indexed graph (nodes + edges). Tests compare against snapshots.

### 4. Snapshot tests

Use `insta` for GraphQL response snapshots. Canonical queries:

- `node(id: "...")` for each node kind.
- `search("foo")` returns expected ordering.
- `callers` / `callees` at depth 1, 2, 3.
- `changedSince(ref)` against `git_sample`.

### 5. Property tests

Use `proptest`:

- NodeId: parse(format(n)) == n for arbitrary valid nodes.
- Edge dedup: inserting the same edge N times produces 1 row.
- Hashing: identical input → identical content_hash.

### 6. End-to-end tests

`tests/e2e/` runs the full binary against real OSS repos vendored as git submodules:

- `serde` (Rust)
- `requests` (Python)
- `zod` (TypeScript)
- `cobra` (Go)

Asserts: indexing finishes, node count within tolerance band, no panics, query latency < 50ms p95.

### 7. MCP conformance

`spy-mcp/tests/mcp.rs`: spawn `spy-code serve --mcp`, drive it via stdin with recorded JSON-RPC sessions, compare stdout to golden files.

## CI matrix

GitHub Actions (`.github/workflows/ci.yml`):

| Platform | Rust | Test scope |
|---|---|---|
| ubuntu-latest | stable | fmt check + clippy + unit + integration |
| ubuntu-latest | beta | unit + integration |
| macos-latest | stable | clippy + unit + integration |
| windows-latest | stable | clippy + unit + integration |

A separate `release-check` job validates release builds on every push to `main`.

## Coverage target

- Line coverage ≥ 80% for `spy-core`, `spy-storage`, `spy-graph`.
- Resolver crates: judged by fixture coverage, not line coverage.

## Performance regression tests

`benches/` using `criterion`:

- Index 1k-file synthetic repo cold.
- Index 1k-file synthetic repo warm (no changes).
- Run 100 random `node(id)` lookups.
- Run 100 random `search` queries.

CI fails if any benchmark regresses > 20% vs baseline stored in `benches/baseline.json`.

## Manual QA checklist (pre-release)

- [ ] `spy-code init` in empty dir
- [ ] `spy-code index` on each fixture
- [ ] `spy-code query` returns valid JSON
- [ ] `spy-code serve --mcp` accepts at least one tool call from a real MCP client
- [ ] Re-index after git checkout to a different branch
- [ ] Re-index after rename
- [ ] Re-index after delete
- [ ] Force re-index with `--full`
