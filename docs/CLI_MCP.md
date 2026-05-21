# CLI & MCP Interface

## CLI

Binary: `spy-code`. Built from `crates/spy-cli`.

### Commands

```
spy-code init                          # write default spy.config.json
spy-code index [--full] [--path .]     # build/update graph; --full forces re-index
spy-code query '<graphql>' [--json]    # run a GraphQL query against local DB
spy-code get <node_id>                 # convenience: fetch single node
spy-code search <text> [--kind fn]     # convenience: name/desc search
spy-code callers <node_id> [--depth N]
spy-code callees <node_id> [--depth N]
spy-code changed <git_ref>             # nodes changed since ref
spy-code stats                         # node/edge/file counts
spy-code serve --mcp                   # MCP stdio server
spy-code serve --http [--port 4000]    # GraphQL HTTP server (dev)
spy-code embed [--full] [--model <path>] # Generate embeddings for semantic search
spy-code ask "natural language query"  # Ask questions about the codebase
spy-code graph [--path .] [--open]     # Generate and serve graph visualization
```

### Examples

```bash
spy-code init
spy-code index
spy-code search "auth" --kind function
spy-code query '{ node(id: "src:auth.rs:_:login") { name description signatures { params { name type } returns } } }'
spy-code callers src:auth.rs:_:login --depth 2
spy-code changed HEAD~5
spy-code embed --model .spy-code/models/all-MiniLM-L6-v2
spy-code ask "how do I authenticate users in this codebase?"
spy-code graph --open
```

### Output

- Default: pretty text.
- `--json`: raw GraphQL response JSON.
- Exit codes: `0` ok, `1` user error, `2` index error, `3` query error.

---

## MCP server

`spy-code serve --mcp` runs an MCP server over stdio (JSON-RPC 2.0). Implements the standard MCP `tools/list` + `tools/call` flow.

### Exposed tools

| Tool | Args | Returns |
|---|---|---|
| `query_graph` | `{ query: string, variables?: object }` | GraphQL response JSON |
| `get_node` | `{ node_id: string }` | Node JSON or null |
| `search` | `{ query: string, kind?: string, limit?: int }` | `SearchResult[]` |
| `find_callers` | `{ node_id: string, depth?: int }` | `Edge[]` |
| `find_callees` | `{ node_id: string, depth?: int }` | `Edge[]` |
| `changed_since` | `{ git_ref: string }` | `Node[]` |
| `stats` | `{}` | `IndexStats` |

### Tool descriptions

Each tool ships with a description optimized for AI consumption:

```
query_graph: Run a raw GraphQL query against the codebase graph.
  Use this for complex multi-hop queries. Schema is documented at
  spy-code://schema. Prefer the specialized tools below for common ops.

get_node: Fetch one node by its ID. Node IDs are 'dir:file:class:symbol'.
  Returns name, description, signatures (params + returns), and location.

search: Find nodes by fuzzy name/description match. Use this when you
  know roughly what something is called but not its exact ID.

find_callers: List all functions/methods that call the given node.
  Use depth > 1 to walk transitively up the call graph.

find_callees: List all functions/methods called by the given node.
  Use depth > 1 to walk transitively down.

changed_since: List nodes whose source changed since the given git ref.
  Use this to find what an AI agent needs to re-read after a rebase.
```

### Resources

- `spy-code://schema` — full GraphQL SDL
- `spy-code://stats` — current index stats
- `spy-code://config` — loaded config (sanitized)

### Lifecycle

- Server reads config, opens DB read-only, parks on stdin.
- No mutations exposed via MCP. Re-indexing is CLI-only by design.
