# OpenCode-Specific Skills

## OpenCode Integration Overview

OpenCode can utilize `spy-code` to navigate codebases natively, understanding the full semantic and structural graph. This is achieved by either using the MCP integration or by integrating `spy-code` as a standalone CLI tool in OpenCode workflows.

## Configuration & Setup

### Running MCP Locally
To serve the MCP protocol locally for OpenCode:
```bash
spy-code serve --mcp
```
Configure your OpenCode settings to point to this MCP server, ensuring `SPY_CODE_CONFIG_PATH` points to the `spy.config.json` in your workspace root.

### Using as a CLI Tool / Downloadable Skill
`spy-code` can be used as a CLI tool within OpenCode tasks. Use it to gather precise contextual information instead of performing broad `grep` searches:
```bash
# Example: Find exactly where a struct is used
spy-code callers src:models.rs:_:User
```

## When to Use

Use this skill when you need to:
- Conduct deep architectural reviews in OpenCode
- Perform precise dependency analysis rather than text-based searching
- Determine transitive impacts of a change across a large codebase
- Gather exact context for multi-file refactoring
- Ask natural language questions about the codebase architecture

## Available Tools

### MCP / Programmatic Tools
- `search` - Find elements logically
- `callers` & `callees` - Trace code flows
- `changed_since` - Scope reviews to only what has changed
- `stats` & `graphData` - Retrieve analytical and visual overview data

### CLI Commands
- `spy-code search <query>`
- `spy-code callers <node_id>`
- `spy-code stats`
- `spy-code graph`

## Best Practices

1. **Replace Grep with Spy-Code**: Do not use `grep` or `find`. Rely on `spy-code search` and graph traversals to ensure you get semantically correct results without false positives.
2. **Understand Data Impact**: Use the call graph to verify that changes will not break transitive dependencies.
3. **Verify MCP Connectivity**: Ensure the local MCP server is running if using OpenCode's interactive graph features.
4. **Targeted Context**: Always pull only the necessary nodes and their direct relationships into the prompt context to maximize efficiency.
