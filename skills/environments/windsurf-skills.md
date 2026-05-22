# Windsurf/Cascade-Specific Skills

## Windsurf Integration Overview

Windsurf (and Cascade) integrates with spy-code through the MCP (Model Context Protocol) server. The integration core library (`@spy-code/integration-core`) provides hooks and utilities specifically designed for Windsurf's agent architecture.

## Configuration

Add to Windsurf's MCP config (`.windsurf/mcp_config.json`):

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

## Integration Core Setup

Install the integration core library:

```bash
npm install @spy-code/integration-core
```

Initialize in your Windsurf agent:

```typescript
import { 
  MCPClient, 
  AgentHooks, 
  getAgentHooks,
  CacheManager,
  EventBus 
} from '@spy-code/integration-core';

// Initialize MCP client
const mcpClient = new MCPClient();
await mcpClient.connect();
await mcpClient.initialize();

// Get agent hooks instance
const agentHooks = getAgentHooks();

// Register hooks for automatic indexing
agentHooks.registerHook(HookType.POST_WRITE_CODE, async (context) => {
  // Auto re-index after code changes
  await mcpClient.callTool('index', {});
  return { continue: true };
});
```

## Windsurf-Specific Patterns

### Pattern 1: Pre-Read Code Enrichment

Use hooks to enrich context before reading code:

```typescript
agentHooks.registerHook(HookType.PRE_READ_CODE, async (context) => {
  const node = await mcpClient.getNode(context.node?.id);
  if (node) {
    // Add node information to context
    context.metadata = {
      ...context.metadata,
      nodeInfo: node,
      callers: await mcpClient.getCallers(node.id, 1),
      callees: await mcpClient.getCallees(node.id, 1)
    };
  }
  return { continue: true };
});
```

### Pattern 2: Post-Write Code Validation

Validate code changes after writing:

```typescript
agentHooks.registerHook(HookType.POST_WRITE_CODE, async (context) => {
  // Check if the change breaks call graph
  const stats = await mcpClient.getStats();
  if (stats.nodeCount === 0) {
    return { 
      continue: false, 
      error: new Error('Indexing failed after code change') 
    };
  }
  return { continue: true };
});
```

### Pattern 3: Smart Caching

Use the cache manager for performance:

```typescript
const cacheManager = new CacheManager({
  cacheEnabled: true,
  cacheTTL: 300000, // 5 minutes
  maxCacheSize: 1000
});

// Cache search results
const searchResults = await cacheManager.getOrSet(
  `search:${query}`,
  () => mcpClient.search(query)
);
```

### Pattern 4: Event-Driven Updates

Use the event bus for reactive updates:

```typescript
import { eventBus, EventType } from '@spy-code/integration-core';

eventBus.on(EventType.CODE_CHANGED, async (data) => {
  // Trigger re-index on code changes
  await mcpClient.callTool('index', {});
});

eventBus.on(EventType.NODE_FOUND, async (data) => {
  // Enrich context when a node is found
  const node = await mcpClient.getNode(data.nodeId);
  // ... process node
});
```

## Windsurf Agent Hooks

### Available Hooks for Windsurf

1. **PRE_READ_CODE** - Before reading a file, enrich with graph context
2. **POST_READ_CODE** - After reading, update cache
3. **PRE_WRITE_CODE** - Before writing, validate against call graph
4. **POST_WRITE_CODE** - After writing, trigger re-index
5. **PRE_MCP_TOOL_USE** - Before MCP tool call, add context
6. **POST_MCP_TOOL_USE** - After MCP tool call, cache results
7. **POST_SETUP_WORKTREE** - After worktree setup, index new code

### Hook Implementation Example

```typescript
import { HookType, HookContext } from '@spy-code/integration-core';

// Auto-index on worktree setup
agentHooks.registerHook(HookType.POST_SETUP_WORKTREE, async (context: HookContext) => {
  try {
    await mcpClient.callTool('index', { path: context.filePath });
    return { continue: true };
  } catch (error) {
    return { 
      continue: true,  // Don't block on indexing errors
      error: error as Error 
    };
  }
});
```

## Windsurf-Specific Workflows

### Workflow 1: Context-Aware Editing

```
User: "Refactor this function to be more efficient"

Windsurf:
1. PRE_READ_CODE hook enriches context with:
   - Function signature
   - Callers (who uses this function)
   - Callees (what this function calls)
2. Agent analyzes the function with full context
3. Agent proposes refactoring
4. PRE_WRITE_CODE hook validates changes don't break callers
5. POST_WRITE_CODE hook triggers re-index
```

### Workflow 2: Multi-File Changes

```
User: "Update all error handling to use the new error type"

Windsurf:
1. Search for all error handling code
2. Use callers to find all affected functions
3. Plan changes across multiple files
4. Apply changes systematically
5. Validate call graph remains consistent
6. Re-index to update graph
```

### Workflow 3: Feature Implementation

```
User: "Implement a new feature following existing patterns"

Windsurf:
1. Search for similar features
2. Analyze their structure using call graph
3. Generate code following patterns
4. Integrate with existing code
5. Validate integration with callers/callees
6. Update graph
```

## Windsurf Command Integration

Add these commands to Windsurf's command palette:

```typescript
// In your Windsurf extension
const commands = [
  {
    id: 'spy-code.search',
    title: 'Spy-Code: Search Codebase',
    handler: async () => {
      const query = await showInputBox();
      const results = await mcpClient.search(query);
      showQuickPick(results.map(r => r.node.name));
    }
  },
  {
    id: 'spy-code.callers',
    title: 'Spy-Code: Find Callers',
    handler: async () => {
      const nodeId = await getNodeIdFromCursor();
      const callers = await mcpClient.getCallers(nodeId);
      showCallGraph(callers);
    }
  },
  {
    id: 'spy-code.graph',
    title: 'Spy-Code: Show Graph Visualization',
    handler: async () => {
      openGraphUI();
    }
  }
];
```

## Windsurf-Specific Best Practices

1. **Use hooks for automation** - Automate indexing and validation
2. **Leverage caching** - Cache frequently accessed nodes
3. **Enrich context** - Add graph information to agent context
4. **Validate changes** - Check call graph before/after changes
5. **Handle errors gracefully** - Don't block on indexing failures

## Performance Optimization

### Lazy Loading

```typescript
// Load node details only when needed
async function getNodeDetails(nodeId: string) {
  const cached = cacheManager.get(`node:${nodeId}`);
  if (cached) return cached;
  
  const node = await mcpClient.getNode(nodeId);
  cacheManager.set(`node:${nodeId}`, node);
  return node;
}
```

### Batch Operations

```typescript
// Batch multiple queries
async function batchSearch(queries: string[]) {
  return Promise.all(
    queries.map(q => mcpClient.search(q))
  );
}
```

### Background Indexing

```typescript
// Index in background without blocking
agentHooks.registerHook(HookType.POST_WRITE_CODE, async (context) => {
  // Don't await indexing
  mcpClient.callTool('index', {}).catch(console.error);
  return { continue: true };
});
```

## Troubleshooting Windsurf Integration

### Integration core not found
- Verify npm package is installed: `npm list @spy-code/integration-core`
- Check package.json dependencies
- Reinstall if needed: `npm install @spy-code/integration-core`

### Hooks not firing
- Verify hooks are registered before agent actions
- Check `agentHooks.isEnabled()` returns true
- Ensure hook types match Windsurf's lifecycle

### Cache issues
- Clear cache: `cacheManager.clear()`
- Check cache TTL settings
- Verify cache key uniqueness

### MCP connection issues
- Verify spy-code is installed
- Check MCP config in `.windsurf/mcp_config.json`
- Test MCP server: `spy-code serve --mcp`

## Windsurf-Specific Tips

1. **Combine with Windsurf's file awareness** - Use Windsurf's file tracking with spy-code's graph
2. **Use worktrees** - Leverage Windsurf's worktree support with spy-code's git integration
3. **Integrate with Cascade** - Use spy-code data in Cascade's decision-making
4. **Custom hooks** - Create custom hooks for your specific workflows
5. **Monitor events** - Use event bus to monitor and react to changes
