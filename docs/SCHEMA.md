# Schema

## SQLite schema

Single DB at `.spy-code/graph.db`. Migrations managed by `spy-storage`.

### `nodes`

```sql
CREATE TABLE nodes (
  node_id       TEXT PRIMARY KEY,        -- dir:file:class:symbol
  kind          TEXT NOT NULL,           -- 'function' | 'class' | 'constant'
  name          TEXT NOT NULL,
  description   TEXT,                    -- doc comment, nullable
  signatures    TEXT NOT NULL,           -- JSON array of {params, returns}
  language      TEXT NOT NULL,           -- 'rust' | 'python' | 'typescript' | 'go'
  file_path     TEXT NOT NULL,
  start_line    INTEGER NOT NULL,
  end_line      INTEGER NOT NULL,
  content_hash  TEXT NOT NULL,           -- blake3 of source slice
  git_sha       TEXT,                    -- commit at last index
  renamed_from  TEXT                     -- prior node_id, if git detected rename
);

CREATE INDEX idx_nodes_name ON nodes(name);
CREATE INDEX idx_nodes_file ON nodes(file_path);
CREATE INDEX idx_nodes_kind ON nodes(kind);
```

### Edge tables (one per relation)

```sql
CREATE TABLE edges_calls (
  from_id     TEXT NOT NULL,
  to_id       TEXT NOT NULL,
  confidence  REAL NOT NULL DEFAULT 1.0,
  PRIMARY KEY (from_id, to_id),
  FOREIGN KEY (from_id) REFERENCES nodes(node_id) ON DELETE CASCADE
);

CREATE TABLE edges_imports (
  from_id     TEXT NOT NULL,             -- file-level node or module
  to_id       TEXT NOT NULL,
  confidence  REAL NOT NULL DEFAULT 1.0,
  PRIMARY KEY (from_id, to_id)
);

CREATE TABLE edges_references (
  from_id     TEXT NOT NULL,
  to_id       TEXT NOT NULL,
  confidence  REAL NOT NULL DEFAULT 1.0,
  PRIMARY KEY (from_id, to_id)
);

CREATE INDEX idx_calls_to ON edges_calls(to_id);
CREATE INDEX idx_imports_to ON edges_imports(to_id);
CREATE INDEX idx_refs_to ON edges_references(to_id);
```

### `files`

```sql
CREATE TABLE files (
  path           TEXT PRIMARY KEY,
  language       TEXT NOT NULL,
  content_hash   TEXT NOT NULL,
  last_indexed   INTEGER NOT NULL,       -- unix ts
  git_sha        TEXT
);
```

### `index_meta`

```sql
CREATE TABLE index_meta (
  key TEXT PRIMARY KEY,
  value TEXT NOT NULL
);
-- keys: 'last_git_sha', 'schema_version', 'config_hash'
```

### Search (FTS5)

```sql
CREATE VIRTUAL TABLE nodes_fts USING fts5(
  node_id UNINDEXED,
  name,
  description,
  content=nodes,
  content_rowid=rowid
);
```

Triggers keep `nodes_fts` in sync on insert/update/delete of `nodes`.

### Embeddings

```sql
CREATE TABLE node_embeddings (
  node_id TEXT PRIMARY KEY,
  embedding BLOB NOT NULL,
  embedding_model TEXT NOT NULL,
  created_at INTEGER NOT NULL
);

CREATE TABLE source_embeddings (
  node_id TEXT PRIMARY KEY,
  embedding BLOB NOT NULL,
  embedding_model TEXT NOT NULL,
  created_at INTEGER NOT NULL
);

CREATE TABLE embedding_progress (
  id INTEGER PRIMARY KEY,
  total_nodes INTEGER NOT NULL,
  processed_nodes INTEGER NOT NULL DEFAULT 0,
  status TEXT NOT NULL,
  started_at INTEGER NOT NULL,
  completed_at INTEGER
);
```

---

## GraphQL schema

```graphql
scalar NodeID
scalar GitRef

enum NodeKind { FUNCTION CLASS CONSTANT }
enum Language { RUST PYTHON TYPESCRIPT JAVASCRIPT GO }
enum EdgeKind { CALLS IMPORTS REFERENCES }

type Param {
  name: String!
  type: String
}

type Signature {
  params: [Param!]!
  returns: String
}

type Node {
  id: NodeID!
  kind: NodeKind!
  name: String!
  description: String
  signatures: [Signature!]!
  language: Language!
  filePath: String!
  startLine: Int!
  endLine: Int!
  gitSha: String
  renamedFrom: NodeID

  callers(limit: Int = 50): [Edge!]!
  callees(limit: Int = 50): [Edge!]!
  importers(limit: Int = 50): [Edge!]!
  imports(limit: Int = 50): [Edge!]!
  referencedBy(limit: Int = 50): [Edge!]!
  references(limit: Int = 50): [Edge!]!
}

type Edge {
  from: Node!
  to: Node!
  kind: EdgeKind!
  confidence: Float!
}

type SearchResult {
  node: Node!
  score: Float!
}

type Query {
  node(id: NodeID!): Node
  search(query: String!, kind: NodeKind, limit: Int = 20): [SearchResult!]!
  callers(id: NodeID!, depth: Int = 1): [Edge!]!
  callees(id: NodeID!, depth: Int = 1): [Edge!]!
  changedSince(ref: GitRef!): [Node!]!
  files: [String!]!
  stats: IndexStats!
  semanticSearchEmbeddings(query: String!, limit: Int = 20): [SearchResult!]!
  embeddingsStatus: EmbeddingStatus!
  graphData(filter: GraphFilter): GraphData!
}

input GraphFilter {
  filePath: String
  nodeKinds: [NodeKind!]
  languages: [Language!]
  edgeKinds: [EdgeKind!]
}

type GraphData {
  nodes: [Node!]!
  edges: [Edge!]!
}

type EmbeddingStatus {
  totalNodes: Int!
  processedNodes: Int!
  status: String!
  startedAt: Int!
  completedAt: Int
}

type IndexStats {
  nodeCount: Int!
  edgeCount: Int!
  fileCount: Int!
  lastIndexed: String
  lastGitSha: String
}
```

No mutations. Indexing happens via CLI, not GraphQL.
