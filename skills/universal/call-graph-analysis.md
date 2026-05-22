# Call Graph Analysis Skill

## When to Use

Use this skill when you need to:
- Understand how functions call each other
- Find all callers of a specific function
- Find all functions called by a specific function
- Analyze code dependencies
- Understand impact of changes
- Trace execution flow
- Find dead code (functions with no callers)

## Available Tools

### MCP Tools
- `find_callers` - List all functions/methods that call the given node
- `find_callees` - List all functions/methods called by the given node
- `query_graph` - Run raw GraphQL queries for complex analysis

### CLI Commands
- `spy-code callers <node_id> [--depth N]` - Find callers
- `spy-code callees <node_id> [--depth N]` - Find callees

## Example Queries

### Find direct callers of a function
```bash
spy-code callers src:auth.rs:_:login
```

### Find callers up to 3 levels deep
```bash
spy-code callers src:auth.rs:_:login --depth 3
```

### Find what a function calls
```bash
spy-code callees src:auth.rs:_:login
```

### GraphQL query for callers with details
```graphql
{
  callers(id: "src:auth.rs:_:login", depth: 2) {
    from {
      name
      filePath
      startLine
    }
    to {
      name
    }
    kind
    confidence
  }
}
```

## Best Practices

1. **Start with depth 1** - Use `--depth 1` first, then increase if needed
2. **Check confidence** - Edge confidence < 1.0 indicates potential ambiguity
3. **Use for impact analysis** - Before changing a function, check its callers
4. **Find entry points** - Use callees to trace from main functions
5. **Detect dead code** - Functions with no callers might be unused

## Common Patterns

### Pattern 1: Impact Analysis
```bash
# Find all callers before making changes
spy-code callers src:api:handlers:process_request --depth 2

# Review each caller to understand impact
# Make changes
# Re-index if needed
```

### Pattern 2: Trace Execution Flow
```bash
# Start from entry point
spy-code callees src:main:_:main --depth 3

# Follow the call chain
# Identify key functions
# Understand the flow
```

### Pattern 3: Find Dead Code
```bash
# Search for functions
spy-code search "helper" --kind function

# Check each for callers
spy-code callers src:utils:helpers:_:unused_function

# If no callers, consider removing
```

### Pattern 4: Understand Dependencies
```bash
# Find what a module depends on
spy-code callees src:auth:_:authenticate

# Analyze the dependency graph
# Identify tight coupling
# Plan refactoring
```

## Depth Parameter

The `depth` parameter controls how many levels to traverse:

- `depth: 1` - Only direct callers/callees
- `depth: 2` - Direct + one level of indirection
- `depth: 3+` - Deeper traversal (use carefully, can be large)

**Warning**: High depth values can return very large results. Start low and increase gradually.

## Edge Confidence

Edges have a `confidence` score from 0.0 to 1.0:

- `1.0` - Certain (fully resolved)
- `< 1.0` - Probabilistic (ambiguous resolution)

Low confidence edges may indicate:
- Dynamic dispatch (method calls on interfaces)
- Function pointers
- Ambiguous imports
- Macro-generated code

## GraphQL Schema for Call Graph

```graphql
type Edge {
  from: Node!
  to: Node!
  kind: EdgeKind!  # CALLS, IMPORTS, REFERENCES
  confidence: Float!
}

type Node {
  callers(limit: Int = 50): [Edge!]!
  callees(limit: Int = 50): [Edge!]!
}
```

## Performance Considerations

- Call graph queries can be expensive on large codebases
- Use `limit` parameter to cap results
- Consider filtering by file or module first
- Cache results when doing repeated analysis

## Error Handling

- If queries timeout, reduce depth or add limits
- If results seem incomplete, check if indexing is up to date
- If confidence is low, verify manually with source code
