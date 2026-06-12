# GitHub Copilot-Specific Skills

## Copilot Integration Overview

GitHub Copilot integrates with spy-code through the MCP (Model Context Protocol) server. This enables Copilot Chat to understand codebase structure and provide more contextually relevant suggestions.

## Configuration

Add to Copilot's MCP config (location varies by platform):

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

**Note**: Copilot's MCP integration is evolving. Check Copilot documentation for the exact config location.

## Copilot-Specific Patterns

### Pattern 1: Context-Aware Code Completion

When Copilot suggests code:

```
User: types "function process"

Copilot should:
1. Use spy-code search to find existing "process" functions
2. Analyze their signatures and patterns
3. Suggest code that matches the codebase style
4. Ensure the suggestion integrates with existing call graph
```

### Pattern 2: Chat-Based Code Exploration

In Copilot Chat:

```
User: "Show me how authentication works"

Copilot should:
1. Use spy-code search to find auth functions
2. Use spy-code callers to trace the flow
3. Use spy-code get to show signatures
4. Provide an explanation with code examples
5. Offer to navigate to specific files
```

### Pattern 3: Refactoring Suggestions

```
User: "Suggest refactoring for this function"

Copilot should:
1. Use spy-code callers to find all uses
2. Analyze the function's complexity
3. Check for similar patterns in the codebase
4. Suggest refactoring that maintains compatibility
5. Show impact analysis
```

### Pattern 4: Test Generation

```
User: "Generate tests for this function"

Copilot should:
1. Use spy-code get to understand the function signature
2. Use spy-code callees to understand dependencies
3. Find similar test patterns in the codebase
4. Generate tests that match the project's testing style
5. Ensure tests cover all callers' use cases
```

## Copilot Chat Integration

### Example Chat Prompts

**Understanding Code**:
```
"How does the payment processing work in this codebase?"
→ Copilot uses spy-code to find payment functions, trace call graph, explain flow
```

**Finding Code**:
```
"Where is the user validation logic?"
→ Copilot uses spy-code search to find validation functions
```

**Architecture Questions**:
```
"What are the main modules in this project?"
→ Copilot uses spy-code stats and graph data to explain architecture
```

**Impact Analysis**:
```
"What will break if I change the User model?"
→ Copilot uses spy-code callers to find all code using User
```

## Copilot Tool Usage

### Recommended Tool Order in Chat

1. **Understand the question** - Parse user intent
2. **Search for relevant code** - Use `search` to find nodes
3. **Get context** - Use `get_node` for details
4. **Analyze relationships** - Use `callers`/`callees` for context
5. **Provide answer** - Combine spy-code data with Copilot's knowledge

### Copilot-Specific Best Practices

1. **Combine with Copilot's training** - Use spy-code for project-specific context
2. **Provide file references** - Always include file paths in responses
3. **Show code snippets** - Use spy-code data to show relevant code
4. **Suggest navigation** - Offer to open files in the editor
5. **Explain relationships** - Use call graph to explain connections

## Copilot-Specific Workflows

### Workflow 1: Onboarding New Developers

```
User: "Help me understand this codebase"

Copilot:
1. Run: stats (get overview)
2. Run: search "main" (find entry points)
3. Run: callees <main> (trace execution)
4. Provide architecture overview
5. Suggest starting points for exploration
6. Offer to explain specific modules
```

### Workflow 2: Implementing Features

```
User: "I need to add a new API endpoint"

Copilot:
1. Search for existing API endpoints
2. Analyze their patterns using call graph
3. Suggest following the same pattern
4. Generate code matching the style
5. Show where to integrate it
6. Verify with call graph
```

### Workflow 3: Debugging

```
User: "I'm getting an error in the authentication flow"

Copilot:
1. Search for authentication functions
2. Trace the call flow using callers/callees
3. Identify potential error sources
4. Suggest debugging locations
5. Show similar error handling patterns
6. Help add logging if needed
```

### Workflow 4: Code Review

```
User: "Review this pull request"

Copilot:
1. Use changed_since to find modified code
2. For each change:
   - Check callers for impact
   - Verify patterns match codebase
   - Check for breaking changes
3. Provide review comments
4. Suggest improvements
5. Identify missing tests
```

## Copilot Extensions

### Custom Copilot Extensions

Create a Copilot extension that uses spy-code:

```typescript
// copilot-extension.ts
import { CopilotExtension } from '@copilot/extension';
import { MCPClient } from '@spy-code/integration-core';

class SpyCodeExtension implements CopilotExtension {
  private mcpClient: MCPClient;

  async initialize() {
    this.mcpClient = new MCPClient();
    await this.mcpClient.connect();
  }

  async handleChat(query: string) {
    // Use spy-code to enhance responses
    const results = await this.mcpClient.search(query);
    return this.formatResults(results);
  }

  async handleCodeCompletion(context: any) {
    // Use spy-code for context-aware completions
    const similar = await this.mcpClient.search(context.prefix);
    return this.generateCompletion(context, similar);
  }
}
```

## Copilot-Specific Tips

1. **Use Copilot's file awareness** - Combine with spy-code's graph context
2. **Leverage Copilot's training** - Use spy-code for project-specific knowledge
3. **Provide actionable suggestions** - Always suggest next steps
4. **Show don't just tell** - Include code examples in responses
5. **Maintain Copilot's tone** - Keep responses helpful and concise

## Integration with Copilot Features

### Copilot Workspace

Use spy-code to enhance Copilot Workspace:
- Provide codebase overview
- Show module relationships
- Identify key files
- Suggest starting points

### Copilot Pull Request Summaries

Enhance PR summaries with spy-code:
- Show changed functions
- List affected callers
- Identify potential impacts
- Suggest review focus areas

### Copilot Code Explanation

Enhance code explanations with spy-code:
- Show function signatures
- Explain call relationships
- Identify dependencies
- Show usage patterns

## Troubleshooting Copilot Integration

### MCP server not available
- Verify spy-code is installed
- Check Copilot's MCP config location
- Ensure MCP is enabled in Copilot settings

### Chat responses don't use spy-code
- Verify MCP server is running
- Check Copilot has MCP permissions
- Ensure spy-code tools are listed in Copilot

### Performance issues
- Use search filters to reduce scope
- Cache frequently accessed nodes
- Consider using embeddings for semantic search

### Context not included in responses
- Verify spy-code tools are being called
- Check tool responses are being parsed
- Ensure results are being formatted correctly

## Copilot-Specific Limitations

1. **MCP availability** - MCP integration may not be available in all Copilot versions
2. **Config location** - MCP config location varies by platform and version
3. **Tool access** - Not all Copilot features may have access to MCP tools
4. **Response length** - Copilot may truncate long responses
5. **Rate limits** - MCP calls may be rate-limited

## Future Enhancements

As Copilot's MCP integration evolves:
- Direct integration with Copilot's code completion
- Real-time call graph visualization in Copilot UI
- Automatic re-indexing on file changes
- Integration with Copilot's testing features
- Enhanced PR review capabilities

## When to Use

Use this skill when you need to:
- Get context-aware autocomplete suggestions
- Ask specific questions about codebase architecture in Copilot Chat
- Trace caller and callee workflows without leaving the editor
- Determine the scope of a feature change

## Available Tools

### MCP Tools
- `search` - Find code symbols
- `get_node` - Retrieve code details
- `callers` - Get functions calling a node
- `callees` - Get functions called by a node

### CLI Commands (Local Use)
To use `spy-code` manually alongside Copilot:
```bash
spy-code search "payment"
spy-code callers <node_id>
```

### Running MCP Locally
To run the MCP server for Copilot:
```bash
spy-code serve --mcp
```

## Best Practices

1. **Ask Contextual Questions**: Tell Copilot to "use spy-code to find the callers of X".
2. **Use CLI for Context Feeding**: Run CLI commands in the terminal to copy/paste specific call graphs into Copilot chat.
3. **Keep Context Localized**: Only fetch nodes necessary for the current completion task.
