# MCP Server Setup Guide

This guide covers setting up and configuring the spy-code MCP (Model Context Protocol) server for AI coding environments.

## What is MCP?

The Model Context Protocol (MCP) is a standardized protocol for AI assistants to interact with external tools and resources. Spy-Code implements an MCP server that exposes codebase intelligence capabilities through a standardized interface.

## MCP Server Overview

The spy-code MCP server provides:

- **Tools**: Interactive functions for querying the codebase graph
- **Resources**: Static data like schema, stats, and configuration
- **Stdio Transport**: Communication over standard input/output

## Starting the MCP Server

### Manual Start

```bash
spy-code serve --mcp
```

The server will:
1. Read `spy.config.json` from the current directory
2. Open the graph database in read-only mode
3. Listen for JSON-RPC 2.0 messages on stdin
4. Respond with tool results and resources

### Environment Variables

- `SPY_CODE_CONFIG_PATH` - Path to spy.config.json (default: `./spy.config.json`)
- `SPY_CODE_DB_PATH` - Path to graph database (default: from config)

## MCP Tools Reference

### query_graph

Run a raw GraphQL query against the codebase graph.

**Parameters:**
```json
{
  "query": "string (required)",
  "variables": "object (optional)"
}
```

**Example:**
```json
{
  "query": "{ node(id: \"src:auth.rs:_:login\") { name description } }"
}
```

**Use when:** You need complex multi-hop queries or want full GraphQL power.

---

### get_node

Fetch a single node by its ID.

**Parameters:**
```json
{
  "node_id": "string (required)"
}
```

**Example:**
```json
{
  "node_id": "src:auth.rs:_:login"
}
```

**Returns:** Node object with name, description, signatures, location, etc.

**Use when:** You know the exact node ID and need its details.

---

### search

Find nodes by fuzzy name/description match.

**Parameters:**
```json
{
  "query": "string (required)",
  "kind": "string (optional)",
  "limit": "integer (optional)"
}
```

**Example:**
```json
{
  "query": "authenticate",
  "kind": "function",
  "limit": 20
}
```

**Returns:** Array of search results with nodes and scores.

**Use when:** You know roughly what something is called but not its exact ID.

---

### find_callers

List all functions/methods that call the given node.

**Parameters:**
```json
{
  "node_id": "string (required)",
  "depth": "integer (optional, default: 1)"
}
```

**Example:**
```json
{
  "node_id": "src:auth.rs:_:login",
  "depth": 2
}
```

**Returns:** Array of edges showing caller relationships.

**Use when:** Understanding impact of changes or tracing who uses a function.

---

### find_callees

List all functions/methods called by the given node.

**Parameters:**
```json
{
  "node_id": "string (required)",
  "depth": "integer (optional, default: 1)"
}
```

**Example:**
```json
{
  "node_id": "src:auth.rs:_:login",
  "depth": 2
}
```

**Returns:** Array of edges showing callee relationships.

**Use when:** Tracing execution flow or understanding dependencies.

---

### changed_since

List nodes whose source changed since a given git ref.

**Parameters:**
```json
{
  "git_ref": "string (required)"
}
```

**Example:**
```json
{
  "git_ref": "HEAD~5"
}
```

**Returns:** Array of changed nodes.

**Use when:** Finding what changed after rebase/merge or identifying affected code.

---

### stats

Get index statistics.

**Parameters:**
```json
{}
```

**Returns:** Object with nodeCount, edgeCount, fileCount, lastIndexed, lastGitSha.

**Use when:** Checking if indexing is complete or understanding codebase size.

---

### ask

Ask natural language questions about the codebase (requires embeddings).

**Parameters:**
```json
{
  "query": "string (required)",
  "limit": "integer (optional, default: 20)"
}
```

**Example:**
```json
{
  "query": "how do I authenticate users?",
  "limit": 10
}
```

**Returns:** Array of search results based on semantic similarity.

**Use when:** You have conceptual questions or don't know the terminology.

---

### embed

Generate embeddings for semantic search.

**Parameters:**
```json
{
  "full": "boolean (optional, default: false)"
}
```

**Example:**
```json
{
  "full": true
}
```

**Returns:** Success/failure status.

**Use when:** You want to use semantic search (`ask` tool).

## MCP Resources Reference

### spy-code://schema

Full GraphQL SDL schema.

**Use when:** You need to understand the available GraphQL queries and types.

**Example response:**
```graphql
type Node {
  id: NodeID!
  kind: NodeKind!
  name: String!
  description: String
  signatures: [Signature!]!
  # ... more fields
}
```

---

### spy-code://stats

Current index statistics.

**Use when:** Checking index status or codebase size.

**Example response:**
```json
{
  "nodeCount": 1234,
  "edgeCount": 5678,
  "fileCount": 89,
  "lastIndexed": "2024-01-15T10:30:00Z",
  "lastGitSha": "abc123def456"
}
```

---

### spy-code://config

Loaded configuration (sanitized).

**Use when:** Debugging configuration issues or understanding current settings.

**Example response:**
```json
{
  "version": 1,
  "db_path": ".spy-code/graph.db",
  "languages": {
    "rust": {
      "enabled": true,
      "roots": ["src/"]
    }
  }
}
```

## MCP Protocol Details

### Initialization

The MCP server expects an initialization handshake:

```json
{
  "jsonrpc": "2.0",
  "id": 1,
  "method": "initialize",
  "params": {
    "protocolVersion": "2024-11-05",
    "capabilities": {
      "tools": {},
      "resources": {}
    },
    "clientInfo": {
      "name": "your-client-name",
      "version": "1.0.0"
    }
  }
}
```

After receiving the initialization response, send:

```json
{
  "jsonrpc": "2.0",
  "id": 2,
  "method": "initialized",
  "params": {}
}
```

### Tool Call

```json
{
  "jsonrpc": "2.0",
  "id": 3,
  "method": "tools/call",
  "params": {
    "name": "search",
    "arguments": {
      "query": "authenticate"
    }
  }
}
```

### Resource Read

```json
{
  "jsonrpc": "2.0",
  "id": 4,
  "method": "resources/read",
  "params": {
    "uri": "spy-code://schema"
  }
}
```

## Configuration by Environment

### Cursor

Config file: `.cursor/mcp_config.json`

```json
{
  "mcpServers": {
    "spy-code": {
      "command": "spy-code",
      "args": ["serve", "--mcp"],
      "env": {
        "SPY_CODE_CONFIG_PATH": "${workspaceFolder}/spy.config.json"
      }
    }
  }
}
```

### Windsurf/Cascade

Config file: `.windsurf/mcp_config.json`

```json
{
  "mcpServers": {
    "spy-code": {
      "command": "spy-code",
      "args": ["serve", "--mcp"],
      "env": {
        "SPY_CODE_CONFIG_PATH": "${workspaceFolder}/spy.config.json"
      }
    }
  }
}
```

### Claude Desktop

Config file: `~/Library/Application Support/Claude/claude_desktop_config.json` (macOS) or `~/.config/Claude/claude_desktop_config.json` (Linux)

```json
{
  "mcpServers": {
    "spy-code": {
      "command": "spy-code",
      "args": ["serve", "--mcp"],
      "env": {
        "SPY_CODE_CONFIG_PATH": "${workspaceFolder}/spy.config.json"
      }
    }
  }
}
```

### GitHub Copilot

Config file: Location varies by platform and version

```json
{
  "mcpServers": {
    "spy-code": {
      "command": "spy-code",
      "args": ["serve", "--mcp"],
      "env": {
        "SPY_CODE_CONFIG_PATH": "${workspaceFolder}/spy.config.json"
      }
    }
  }
}
```

## Security Considerations

### Read-Only Access

The MCP server opens the graph database in read-only mode. It cannot:
- Modify source code
- Delete files
- Change the graph database
- Execute arbitrary commands

### Workspace Isolation

The server operates within the workspace context:
- Only accesses files in the configured workspace
- Respects ignore patterns from `spy.config.json`
- Does not access files outside the workspace

### Environment Variables

Be careful with environment variables in MCP config:
- Don't expose sensitive paths
- Use `${workspaceFolder}` for workspace-relative paths
- Avoid hardcoding absolute paths

## Troubleshooting

### Server Won't Start

**Symptoms:** MCP server fails to start or immediately exits

**Solutions:**
1. Check spy-code is installed: `spy-code --version`
2. Verify `spy.config.json` exists and is valid
3. Check graph database exists: `.spy-code/graph.db`
4. Run `spy-code index` if database doesn't exist
5. Check file permissions on config and database

### Tools Not Available

**Symptoms:** AI environment doesn't show spy-code tools

**Solutions:**
1. Restart your AI coding environment
2. Verify MCP config file is in correct location
3. Check MCP config is valid JSON
4. Ensure spy-code is in PATH
5. Check AI environment supports MCP

### Queries Return No Results

**Symptoms:** Search or query tools return empty results

**Solutions:**
1. Run `spy-code stats` to check if indexing is complete
2. Run `spy-code index` to index your codebase
3. Check `spy.config.json` has correct language settings
4. Verify file paths in config are correct
5. Check ignore patterns aren't too broad

### Performance Issues

**Symptoms:** Slow queries, timeouts

**Solutions:**
1. Use filters to reduce query scope
2. Reduce graph query depth/limits
3. Check if embeddings are needed for semantic search
4. Consider indexing only necessary directories
5. Check system resources (CPU, memory)

### Connection Errors

**Symptoms:** MCP client can't connect to server

**Solutions:**
1. Verify spy-code command works in terminal
2. Check environment variables in MCP config
3. Ensure no firewall blocking stdio communication
4. Check for conflicting MCP servers
5. Try restarting the AI environment

## Advanced Configuration

### Custom Server Command

You can wrap the spy-code command in a script:

```json
{
  "mcpServers": {
    "spy-code": {
      "command": "/path/to/wrapper-script.sh",
      "args": []
    }
  }
}
```

### Multiple Workspaces

For multiple workspaces, configure each separately:

```json
{
  "mcpServers": {
    "spy-code-project1": {
      "command": "spy-code",
      "args": ["serve", "--mcp"],
      "env": {
        "SPY_CODE_CONFIG_PATH": "/path/to/project1/spy.config.json"
      }
    },
    "spy-code-project2": {
      "command": "spy-code",
      "args": ["serve", "--mcp"],
      "env": {
        "SPY_CODE_CONFIG_PATH": "/path/to/project2/spy.config.json"
      }
    }
  }
}
```

### Logging

Enable debug logging by setting environment variable:

```json
{
  "mcpServers": {
    "spy-code": {
      "command": "spy-code",
      "args": ["serve", "--mcp"],
      "env": {
        "SPY_CODE_CONFIG_PATH": "${workspaceFolder}/spy.config.json",
        "RUST_LOG": "debug"
      }
    }
  }
}
```

## Testing MCP Server

### Manual Test

```bash
# Start server
spy-code serve --mcp

# In another terminal, send initialization
echo '{"jsonrpc":"2.0","id":1,"method":"initialize","params":{"protocolVersion":"2024-11-05","capabilities":{},"clientInfo":{"name":"test","version":"1.0"}}}' | spy-code serve --mcp

# List tools
echo '{"jsonrpc":"2.0","id":2,"method":"tools/list"}' | spy-code serve --mcp

# Call a tool
echo '{"jsonrpc":"2.0","id":3,"method":"tools/call","params":{"name":"stats","arguments":{}}}' | spy-code serve --mcp
```

### Using MCP Inspector

If you have the MCP inspector tool:

```bash
mcp-inspector spy-code serve --mcp
```

This provides an interactive UI for testing MCP servers.

## Best Practices

1. **Index Before Use** - Always run `spy-code index` before using MCP tools
2. **Use Specific Tools** - Prefer specific tools (get_node, search) over raw GraphQL
3. **Filter Results** - Use kind and limit parameters to reduce result size
4. **Cache Results** - Cache frequently accessed nodes in your client
5. **Handle Errors** - Always handle errors gracefully in your MCP client
6. **Validate Config** - Validate MCP config JSON before using
7. **Monitor Performance** - Monitor query performance and optimize as needed

## Next Steps

- [Integrations Guide](./INTEGRATIONS.md) - Overview of all integrations
- [Agent Usage Guide](./AGENT_USAGE.md) - How agents should use spy-code
- [Skill Reference](./SKILL_REFERENCE.md) - Complete skill catalog
- [CLI_MCP.md](./CLI_MCP.md) - CLI and MCP interface details
