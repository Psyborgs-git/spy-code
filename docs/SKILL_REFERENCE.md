# Skill Reference

Complete catalog of spy-code skills for AI coding agents.

## Skill Overview

Skills are reusable patterns for using spy-code effectively. Each skill includes:
- When to use it
- Available tools
- Example queries
- Best practices
- Common patterns

## Universal Skills

### Code Navigation

**File:** [skills/universal/code-navigation.md](../skills/universal/code-navigation.md)

**When to Use:**
- Finding functions, classes, or constants
- Navigating to specific code locations
- Understanding codebase structure
- Locating implementation details

**Available Tools:**
- `search` - Fuzzy name/description search
- `get_node` - Fetch node by ID
- `query_graph` - Raw GraphQL queries

**Example Queries:**
```bash
spy-code search "auth" --kind function
spy-code get src:auth.rs:_:login
spy-code query '{ node(id: "src:auth.rs:_:login") { name description } }'
```

**Best Practices:**
1. Start with search when you don't know the exact node ID
2. Use kind filters to narrow results
3. Check node IDs follow the format `dir:file:class:symbol`
4. Review signatures before using functions
5. Verify file paths are correct

**Common Patterns:**
- Find and Inspect: Search → Get details → Read file
- Explore by File: List files → Search within files
- Find Related Code: Find class → Get methods → Look at signatures

---

### Call Graph Analysis

**File:** [skills/universal/call-graph-analysis.md](../skills/universal/call-graph-analysis.md)

**When to Use:**
- Understanding how functions call each other
- Finding all callers of a specific function
- Finding all functions called by a specific function
- Analyzing code dependencies
- Understanding impact of changes
- Tracing execution flow

**Available Tools:**
- `find_callers` - List functions that call a node
- `find_callees` - List functions called by a node
- `query_graph` - Complex graph queries

**Example Queries:**
```bash
spy-code callers src:auth.rs:_:login
spy-code callers src:auth.rs:_:login --depth 3
spy-code callees src:auth.rs:_:login
```

**Best Practices:**
1. Start with depth 1, increase if needed
2. Check confidence scores (< 1.0 indicates ambiguity)
3. Use for impact analysis before changes
4. Find entry points using callees
5. Detect dead code (functions with no callers)

**Common Patterns:**
- Impact Analysis: Find callers → Review → Make changes → Re-index
- Trace Execution: Start from entry → Follow callees → Understand flow
- Find Dead Code: Search functions → Check callers → Remove unused
- Understand Dependencies: Find what a module depends on → Analyze → Plan refactoring

---

### Semantic Search

**File:** [skills/universal/semantic-search.md](../skills/universal/semantic-search.md)

**When to Use:**
- Finding code by meaning rather than exact names
- Asking natural language questions
- Discovering related functionality
- Finding code that implements a concept
- Searching for "how to" patterns

**Available Tools:**
- `ask` - Natural language questions
- `semanticSearchEmbeddings` - Vector-based search
- `embeddingsStatus` - Check embedding status

**Example Queries:**
```bash
spy-code ask "how do I authenticate users?"
spy-code ask "where is the database connection configured?"
spy-code query '{ semanticSearchEmbeddings(query: "authentication", limit: 10) { node { name } score } }'
```

**Prerequisites:**
- Generate embeddings first: `spy-code embed`
- Check status: `spy-code query '{ embeddingsStatus { ... } }'`

**Best Practices:**
1. Generate embeddings before using semantic search
2. Use specific questions for better results
3. Check scores (higher = better match)
4. Combine with keyword search for best results
5. Re-embed after significant code changes

**Common Patterns:**
- Understand "How To": Ask question → Review results → Follow pattern
- Find Related Code: Ask about concept → Explore results → Learn architecture
- Discovery: Ask about functionality → Learn about codebase → Find utilities
- Code Review: Find similar implementations → Compare → Suggest improvements

---

### Change Tracking

**File:** [skills/universal/change-tracking.md](../skills/universal/change-tracking.md)

**When to Use:**
- Finding code that changed since a git commit
- Understanding changes between branches
- Identifying affected code after rebase
- Tracking code evolution
- Finding nodes needing re-reading after updates

**Available Tools:**
- `changed_since` - List nodes changed since git ref
- `query_graph` - Raw GraphQL queries

**Example Queries:**
```bash
spy-code changed HEAD~1
spy-code changed abc123def456
spy-code changed origin/main
```

**Best Practices:**
1. Use after rebase/merge to find changes
2. Combine with call graph for impact analysis
3. Re-index when needed after changes
4. Use meaningful refs (branches, commits)
5. Track gitSha to verify indexing

**Common Patterns:**
- Post-Rebase Analysis: Find changes → Review → Re-index → Verify
- Pull Request Impact: Find changes → Analyze → Check callers → Review
- Branch Comparison: Compare → Understand changes → Identify conflicts → Plan
- Regression Testing: Find changes → Focus testing → Check callers → Prioritize

---

### Graph Visualization

**File:** [skills/universal/graph-visualization.md](../skills/universal/graph-visualization.md)

**When to Use:**
- Visualizing code relationships
- Exploring codebase structure interactively
- Understanding module dependencies
- Identifying architectural patterns
- Presenting architecture to others

**Available Tools:**
- `graphData` - Get filtered graph data
- `query_graph` - Raw GraphQL queries

**Example Queries:**
```graphql
{
  graphData {
    nodes { id name kind filePath }
    edges { from { id } to { id } kind }
  }
}

{
  graphData(filter: { filePath: "src/auth.rs" }) {
    nodes { name kind }
    edges { kind }
  }
}
```

**Starting the Graph UI:**
```bash
spy-code graph --open
spy-code serve --http --port 4000
# Visit http://localhost:4000/graph
```

**Best Practices:**
1. Start filtered to reduce graph size
2. Focus on specific areas (file, module)
3. Use edge filters to show relevant relationships
4. Check confidence for edge reliability
5. Export for documentation

**Common Patterns:**
- Explore Module: Filter by file → Visualize → Understand structure
- Analyze Dependencies: Filter by imports → Visualize → Identify coupling
- Understand Call Flow: Filter by calls → Visualize → Trace execution
- Cross-Language: Filter by languages → Visualize → Understand polyglot code

---

## Environment-Specific Skills

### Cursor Skills

**File:** [skills/environments/cursor-skills.md](../skills/environments/cursor-skills.md)

**Cursor-Specific Patterns:**
- Code Understanding in Chat
- Context-Aware Code Generation
- Refactoring Assistance
- Impact Analysis

**Configuration:** `.cursor/mcp_config.json`

**Key Features:**
- Code-aware chat responses
- Context-aware code completion
- Refactoring assistance
- Impact analysis

**Cursor-Specific Tips:**
1. Always provide context with file paths and line numbers
2. Show call graphs when explaining code
3. Use semantic search for conceptual questions
4. Warn about low-confidence edges
5. Suggest related code

---

### Windsurf/Cascade Skills

**File:** [skills/environments/windsurf-skills.md](../skills/environments/windsurf-skills.md)

**Windsurf-Specific Patterns:**
- Pre-Read Code Enrichment
- Post-Write Code Validation
- Smart Caching
- Event-Driven Updates

**Configuration:** `.windsurf/mcp_config.json`

**Integration Core:**
```typescript
import { 
  MCPClient, 
  AgentHooks, 
  AutoIndexHook,
  ContextEnrichmentHook 
} from '@spy-code/integration-core';
```

**Key Features:**
- Agent hooks for automation
- Context enrichment
- Smart caching
- Error recovery
- Skill engine

**Windsurf-Specific Tips:**
1. Use hooks for automation
2. Leverage caching
3. Enrich context with graph information
4. Validate changes before/after
5. Handle errors gracefully

---

### GitHub Copilot Skills

**File:** [skills/environments/copilot-skills.md](../skills/environments/copilot-skills.md)

**Copilot-Specific Patterns:**
- Context-Aware Code Completion
- Chat-Based Code Exploration
- Refactoring Suggestions
- Test Generation

**Configuration:** Copilot's MCP config (location varies)

**Key Features:**
- Context-aware completions
- Chat-based exploration
- Refactoring suggestions
- Test generation

**Copilot-Specific Tips:**
1. Combine with Copilot's training for project-specific context
2. Provide file references in responses
3. Show code snippets
4. Suggest navigation
5. Maintain Copilot's tone

---

### Claude Skills

**File:** [skills/environments/claude-skills.md](../skills/environments/claude-skills.md)

**Claude-Specific Patterns:**
- Deep Code Understanding
- Context-Aware Code Generation
- Comprehensive Refactoring
- Architectural Analysis

**Configuration:** `~/Library/Application Support/Claude/claude_desktop_config.json` (macOS) or `~/.config/Claude/claude_desktop_config.json` (Linux)

**Key Features:**
- Deep code understanding
- Comprehensive explanations
- Multi-step reasoning
- Natural language queries

**Claude-Specific Tips:**
1. Be thorough in analysis
2. Provide comprehensive context
3. Use examples for suggestions
4. Explain trade-offs
5. Think step by step

---

## Skill Composition

### Combining Skills

Skills can be combined for complex tasks:

**Example: Understanding a New Codebase**
1. **Code Navigation** - Find entry points
2. **Call Graph Analysis** - Trace execution flow
3. **Graph Visualization** - Understand architecture
4. **Semantic Search** - Discover functionality

**Example: Implementing a Feature**
1. **Code Navigation** - Find similar features
2. **Call Graph Analysis** - Understand patterns
3. **Change Tracking** - Plan integration
4. **Semantic Search** - Find related code

**Example: Debugging**
1. **Code Navigation** - Find relevant functions
2. **Call Graph Analysis** - Trace execution
3. **Semantic Search** - Find error handling patterns
4. **Change Tracking** - Check recent changes

### Skill Selection Guide

| Task | Primary Skill | Secondary Skills |
|------|---------------|------------------|
| Find code | Code Navigation | Semantic Search |
| Understand flow | Call Graph Analysis | Graph Visualization |
| Conceptual questions | Semantic Search | Code Navigation |
| After rebase | Change Tracking | Call Graph Analysis |
| Architecture review | Graph Visualization | Call Graph Analysis |
| Refactoring | Call Graph Analysis | Change Tracking |
| Debugging | Code Navigation | Call Graph Analysis |
| Code review | Change Tracking | Call Graph Analysis |

## Skill Engine Integration

The `@spy-code/integration-core` package provides a skill engine for programmatic skill usage:

```typescript
import { 
  SkillRegistry, 
  getSkillRegistry,
  MCPClient 
} from '@spy-code/integration-core';

// Initialize
const mcpClient = new MCPClient();
await mcpClient.connect();

const skillRegistry = getSkillRegistry();
await skillRegistry.initialize(mcpClient);

// Match skills
const matches = skillRegistry.match("how do I authenticate users?");
const bestMatch = matches[0];

// Execute skill
const result = await skillRegistry.execute(bestMatch.skill.id, {
  request: "how do I authenticate users?"
});
```

## Skill Metadata

Each skill includes:

- **ID**: Unique identifier (filename without .md)
- **Name**: Human-readable name
- **Description**: What the skill does
- **When to Use**: List of use cases
- **Available Tools**: MCP tools used
- **Example Queries**: Example commands/queries
- **Best Practices**: Guidelines for effective use
- **Common Patterns**: Reusable workflows

## Skill Development

### Creating a New Skill

1. Create a new markdown file in `skills/universal/` or `skills/environments/`
2. Follow the skill template:
   ```markdown
   # Skill Name
   
   ## When to Use
   - Use case 1
   - Use case 2
   
   ## Available Tools
   - tool1 - description
   - tool2 - description
   
   ## Example Queries
   ```bash
   example command
   ```
   
   ## Best Practices
   1. Practice 1
   2. Practice 2
   
   ## Common Patterns
   ### Pattern Name
   Description
   Steps
   ```

3. Test the skill with various queries
4. Document any environment-specific variations

### Skill Validation

Before adding a skill:
- Ensure it solves a real problem
- Verify tool availability
- Test with multiple codebases
- Check for overlap with existing skills
- Document edge cases

## Skill Maintenance

### Updating Skills

- Update examples as spy-code evolves
- Add new tools as they're added
- Refine best practices based on usage
- Update patterns for new workflows

### Deprecating Skills

- Mark as deprecated in documentation
- Provide migration path to new skills
- Keep for backward compatibility if needed
- Remove after deprecation period

## Skill Performance

### Performance Tips

1. **Use filters** - Reduce query scope
2. **Limit depth** - Start with depth 1
3. **Cache results** - Avoid repeated queries
4. **Batch operations** - Parallelize when possible
5. **Use appropriate tools** - Right tool for the job

### Monitoring

Monitor skill usage to:
- Identify popular skills
- Find performance bottlenecks
- Discover missing skills
- Improve documentation

## Skill Examples

### Example 1: Finding Authentication Code

**Using Code Navigation Skill:**
```bash
# Search for auth functions
spy-code search "auth" --kind function

# Get specific function
spy-code get src:auth.rs:_:login

# Get callers to understand usage
spy-code callers src:auth.rs:_:login --depth 2
```

**Using Semantic Search Skill:**
```bash
# Ask natural language question
spy-code ask "how do I authenticate users?"
```

### Example 2: Understanding Impact of Changes

**Using Change Tracking Skill:**
```bash
# Find changed code
spy-code changed origin/main

# For each changed function, check impact
spy-code callers src:api:handlers:_:updated_function --depth 2
```

**Using Call Graph Analysis Skill:**
```bash
# Analyze dependencies
spy-code callees src:models:User --depth 2
```

### Example 3: Visualizing Module Structure

**Using Graph Visualization Skill:**
```graphql
{
  graphData(filter: {
    filePath: "src/auth/"
    nodeKinds: [FUNCTION]
    edgeKinds: [CALLS]
  }) {
    nodes { name }
    edges { from { name } to { name } }
  }
}
```

Or use the UI:
```bash
spy-code graph --open
```

## Next Steps

- [Integrations Guide](./INTEGRATIONS.md) - Environment-specific setup
- [MCP Setup Guide](./MCP_SETUP.md) - MCP server configuration
- [Agent Usage Guide](./AGENT_USAGE.md) - How agents should use spy-code
- [Skills Directory](../skills/) - Detailed skill documentation
