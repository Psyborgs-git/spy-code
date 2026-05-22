# Agent Usage Guide

This guide explains how AI coding agents should use spy-code to provide better assistance.

## Agent Integration Principles

### 1. Context Enrichment

Agents should enrich their context with spy-code data before making decisions:

```typescript
// Before reading a file
const node = await mcpClient.getNode(nodeId);
const callers = await mcpClient.getCallers(nodeId, 1);
const callees = await mcpClient.getCallees(nodeId, 1);

// Use this context to understand:
// - What this function does
// - Who uses it
// - What it depends on
```

### 2. Incremental Understanding

Start broad, then narrow down:

```typescript
// Step 1: Search broadly
const results = await mcpClient.search("authentication");

// Step 2: Get specific node
const node = await mcpClient.getNode(results[0].node.id);

// Step 3: Understand relationships
const callers = await mcpClient.getCallers(node.id, 2);
```

### 3. Impact Analysis

Before suggesting changes, analyze impact:

```typescript
// Find all callers
const callers = await mcpClient.getCallers(nodeId, 2);

// Assess impact
const impact = callers.length;
const criticalCallers = callers.filter(c => c.confidence < 1.0);

// Warn user if impact is high
if (impact > 10) {
  warnUser(`This change affects ${impact} functions`);
}
```

### 4. Pattern Recognition

Use spy-code to find and follow patterns:

```typescript
// Find similar implementations
const similar = await mcpClient.search("validate");

// Analyze the pattern
const pattern = similar.map(s => ({
  name: s.node.name,
  signature: s.node.signatures[0],
  location: s.node.filePath
}));

// Apply the pattern to new code
```

## Tool Usage Guidelines

### When to Use Each Tool

| Tool | Use When | Example |
|------|----------|---------|
| `search` | You don't know exact names | "Find auth functions" |
| `get_node` | You have the node ID | Get details for `src:auth.rs:_:login` |
| `find_callers` | Understanding impact | "Who uses this function?" |
| `find_callees` | Tracing execution | "What does this call?" |
| `changed_since` | After rebase/merge | "What changed since main?" |
| `ask` | Conceptual questions | "How do I handle errors?" |
| `query_graph` | Complex queries | Multi-hop graph traversal |

### Tool Selection Flow

```
User Request
    ↓
Is it a natural language question?
    ├─ Yes → Use `ask` (if embeddings available)
    │         or `search` with keywords
    └─ No → Do you have a node ID?
              ├─ Yes → Use `get_node`
              └─ No → Use `search`
                       ↓
                    Need relationships?
                       ├─ Yes → Use `find_callers`/`find_callees`
                       └─ No → Done
```

## Query Construction

### Search Queries

**Good search queries:**
- "authenticate" - Single keyword
- "user validation" - Two keywords
- "process request" - Common phrase

**Less effective:**
- "how do I" - Too generic
- "the thing that" - Too vague
- Very long phrases - Better to use `ask`

### GraphQL Queries

**Simple node query:**
```graphql
{
  node(id: "src:auth.rs:_:login") {
    name
    description
    filePath
  }
}
```

**Node with signatures:**
```graphql
{
  node(id: "src:auth.rs:_:login") {
    name
    signatures {
      params { name type }
      returns
    }
  }
}
```

**Call graph query:**
```graphql
{
  callers(id: "src:auth.rs:_:login", depth: 2) {
    from { name filePath }
    to { name }
    kind
    confidence
  }
}
```

**Filtered graph query:**
```graphql
{
  graphData(filter: {
    filePath: "src/auth.rs"
    nodeKinds: [FUNCTION]
    edgeKinds: [CALLS]
  }) {
    nodes { name }
    edges { from { name } to { name } }
  }
}
```

## Common Agent Workflows

### Workflow 1: Understanding a Codebase

```
1. Get overview
   → stats (nodeCount, edgeCount, fileCount)

2. Find entry points
   → search "main" --kind function

3. Trace execution
   → callees <main_id> --depth 2

4. Identify key modules
   → search by module name

5. Provide architecture overview
   → Synthesize findings
```

### Workflow 2: Implementing a Feature

```
1. Check if feature exists
   → search "feature_name"

2. Find similar patterns
   → search "similar_feature"

3. Analyze existing implementations
   → get_node for each result
   → callees to understand dependencies

4. Generate code following patterns
   → Apply identified patterns

5. Verify integration
   → callers to check integration points
   → index to update graph

6. Validate
   → Query graph to verify structure
```

### Workflow 3: Debugging

```
1. Find relevant functions
   → search "error" or specific symptom

2. Trace execution flow
   → callees <function_id> --depth 2

3. Identify potential issues
   → Check low-confidence edges
   → Look for missing error handling

4. Suggest fixes
   → Find similar error handling patterns
   → Apply patterns

5. Verify fix
   → Check callers still work
```

### Workflow 4: Refactoring

```
1. Analyze current code
   → get_node <function_id>
   → callers to understand usage
   → callees to understand dependencies

2. Find refactoring patterns
   → search for similar refactored code

3. Plan changes
   → Identify all affected code
   → Plan step-by-step changes

4. Implement changes
   → Apply refactoring
   → Update all callers

5. Verify
   → index to update graph
   → Query to verify structure
   → Check callers still valid
```

### Workflow 5: Code Review

```
1. Find changed code
   → changed_since <git_ref>

2. For each change:
   a. Get node details
      → get_node <node_id>
   
   b. Check impact
      → callers <node_id> --depth 2
   
   c. Analyze changes
      → Compare with patterns
      → Check for breaking changes
   
   d. Identify issues
      → Low confidence edges
      → Missing error handling
      → Inconsistent patterns

3. Provide review
   → Summarize findings
   → Suggest improvements
   → Recommend tests
```

## Error Handling

### Handle Missing Nodes

```typescript
const node = await mcpClient.getNode(nodeId);
if (!node) {
  // Node doesn't exist
  // Might need to re-index
  await mcpClient.callTool('index', {});
  // Retry
  const node = await mcpClient.getNode(nodeId);
}
```

### Handle Low Confidence

```typescript
const callers = await mcpClient.getCallers(nodeId, 2);
const uncertain = callers.filter(c => c.confidence < 1.0);

if (uncertain.length > 0) {
  // Warn user about uncertainty
  warnUser(`Some relationships are uncertain (${uncertain.length} edges)`);
  // Suggest manual verification
}
```

### Handle Timeouts

```typescript
try {
  const result = await mcpClient.callTool('search', { query: term });
} catch (error) {
  if (error.code === 'TIMEOUT') {
    // Retry with filters
    const result = await mcpClient.callTool('search', { 
      query: term,
      limit: 10 
    });
  }
}
```

### Handle Indexing Errors

```typescript
try {
  await mcpClient.callTool('index', {});
} catch (error) {
  // Don't block on indexing errors
  console.warn('Indexing failed, continuing...');
  // Proceed with potentially stale data
}
```

## Performance Optimization

### Use Filters

```typescript
// Bad: Get everything
const allNodes = await mcpClient.search("");

// Good: Filter by kind
const functions = await mcpClient.search("auth", { kind: "function" });

// Better: Filter by kind and limit
const functions = await mcpClient.search("auth", { 
  kind: "function",
  limit: 20 
});
```

### Limit Depth

```typescript
// Bad: Unlimited depth
const allCallers = await mcpClient.getCallers(nodeId, 10);

// Good: Limited depth
const directCallers = await mcpClient.getCallers(nodeId, 1);

// Better: Limited depth with limit
const callers = await mcpClient.getCallers(nodeId, 2);
// Process only first 20
const topCallers = callers.slice(0, 20);
```

### Cache Results

```typescript
// Cache frequently accessed nodes
const cache = new Map();

async function getCachedNode(nodeId) {
  if (cache.has(nodeId)) {
    return cache.get(nodeId);
  }
  const node = await mcpClient.getNode(nodeId);
  cache.set(nodeId, node);
  return node;
}
```

### Batch Operations

```typescript
// Bad: Sequential calls
for (const id of nodeIds) {
  const node = await mcpClient.getNode(id);
}

// Good: Parallel calls
const nodes = await Promise.all(
  nodeIds.map(id => mcpClient.getNode(id))
);
```

## Best Practices

### 1. Always Check Index Status

Before using spy-code, check if indexing is complete:

```typescript
const stats = await mcpClient.getStats();
if (stats.nodeCount === 0) {
  // Need to index
  await mcpClient.callTool('index', {});
}
```

### 2. Use Semantic Search When Appropriate

For conceptual questions, use `ask` instead of `search`:

```typescript
// Good for conceptual questions
const results = await mcpClient.ask("how do I authenticate users?");

// Good for exact name searches
const results = await mcpClient.search("authenticate");
```

### 3. Verify Node IDs

Before using a node ID, verify it exists:

```typescript
const node = await mcpClient.getNode(nodeId);
if (!node) {
  // Handle missing node
  return;
}
```

### 4. Respect Confidence Scores

Low confidence edges (< 1.0) indicate uncertainty:

```typescript
if (edge.confidence < 1.0) {
  // Warn user
  // Suggest manual verification
}
```

### 5. Re-index After Changes

After making code changes, re-index:

```typescript
await mcpClient.callTool('index', {});
```

### 6. Use Appropriate Depth

Start with depth 1, increase only if needed:

```typescript
// Start with direct relationships
const callers = await mcpClient.getCallers(nodeId, 1);

// Increase only if needed
if (callers.length === 0) {
  const callers = await mcpClient.getCallers(nodeId, 2);
}
```

### 7. Provide Context in Responses

Always include file paths and line numbers:

```typescript
const node = await mcpClient.getNode(nodeId);
console.log(`Function: ${node.name}`);
console.log(`Location: ${node.filePath}:${node.startLine}`);
```

### 8. Handle Errors Gracefully

Never let spy-code errors block the agent:

```typescript
try {
  const result = await mcpClient.callTool('search', { query });
} catch (error) {
  // Fall back to other methods
  // Don't block the agent
}
```

## Integration with Agent Capabilities

### Combine with File Reading

```typescript
// Get node from spy-code
const node = await mcpClient.getNode(nodeId);

// Read the actual file
const fileContent = await readFile(node.filePath);

// Provide context-aware response
```

### Combine with Git Operations

```typescript
// Get changed nodes
const changed = await mcpClient.changedSince('HEAD~5');

// Get git diff for each
for (const node of changed) {
  const diff = await gitDiff(node.filePath);
  // Analyze changes
}
```

### Combine with Testing

```typescript
// Find functions
const functions = await mcpClient.search("validate");

// Find their tests
const tests = await findTests(functions);

// Suggest missing tests
const untested = functions.filter(f => !hasTest(f));
```

## Common Pitfalls

### 1. Assuming Index is Current

Always check `stats.lastIndexed` before relying on data.

### 2. Ignoring Confidence Scores

Low confidence edges may be incorrect. Always warn users.

### 3. Using Too Much Depth

High depth values can return huge results. Start low.

### 4. Not Caching Results

Repeated queries for the same data are wasteful. Cache results.

### 5. Blocking on Indexing Errors

Indexing errors shouldn't block the agent. Continue with stale data if needed.

### 6. Not Verifying Node IDs

Node IDs may be invalid or outdated. Always verify before use.

### 7. Using Wrong Tool for the Job

Use `search` for keywords, `ask` for concepts, `get_node` for specifics.

### 8. Not Providing Context

Responses should always include file paths and line numbers.

## Next Steps

- [Integrations Guide](./INTEGRATIONS.md) - Environment-specific integration
- [MCP Setup Guide](./MCP_SETUP.md) - MCP server configuration
- [Skill Reference](./SKILL_REFERENCE.md) - Complete skill catalog
- [Skills Directory](../skills/) - Detailed skill documentation
