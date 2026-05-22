# Cursor-Specific Skills

## Cursor Integration Overview

Cursor integrates with spy-code through the MCP (Model Context Protocol) server. Configure spy-code in Cursor's MCP settings to enable these skills.

## Configuration

Add to Cursor's MCP config (`.cursor/mcp_config.json`):

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

## Cursor-Specific Patterns

### Pattern 1: Code Understanding in Chat

When Cursor asks about code structure:

```
User: "How does authentication work in this codebase?"

Cursor should:
1. Use spy-code search to find auth-related functions
2. Use spy-code callers to understand the call flow
3. Use spy-code get to read function signatures
4. Provide a comprehensive explanation
```

### Pattern 2: Context-Aware Code Generation

When generating code that needs to integrate with existing code:

```
User: "Add a new endpoint for user profile updates"

Cursor should:
1. Use spy-code search to find existing user endpoints
2. Use spy-code callees to understand the pattern
3. Generate code following the same pattern
4. Use spy-code to verify the new code fits the structure
```

### Pattern 3: Refactoring Assistance

When refactoring code:

```
User: "Extract the validation logic into a separate function"

Cursor should:
1. Use spy-code callers to find all uses of the validation
2. Extract the function
3. Update all callers
4. Use spy-code to verify the call graph is correct
```

### Pattern 4: Impact Analysis

Before making changes:

```
User: "I want to change the User model"

Cursor should:
1. Use spy-code callers to find all code using User
2. Use spy-code changed_since to see recent changes
3. Assess the impact
4. Warn about potential breaking changes
```

## Cursor Tool Usage

### Recommended Tool Order

1. **Start with search** - `search` to find relevant code
2. **Get details** - `get_node` for specific nodes
3. **Understand relationships** - `callers`/`callees` for context
4. **Verify changes** - `changed_since` after modifications

### Cursor-Specific Best Practices

1. **Always provide context** - Include file paths and line numbers in responses
2. **Show call graphs** - Visualize relationships when explaining code
3. **Use semantic search** - When user asks conceptual questions
4. **Check confidence** - Warn about low-confidence edges
5. **Suggest related code** - Use search to find similar patterns

## Example Cursor Workflows

### Workflow 1: Understanding a Codebase

```
User: "Help me understand this codebase"

Cursor:
1. Run: spy-code stats (get overview)
2. Run: spy-code search "main" --kind function (find entry points)
3. Run: spy-code callees <main_id> --depth 2 (trace execution)
4. Provide high-level architecture overview
5. Offer to explore specific modules
```

### Workflow 2: Implementing a Feature

```
User: "Add a new feature for X"

Cursor:
1. Run: spy-code search "X" (check if exists)
2. Run: spy-code search "similar_feature" (find patterns)
3. Analyze existing implementations
4. Generate code following patterns
5. Run: spy-code index (update graph)
6. Verify integration with call graph
```

### Workflow 3: Debugging

```
User: "There's a bug in the authentication flow"

Cursor:
1. Run: spy-code search "authenticate" (find auth functions)
2. Run: spy-code callers <auth_id> --depth 2 (trace call flow)
3. Identify potential failure points
4. Suggest debugging locations
5. Help add logging/error handling
```

### Workflow 4: Code Review

```
User: "Review this pull request"

Cursor:
1. Run: spy-code changed_since main (find changed code)
2. For each changed function:
   - Run: spy-code callers <func_id> (check impact)
   - Analyze the changes
   - Check for breaking changes
3. Provide review comments
4. Suggest tests for affected areas
```

## Cursor Command Palette Integration

Add these commands to Cursor's command palette:

```json
{
  "commands": [
    {
      "title": "Spy-Code: Search Codebase",
      "command": "spy-code.search"
    },
    {
      "title": "Spy-Code: Find Callers",
      "command": "spy-code.callers"
    },
    {
      "title": "Spy-Code: Find Callees",
      "command": "spy-code.callees"
    },
    {
      "title": "Spy-Code: Show Call Graph",
      "command": "spy-code.graph"
    },
    {
      "title": "Spy-Code: Re-index",
      "command": "spy-code.index"
    }
  ]
}
```

## Cursor-Specific Tips

1. **Use @ mentions** - Reference files and functions Cursor highlights
2. **Leverage Cursor's context** - Combine Cursor's file awareness with spy-code's graph
3. **Inline suggestions** - Use spy-code data to improve Cursor's autocomplete
4. **Chat explanations** - Use spy-code to provide detailed architectural context
5. **Multi-file edits** - Use spy-code callers to find all files needing updates

## Troubleshooting Cursor Integration

### MCP server not connecting
- Verify spy-code is installed: `spy-code --version`
- Check MCP config path in Cursor settings
- Ensure spy.config.json exists in workspace

### Tools not available
- Restart Cursor after adding MCP config
- Check MCP server is running: `spy-code serve --mcp`
- Verify Cursor has MCP permissions

### Search returns no results
- Ensure codebase is indexed: `spy-code index`
- Check spy.config.json has correct language settings
- Verify workspace path is correct

### Performance issues
- Use filters to reduce query scope
- Consider re-indexing with `--full` flag
- Check if embeddings are needed for semantic search
