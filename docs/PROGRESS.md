# spy-code — Feature Parity & Progress Report

> Generated: 2026-04-20  
> Branch: `copilot/create-implementation-plan-for-features`

---

## Summary

This document maps every feature from the spec / docs against the current
implementation state, test coverage, and remaining gaps.

**Validation runs (as of this report):**

| Target | Files | Nodes | Edges | Status |
|--------|-------|-------|-------|--------|
| spy-code (self-index) | 23 | 449 | 168 | ✅ Pass |
| python_sample fixture | 3 | 16 | 31 | ✅ Pass |

---

## Feature Parity Matrix

### Core Data Model (`spy-core`)

| Feature | Spec | Implemented | Tests | Notes |
|---------|------|-------------|-------|-------|
| `NodeId` (dir:file:class:symbol, 512-char) | Yes | ✅ | ✅ unit | – |
| `NodeKind` (Function/Class/Constant) | Yes | ✅ | ✅ unit | – |
| `EdgeKind` (Calls/Imports/References) | Yes | ✅ | ✅ unit | – |
| `Language` (Rust/Python/TS/JS/Go) | Yes | ✅ | ✅ unit | – |
| `ProjectScope` (name lookup, all_nodes) | Yes | ✅ | ✅ unit | – |
| `Config` JSON (all fields) | Yes | ✅ | ✅ unit | Custom ser/deser for `parallelism` |
| `LanguageConfig` roots/ignore/enabled | Yes | ✅ | ✅ integration | **Enforcement now wired** |

---

### Storage (`spy-storage`)

| Feature | Spec | Implemented | Tests | Notes |
|---------|------|-------------|-------|-------|
| SQLite nodes table + FTS5 | Yes | ✅ | ✅ unit | Triggers keep FTS5 in sync |
| `edges_calls` table + FK | Yes | ✅ | ✅ unit | – |
| `edges_imports` table + FK | Yes | ✅ | ✅ unit | **Now populated by resolvers** |
| `edges_references` table + FK | Yes | ✅ | ✅ unit | **Now populated by Python resolver** |
| `files` table | Yes | ✅ | ✅ unit | – |
| `index_meta` (last_git_sha, config_hash) | Yes | ✅ | ✅ unit | – |
| `get_stats` | Yes | ✅ | ✅ unit | – |
| `list_files` | Yes | ✅ | – | Used in path-normalization test |
| `get_nodes_for_files` (changed query) | Yes | ✅ | – | Requires absolute paths (now fixed) |
| `search_nodes` (FTS5) | Yes | ✅ | ✅ unit | – |
| `get_incoming_edges` / `get_edges` | Yes | ✅ | ✅ unit | – |
| `renamed_from` population | Spec | ❌ | – | Schema column exists; git rename not wired |

---

### Indexer (`spy-indexer`)

| Feature | Spec | Implemented | Tests | Notes |
|---------|------|-------------|-------|-------|
| Two-pass indexing (nodes → scope → edges) | Yes | ✅ | ✅ integration | – |
| Full re-index (`--full`) | Yes | ✅ | ✅ integration | – |
| Incremental (git diff) | Yes | ✅ | ✅ integration | – |
| Incremental (content-hash fallback) | Yes | ✅ | ✅ integration | – |
| Config-hash invalidation (force full) | Yes | ✅ | – | – |
| **Path normalization (absolute stored paths)** | Yes | ✅ **Fixed** | ✅ integration | Canonicalize at start of `index()` |
| `roots` per-language enforcement | Spec | ✅ **Fixed** | ✅ integration | Filters files not under any root |
| `ignore` glob patterns per-language | Spec | ✅ **Fixed** | ✅ integration | Glob match on relative path |
| Max file size (`max_file_size_kb`) | Yes | ✅ | – | – |
| Hardcoded ignore dirs (target/.git/etc.) | Yes | ✅ | – | Separate from config ignore |
| `fail_fast` mode | Yes | ✅ | – | – |
| Parallel indexing (`parallelism` config) | Spec | ❌ | – | Config parsed but single-threaded |
| `renamed_from` tracking | Yes | ❌ | – | `apply_git_diff` handles delete/add but not rename attribution |

---

### Resolvers (`spy-resolvers`)

#### Python

| Feature | Spec | Implemented | Tests | Notes |
|---------|------|-------------|-------|-------|
| `function_definition` nodes | Yes | ✅ | ✅ unit | – |
| `class_definition` nodes | Yes | ✅ | ✅ unit | – |
| Module-level constant nodes | Yes | ✅ | ✅ unit | – |
| Docstring extraction | Yes | ✅ | ✅ unit | – |
| `@overload` collapse | Yes | ✅ | ✅ unit | – |
| Typed param/return signatures | Yes | ✅ | ✅ unit | – |
| `calls` edges | Yes | ✅ | ✅ integration | – |
| **`infer_containing_function` class context** | Yes | ✅ **Fixed** | ✅ integration | Was using wrong class name for methods |
| **`imports` edges** | Yes | ✅ **New** | ✅ integration | Module-level imports → project scope |
| **`references` edges (type annotations)** | Yes | ✅ **New** | – | Param types + return type resolved |

#### Rust

| Feature | Spec | Implemented | Tests | Notes |
|---------|------|-------------|-------|-------|
| All node kinds (fn/struct/enum/trait/impl/const) | Yes | ✅ | ✅ unit | – |
| Doc comments (`///`, `//!`) | Yes | ✅ | ✅ unit | – |
| `calls` edges | Yes | ✅ | – | Direct call resolution only |
| `imports` edges (`use` declarations) | Spec | ❌ | – | `use_declaration` not parsed |
| `references` edges (type names in signatures) | Spec | ❌ | – | Not implemented |

#### TypeScript / JavaScript

| Feature | Spec | Implemented | Tests | Notes |
|---------|------|-------------|-------|-------|
| Function/method/class/const nodes | Yes | ✅ | ✅ unit | – |
| JSDoc extraction | Yes | ✅ | ✅ unit | – |
| `calls` edges | Yes | ✅ | – | – |
| **`imports` edges** | Spec | ✅ **New** | – | `import { Foo } from '...'` → project scope |
| `references` edges | Spec | ❌ | – | Type annotations not parsed |

#### Go

| Feature | Spec | Implemented | Tests | Notes |
|---------|------|-------------|-------|-------|
| Function/method/struct/const nodes | Yes | ✅ | ✅ unit | – |
| Preceding comment extraction | Yes | ✅ | ✅ unit | – |
| `calls` edges | Yes | ✅ | – | – |
| `imports` edges | Spec | ❌ | – | `import_declaration` not parsed |
| `references` edges | Spec | ❌ | – | Not implemented |

---

### CLI (`spy-code`)

| Command | Spec | Implemented | Tests | Notes |
|---------|------|-------------|-------|-------|
| `init` | Yes | ✅ | – | Creates `spy.config.json` |
| `index [--full] [--path]` | Yes | ✅ | ✅ integration | – |
| `query <graphql>` | Yes | ✅ | – | Routes to GraphQL schema |
| `query --json` | Yes | ✅ **Fixed** | – | Was identical to non-json; now compact vs pretty |
| `get <node-id>` | Yes | ✅ | – | – |
| `search <text> [--kind]` | Yes | ✅ | – | FTS5 search with optional kind filter |
| **`callers <id> [--depth N]`** | Yes | ✅ **Fixed** | – | Was single-hop; now BFS up to N hops |
| **`callees <id> [--depth N]`** | Yes | ✅ **Fixed** | – | Was single-hop; now BFS up to N hops |
| `changed <ref>` | Yes | ✅ | – | Requires absolute paths (now fixed) |
| `stats` | Yes | ✅ | – | – |
| `serve --http [--port]` | Yes | ✅ | – | GraphQL playground |
| `serve --mcp` | Yes | ✅ | ✅ unit | JSON-RPC 2.0 over stdio |

---

### GraphQL Schema (`spy-graph`)

| Feature | Spec | Implemented | Tests | Notes |
|---------|------|-------------|-------|-------|
| `node(id)` | Yes | ✅ | ✅ unit | – |
| `search(query, kind, limit)` | Yes | ✅ | ✅ unit | – |
| `callers(id, depth)` / `callees(id, depth)` | Yes | ✅ | ✅ unit | BFS with cycle detection |
| `changedSince(ref)` | Yes | ✅ | – | Depends on path normalization (now fixed) |
| `files` | Yes | ✅ | – | – |
| `stats` | Yes | ✅ | ✅ unit | – |
| `Node.callers/callees` (relations) | Yes | ✅ | ✅ unit | – |
| `Node.importers/imports` (relations) | Yes | ✅ | ✅ unit | **Populated for Python** |
| `Node.referencedBy/references` (relations) | Yes | ✅ | ✅ unit | **Populated for Python** |

---

### MCP Server (`spy-mcp`)

| Feature | Spec | Implemented | Tests | Notes |
|---------|------|-------------|-------|-------|
| `initialize` response | Yes | ✅ | ✅ unit | – |
| `tools/list` | Yes | ✅ | ✅ unit | All tools advertised |
| `tools/call` (search, callers, callees, etc.) | Yes | ✅ | ✅ unit | – |
| `resources/list` | Yes | ✅ | ✅ unit | – |
| `resources/read` | Yes | ✅ | ✅ unit | – |
| MCP conformance tests | Spec | ❌ | – | Not implemented |

---

## Remaining Gaps (Prioritized)

### P1 — Breaks correctness for real repos

| Gap | Impact | Effort |
|-----|--------|--------|
| `renamed_from` not populated after git rename | `changedSince` misses renamed nodes | Medium |
| Rust resolver: no `imports`/`references` edges | Rust graph is calls-only | High |
| Go resolver: no `imports`/`references` edges | Go graph is calls-only | Medium |
| TS resolver: no `references` edges | TS type graph incomplete | Medium |

### P2 — Missing features (documented, not working)

| Gap | Impact | Effort |
|-----|--------|--------|
| Parallel indexing (`parallelism` config) | Slow on large repos | High |
| `query --json` only outputs raw GraphQL | No human-readable alternate format | Low |

### P3 — Test coverage gaps

| Gap | Affected area | Effort |
|-----|---------------|--------|
| E2E CLI tests (init→index→search→callers) | All commands | Medium |
| MCP conformance tests | spy-mcp | Medium |
| Snapshot tests for node/edge structure | All resolvers | Low |
| Property tests for NodeId, Config | spy-core | Low |
| Benchmark suite | Indexer performance | High |

### P4 — Observability & polish

| Gap | Impact | Effort |
|-----|--------|--------|
| `tracing` / structured logging | Debugging in production | Low |
| `spy-code doctor` command (health check) | User experience | Low |
| `spy-code status` (index freshness) | User experience | Low |

---

## Validation Log

### Phase 1: self-index spy-code

```
spy-code init
spy-code index --full --path .
→ Indexed 23 files, 449 nodes, 168 edges

spy-code stats
→ nodeCount: 449, edgeCount: 168, fileCount: 23

spy-code search "index"       → 13 results ✓
spy-code search "extract" --kind function  → 20 results ✓
spy-code get <upsert_node>    → detailed node output ✓

GraphQL: { stats { nodeCount edgeCount fileCount } }  → correct ✓
GraphQL: { search(query: "index", limit: 3) { node { id name filePath } } }  → correct ✓
GraphQL: { callers(id: "...", depth: 2) }  → BFS traversal ✓
```

### Phase 2: python_sample fixture (animals.py + math.py + zoo.py)

```
spy-code index --full --path tests/fixtures/python_sample
→ Indexed 3 files, 16 nodes, 31 edges

spy-code search "animal"   → 4 results: Animal, add_animal, animal_count ✓
spy-code search "add"      → 3 results: add, add_animal, add_dog ✓

GraphQL: importers(Animal) → 8 importers from zoo.py ✓
GraphQL: referencedBy(Animal) → add_animal (type annotation) ✓
GraphQL: referencedBy(Dog) → add_dog (return type) ✓
GraphQL: callers(add) → create_zoo, animal_count ✓
CLI callees(create_zoo) → add ✓
Imports edges in zoo.py → add, Animal, Dog (confidence 0.7) ✓
References edges in zoo.py → add_animal→Animal, add_dog→Dog ✓
```

---

## Confidence Scoring Reference

| Value | Meaning |
|-------|---------|
| `1.0` | Unambiguous single candidate in scope |
| `0.7` | Best-guess heuristic (file-level import relationship) |
| `0.4` | Multiple candidates; picked most likely |
| `< 0.4` | Dropped |
