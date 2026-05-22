# Code Navigation Skill

## When to Use

Use this skill when you need to:
- Find functions, classes, or constants in the codebase
- Navigate to specific code locations
- Understand the structure of the codebase
- Locate implementation details
- Find where a symbol is defined

## Available Tools

### MCP Tools
- `search` - Find nodes by fuzzy name/description match
- `get_node` - Fetch one node by its ID
- `query_graph` - Run raw GraphQL queries

### CLI Commands
- `spy-code search <text>` - Search for symbols
- `spy-code get <node_id>` - Get node details
- `spy-code query '<graphql>'` - Run GraphQL query

## Example Queries

### Find authentication functions
```bash
spy-code search "auth" --kind function
```

### Get specific node details
```bash
spy-code get src:auth.rs:_:login
```

### GraphQL query for node with signature
```graphql
{
  node(id: "src:auth.rs:_:login") {
    name
    description
    signatures {
      params { name type }
      returns
    }
    filePath
    startLine
    endLine
  }
}
```

### Search by description
```bash
spy-code search "user authentication"
```

## Best Practices

1. **Start with search** - Use `search` when you don't know the exact node ID
2. **Use kind filters** - Filter by `function`, `class`, or `constant` to narrow results
3. **Check node IDs** - Node IDs follow the format `dir:file:class:symbol`
4. **Review signatures** - Always check function signatures before using
5. **Verify file paths** - Ensure you're looking at the right file/location

## Common Patterns

### Pattern 1: Find and Inspect
```bash
# Search for the symbol
spy-code search "process_request"

# Get detailed information
spy-code get src:api:handlers:process_request

# Read the file at the location
# (use your file reading tool)
```

### Pattern 2: Explore by File
```graphql
{
  files
}
```

Then search within specific files:
```bash
spy-code search "function_name" --kind function
```

### Pattern 3: Find Related Code
```bash
# Find a class
spy-code search "User" --kind class

# Get the class node
spy-code get src:models:User

# Look at its methods via signatures
```

## Node ID Format

Node IDs use the format: `directory:file:class:symbol`

- `directory` - Relative path from repo root
- `file` - Filename without extension
- `class` - Class name (or `_` if not in a class)
- `symbol` - Function/constant name

Examples:
- `src:auth.rs:_:login` - Function `login` in `src/auth.rs`
- `src:models:User:save` - Method `save` in class `User` in `src/models`
- `config:constants:_:API_KEY` - Constant `API_KEY` in `config/constants`

## Error Handling

- If `search` returns no results, try broader terms
- If `get_node` returns null, verify the node ID format
- If GraphQL queries fail, check the schema with `spy-code://schema` resource
