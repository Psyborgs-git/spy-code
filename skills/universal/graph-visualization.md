# Graph Visualization Skill

## When to Use

Use this skill when you need to:
- Visualize code relationships
- Explore the codebase structure interactively
- Understand module dependencies
- Identify architectural patterns
- Present code architecture to others
- Debug complex interactions

## Available Tools

### MCP Tools
- `graphData` - Get graph data filtered by criteria
- `query_graph` - Run raw GraphQL queries

### CLI Commands
- `spy-code graph [--path .] [--open]` - Generate and serve graph visualization
- `spy-code serve --http [--port 4000]` - Start GraphQL server with graph UI

## Example Queries

### Get full graph data
```graphql
{
  graphData {
    nodes {
      id
      name
      kind
      filePath
    }
    edges {
      from { id }
      to { id }
      kind
    }
  }
}
```

### Filter by file
```graphql
{
  graphData(filter: { filePath: "src/auth.rs" }) {
    nodes {
      name
      kind
    }
    edges {
      kind
    }
  }
}
```

### Filter by node kinds
```graphql
{
  graphData(filter: { nodeKinds: [FUNCTION] }) {
    nodes {
      name
      filePath
    }
    edges {
      from { name }
      to { name }
      kind
    }
  }
}
```

### Filter by language
```graphql
{
  graphData(filter: { languages: [RUST, PYTHON] }) {
    nodes {
      name
      language
      kind
    }
    edges {
      kind
    }
  }
}
```

### Filter by edge kinds
```graphql
{
  graphData(filter: { edgeKinds: [CALLS] }) {
    nodes {
      name
    }
    edges {
      from { name }
      to { name }
      confidence
    }
  }
}
```

## Starting the Graph UI

### CLI command
```bash
# Start graph server and open in browser
spy-code graph --open

# Start on specific path
spy-code graph --path ./src

# Start without opening browser
spy-code graph
```

### HTTP server
```bash
# Start GraphQL server with graph UI
spy-code serve --http --port 4000

# Visit http://localhost:4000/graph
```

## Graph UI Features

The graph visualization UI provides:
- **Interactive graph** - Pan, zoom, click to explore
- **Node filtering** - Filter by kind, language, file
- **Edge filtering** - Show/hide call, import, reference edges
- **Search** - Find nodes by name
- **Path highlighting** - Show paths between nodes
- **Export** - Export graph as PNG/SVG
- **Layout options** - Force-directed, hierarchical, circular

## Graph Filter Options

```graphql
input GraphFilter {
  filePath: String        # Filter to specific file
  nodeKinds: [NodeKind!]  # Filter by node types
  languages: [Language!]  # Filter by languages
  edgeKinds: [EdgeKind!]  # Filter by edge types
}
```

### Node Kinds
- `FUNCTION` - Functions and methods
- `CLASS` - Classes and structs
- `CONSTANT` - Constants and globals

### Languages
- `RUST`
- `PYTHON`
- `TYPESCRIPT`
- `JAVASCRIPT`
- `GO`

### Edge Kinds
- `CALLS` - Function calls
- `IMPORTS` - Module imports
- `REFERENCES` - Variable/type references

## Best Practices

1. **Start filtered** - Use filters to reduce graph size
2. **Focus on specific areas** - Filter by file or module first
3. **Use edge filters** - Show only relevant edge types
4. **Check confidence** - Low confidence edges may be unreliable
5. **Export for documentation** - Use PNG/SVG exports for docs

## Common Patterns

### Pattern 1: Explore Module Structure
```graphql
{
  graphData(filter: { filePath: "src/auth/" }) {
    nodes {
      name
      kind
    }
    edges {
      from { name }
      to { name }
      kind
    }
  }
}
```

### Pattern 2: Analyze Dependencies
```graphql
{
  graphData(filter: { edgeKinds: [IMPORTS] }) {
    nodes {
      name
      filePath
    }
    edges {
      from { filePath }
      to { filePath }
    }
  }
}
```

### Pattern 3: Understand Call Flow
```graphql
{
  graphData(filter: { 
    nodeKinds: [FUNCTION],
    edgeKinds: [CALLS]
  }) {
    nodes {
      name
      filePath
    }
    edges {
      from { name }
      to { name }
      confidence
    }
  }
}
```

### Pattern 4: Cross-Language Analysis
```graphql
{
  graphData(filter: { 
    languages: [RUST, PYTHON]
  }) {
    nodes {
      name
      language
      kind
    }
    edges {
      from { name language }
      to { name language }
      kind
    }
  }
}
```

## Graph Data Schema

```graphql
type GraphData {
  nodes: [Node!]!
  edges: [Edge!]!
}

type Node {
  id: NodeID!
  kind: NodeKind!
  name: String!
  description: String
  language: Language!
  filePath: String!
  startLine: Int!
  endLine: Int!
}

type Edge {
  from: Node!
  to: Node!
  kind: EdgeKind!
  confidence: Float!
}
```

## Performance Considerations

- Large graphs can be slow to render
- Use filters to reduce node/edge count
- Graph UI loads data on demand
- Consider exporting static images for large graphs
- HTTP server serves graph UI at `/graph` endpoint

## Export Options

From the graph UI:
- **PNG** - Raster image, good for documents
- **SVG** - Vector image, good for web/editing
- **JSON** - Raw graph data, for custom visualization

## Integration with Other Skills

Combine graph visualization with other skills:

1. **Code Navigation** - Find nodes, then visualize their connections
2. **Call Graph Analysis** - Visualize caller/callee relationships
3. **Change Tracking** - Visualize what changed between commits
4. **Semantic Search** - Find related code, then visualize relationships

## Troubleshooting

### Graph UI not loading
- Ensure HTTP server is running: `spy-code serve --http`
- Check port is not in use
- Verify browser can access localhost

### Graph is too large
- Apply filters to reduce scope
- Focus on specific files or modules
- Use edge filters to show only relevant relationships

### Nodes not appearing
- Verify indexing is complete: `spy-code stats`
- Check file filters are correct
- Ensure language is enabled in config

### Edges missing
- Check edge kind filters
- Verify confidence threshold
- Ensure indexing included edge extraction

## Use Cases

### For Architecture Review
- Visualize module dependencies
- Identify circular dependencies
- Understand system boundaries
- Document architecture

### For Onboarding
- Explore codebase structure
- Understand key components
- Find entry points
- Learn code organization

### For Refactoring
- Identify tightly coupled code
- Find potential extraction points
- Understand impact of changes
- Plan refactoring strategy
