# Claude-Specific Skills

## Claude Integration Overview

Claude Desktop integrates with spy-code through the MCP (Model Context Protocol) server. This enables Claude to understand codebase structure and provide more accurate, contextually relevant assistance.

## Configuration

Add to Claude Desktop's MCP config (typically `~/Library/Application Support/Claude/claude_desktop_config.json` on macOS):

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

## Claude-Specific Patterns

### Pattern 1: Deep Code Understanding

When Claude needs to understand code deeply:

```
User: "Explain how the authentication system works"

Claude should:
1. Use spy-code search to find auth-related functions
2. Use spy-code callers to trace the authentication flow
3. Use spy-code get to read function signatures and docs
4. Use spy-code callees to understand what auth functions call
5. Provide a comprehensive explanation with code examples
6. Offer to dive deeper into specific components
```

### Pattern 2: Context-Aware Code Generation

When generating code that integrates with existing code:

```
User: "Add a new middleware for rate limiting"

Claude should:
1. Use spy-code search to find existing middleware
2. Use spy-code callees to understand the middleware pattern
3. Analyze how middleware is registered
4. Generate code following the same pattern
5. Use spy-code to verify the integration points
6. Show where to register the new middleware
```

### Pattern 3: Comprehensive Refactoring

When refactoring code:

```
User: "Refactor the user service to use dependency injection"

Claude should:
1. Use spy-code callers to find all code using User service
2. Use spy-code get to understand the current implementation
3. Search for existing DI patterns in the codebase
4. Plan the refactoring step by step
5. Implement the changes
6. Use spy-code to verify all callers still work
7. Run spy-code index to update the graph
```

### Pattern 4: Architectural Analysis

When analyzing architecture:

```
User: "What are the architectural patterns in this codebase?"

Claude should:
1. Use spy-code stats to get an overview
2. Use spy-code graphData to visualize the structure
3. Analyze edge patterns (imports, calls, references)
4. Identify architectural patterns (layered, modular, etc.)
5. Provide architectural recommendations
6. Show visual representations
```

## Claude Tool Usage

### Recommended Tool Order

1. **Start with exploration** - Use `stats` and `search` to understand the codebase
2. **Get specific details** - Use `get_node` for specific functions/classes
3. **Understand relationships** - Use `callers`/`callees` for context
4. **Analyze changes** - Use `changed_since` for recent modifications
5. **Use semantic search** - Use `ask` for natural language queries
6. **Visualize** - Use `graphData` for complex relationships

### Claude-Specific Best Practices

1. **Provide comprehensive context** - Include file paths, line numbers, and relationships
2. **Show code examples** - Always include relevant code snippets
3. **Explain the "why"** - Not just what code does, but why it's designed that way
4. **Use visualizations** - Describe call graphs and relationships clearly
5. **Suggest next steps** - Always offer actionable next steps
6. **Handle ambiguity** - When spy-code confidence is low, explain the uncertainty

## Claude-Specific Workflows

### Workflow 1: Deep Codebase Exploration

```
User: "Help me understand this codebase"

Claude:
1. Run: stats (get overview: node count, edge count, languages)
2. Run: search "main" --kind function (find entry points)
3. Run: callees <main_id> --depth 2 (trace execution flow)
4. Run: graphData with language filter (show structure by language)
5. Provide comprehensive architecture overview
6. Identify key modules and their responsibilities
7. Suggest areas to explore based on user's goals
8. Offer to dive deeper into specific areas
```

### Workflow 2: Feature Implementation

```
User: "Implement a new feature for user notifications"

Claude:
1. Run: search "notification" (check if notifications exist)
2. Run: search "similar_feature" (find patterns to follow)
3. Analyze existing implementations using get_node and callees
4. Design the new feature following existing patterns
5. Generate code with proper integration points
6. Use callers to find where to integrate
7. Run: index (update the graph)
8. Verify integration with call graph
9. Provide testing suggestions
```

### Workflow 3: Debugging Complex Issues

```
User: "There's a memory leak in the data processing pipeline"

Claude:
1. Run: search "process" --kind function (find processing functions)
2. Run: callers <process_id> --depth 3 (trace the full pipeline)
3. Run: get_node for each function to understand resource usage
4. Identify potential leak sources (unclosed connections, large allocations)
5. Search for similar patterns that handle resources correctly
6. Suggest fixes with code examples
7. Explain how to verify the fix
8. Suggest monitoring/logging improvements
```

### Workflow 4: Comprehensive Code Review

```
User: "Review this pull request thoroughly"

Claude:
1. Run: changed_since main (find all changed code)
2. For each changed function:
   - Run: get_node (understand the change)
   - Run: callers (check impact)
   - Run: callees (check dependencies)
   - Analyze the change for correctness
3. Check for breaking changes
4. Verify patterns match codebase conventions
5. Identify potential bugs or issues
6. Suggest improvements
7. Recommend tests for affected areas
8. Provide a summary with action items
```

### Workflow 5: Architectural Refactoring

```
User: "Refactor this monolithic module into smaller services"

Claude:
1. Run: graphData with filePath filter (visualize the module)
2. Analyze the module's responsibilities using callers/callees
3. Identify natural boundaries based on call patterns
4. Search for similar service patterns in the codebase
5. Design the new architecture
6. Plan the refactoring in steps
7. Implement the changes systematically
8. Use callers to ensure all references are updated
9. Run: index to update the graph
10. Verify the new architecture with graphData
```

## Claude-Specific Enhancements

### Using Claude's Reasoning

Combine Claude's reasoning with spy-code's data:

```
User: "Is this code well-structured?"

Claude:
1. Use spy-code to get objective data (call graph, complexity)
2. Apply Claude's reasoning to evaluate structure
3. Consider SOLID principles, design patterns
4. Provide subjective assessment with evidence
5. Suggest specific improvements
```

### Natural Language Queries

Leverage Claude's NLP with spy-code's semantic search:

```
User: "How do I handle errors in this codebase?"

Claude:
1. Use spy-code ask "error handling" (semantic search)
2. Analyze the results
3. Synthesize patterns
4. Provide best practices
5. Show code examples
```

### Multi-Step Reasoning

Use spy-code data to support complex reasoning:

```
User: "Should I extract this class into a separate module?"

Claude:
1. Use spy-code callers to find usage
2. Use spy-code callees to understand dependencies
3. Analyze cohesion and coupling
4. Consider the broader architecture
5. Provide reasoned recommendation
6. Show impact analysis
```

## Claude-Specific Tips

1. **Be thorough** - Claude excels at comprehensive analysis, leverage this
2. **Provide context** - Always explain the broader context
3. **Use examples** - Show code examples for all suggestions
4. **Explain trade-offs** - Discuss pros and cons of different approaches
5. **Think step by step** - Break down complex tasks into clear steps
6. **Verify assumptions** - Use spy-code to verify assumptions about code

## Integration with Claude's Capabilities

### Artifacts

Use spy-code data in Claude's artifacts:
- Generate architecture diagrams
- Create documentation from graph data
- Build interactive visualizations

### Code Execution

Combine spy-code with Claude's code execution:
- Use spy-code to find test files
- Execute tests to verify changes
- Validate refactoring preserves behavior

### File Operations

Use spy-code to guide file operations:
- Use callers to find all files needing updates
- Verify file paths before operations
- Check for conflicts before editing

## Troubleshooting Claude Integration

### MCP server not connecting
- Verify spy-code is installed: `spy-code --version`
- Check Claude Desktop's MCP config path
- Ensure spy.config.json exists in workspace
- Restart Claude Desktop after config changes

### Tools not available in Claude
- Verify MCP server is running: `spy-code serve --mcp`
- Check Claude has MCP permissions
- Ensure tools are listed in MCP server's tool list
- Check Claude Desktop version supports MCP

### Responses don't use spy-code data
- Verify Claude is calling MCP tools
- Check tool responses are being parsed correctly
- Ensure spy-code data is being integrated into responses
- Try explicit requests to use spy-code

### Performance issues
- Use search filters to reduce query scope
- Cache frequently accessed nodes
- Consider using embeddings for semantic search
- Reduce graph query depth/limits

## Claude-Specific Limitations

1. **MCP availability** - Requires Claude Desktop with MCP support
2. **Config location** - Config location varies by OS
3. **Tool access** - Not all Claude interfaces may have MCP access
4. **Response length** - Long responses may be truncated
5. **Rate limits** - MCP calls may be rate-limited

## Advanced Claude Patterns

### Pattern: Comparative Analysis

```
User: "Compare two approaches to X"

Claude:
1. Use spy-code to find both implementations
2. Analyze their call graphs
3. Compare performance, maintainability, complexity
4. Provide detailed comparison
5. Recommend based on context
```

### Pattern: Learning from Codebase

```
User: "What patterns does this codebase use?"

Claude:
1. Use spy-code graphData to identify patterns
2. Search for specific design patterns
3. Analyze how they're implemented
4. Provide pattern catalog with examples
5. Suggest where to apply new patterns
```

### Pattern: Migration Planning

```
User: "Plan migration from X to Y"

Claude:
1. Use spy-code to find all X usage
2. Analyze dependencies using callers
3. Plan migration steps
4. Identify potential issues
5. Provide rollback plan
6. Estimate effort
```
