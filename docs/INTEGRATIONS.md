# Spy-Code Integrations Guide

This guide covers all available integrations for spy-code with AI coding environments and IDEs.

## Overview

Spy-Code integrates with AI coding environments through the Model Context Protocol (MCP), enabling AI agents to understand codebase structure, query code relationships, and provide more contextually relevant assistance.

## Supported Environments

| Environment | Integration Type | Status |
|-------------|------------------|--------|
| Cursor | MCP Server | ✅ Supported |
| Windsurf/Cascade | MCP Server + Integration Core | ✅ Supported |
| Claude Desktop | MCP Server | ✅ Supported |
| GitHub Copilot | MCP Server | ✅ Supported |
| VS Code | Extension | 🚧 Planned |
| JetBrains IDEs | Plugin | 🚧 Planned |
| Neovim | Plugin | 🚧 Planned |

## Quick Start

### Option 1: Auto-Installer (Recommended)

Run the auto-installer script to automatically detect your environment and configure spy-code:

```bash
# From the spy-code repository
./scripts/install-spy-code-skill.sh
```

The script will:
1. Check if spy-code is installed
2. Detect your AI coding environment
3. Initialize spy-code configuration
4. Index your codebase
5. Configure MCP integration
6. Verify the setup

### Option 2: Manual Setup

#### Step 1: Install spy-code

```bash
# Via npm
npm install -g spy-code

# Via pip
pip install spy-code
```

#### Step 2: Initialize Configuration

```bash
cd your-project
spy-code init
```

#### Step 3: Index Your Codebase

```bash
spy-code index
```

#### Step 4: Configure MCP

Choose your environment and copy the appropriate config:

**Cursor** - Add to `.cursor/mcp_config.json`:
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

**Windsurf/Cascade** - Add to `.windsurf/mcp_config.json`:
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

**Claude Desktop** - Add to `claude_desktop_config.json`:
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

**GitHub Copilot** - Add to Copilot's MCP config:
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

#### Step 5: Restart Your Environment

Restart your AI coding environment to load the MCP configuration.

#### Step 6: Test the Integration

In your AI coding environment, try:
```
"Search for functions related to authentication"
"Show me the call graph for the login function"
"How do I handle errors in this codebase?"
```

## Environment-Specific Guides

### Cursor

Cursor integrates with spy-code through MCP. See [skills/environments/cursor-skills.md](../skills/environments/cursor-skills.md) for Cursor-specific patterns and workflows.

**Key Features:**
- Code-aware chat responses
- Context-aware code completion
- Refactoring assistance
- Impact analysis

**Configuration:** `.cursor/mcp_config.json`

### Windsurf/Cascade

Windsurf and Cascade have enhanced integration through the `@spy-code/integration-core` library, providing:
- Agent hooks for automatic indexing
- Context enrichment
- Smart caching
- Error recovery

**Installation:**
```bash
npm install @spy-code/integration-core
```

**Usage:**
```typescript
import { 
  MCPClient, 
  AgentHooks, 
  AutoIndexHook,
  ContextEnrichmentHook 
} from '@spy-code/integration-core';

const mcpClient = new MCPClient();
await mcpClient.connect();

const hooks = getAgentHooks();
const autoIndex = new AutoIndexHook(mcpClient);
autoIndex.register(hooks);
```

See [skills/environments/windsurf-skills.md](../skills/environments/windsurf-skills.md) for detailed usage.

### Claude Desktop

Claude Desktop uses MCP for integration. See [skills/environments/claude-skills.md](../skills/environments/claude-skills.md) for Claude-specific patterns.

**Key Features:**
- Deep code understanding
- Comprehensive explanations
- Multi-step reasoning
- Natural language queries

**Configuration:** `~/Library/Application Support/Claude/claude_desktop_config.json` (macOS) or `~/.config/Claude/claude_desktop_config.json` (Linux)

### GitHub Copilot

Copilot integrates through MCP. See [skills/environments/copilot-skills.md](../skills/environments/copilot-skills.md) for Copilot-specific patterns.

**Key Features:**
- Context-aware completions
- Chat-based code exploration
- Refactoring suggestions
- Test generation

## MCP Server Tools

The spy-code MCP server exposes the following tools:

| Tool | Description | Use When |
|------|-------------|----------|
| `query_graph` | Run raw GraphQL queries | Complex multi-hop queries |
| `get_node` | Fetch a node by ID | You know the exact node ID |
| `search` | Fuzzy name/description search | You know roughly what to find |
| `find_callers` | Find functions that call a node | Understanding impact of changes |
| `find_callees` | Find functions called by a node | Tracing execution flow |
| `changed_since` | Find nodes changed since git ref | After rebase/merge |
| `stats` | Get index statistics | Checking index status |
| `ask` | Natural language queries | Conceptual questions |
| `embed` | Generate embeddings | For semantic search |

## MCP Resources

The spy-code MCP server exposes the following resources:

| Resource | URI | Description |
|----------|-----|-------------|
| GraphQL Schema | `spy-code://schema` | Full GraphQL SDL |
| Index Stats | `spy-code://stats` | Current index statistics |
| Config | `spy-code://config` | Loaded configuration |

## Agent Skills

Spy-Code includes comprehensive agent skills for different use cases:

### Universal Skills

Located in [skills/universal/](../skills/universal/):

- **code-navigation.md** - Finding and navigating code
- **call-graph-analysis.md** - Analyzing call relationships
- **semantic-search.md** - Natural language queries
- **change-tracking.md** - Tracking code changes
- **graph-visualization.md** - Visualizing code structure

### Environment-Specific Skills

Located in [skills/environments/](../skills/environments/):

- **cursor-skills.md** - Cursor-specific patterns
- **windsurf-skills.md** - Windsurf/Cascade-specific patterns
- **copilot-skills.md** - GitHub Copilot-specific patterns
- **claude-skills.md** - Claude-specific patterns

## Integration Core Library

The `@spy-code/integration-core` npm package provides:

- **MCPClient** - TypeScript MCP client implementation
- **AgentHooks** - Lifecycle hooks for agent actions
- **Hook Implementations** - Pre-built hooks for common patterns
- **CacheManager** - Intelligent caching
- **EventBus** - Event-driven architecture
- **Skill Engine** - Skill loading, matching, and execution

**Installation:**
```bash
npm install @spy-code/integration-core
```

**Usage:**
```typescript
import { 
  MCPClient, 
  AgentHooks, 
  getAgentHooks,
  AutoIndexHook,
  ContextEnrichmentHook,
  SkillRegistry,
  getSkillRegistry
} from '@spy-code/integration-core';

// Initialize MCP client
const mcpClient = new MCPClient();
await mcpClient.connect();

// Get agent hooks
const hooks = getAgentHooks();

// Register pre-built hooks
const autoIndex = new AutoIndexHook(mcpClient);
autoIndex.register(hooks);

const contextEnrichment = new ContextEnrichmentHook(mcpClient);
contextEnrichment.register(hooks);

// Initialize skill registry
const skillRegistry = getSkillRegistry();
await skillRegistry.initialize(mcpClient);

// Match and execute skills
const matches = skillRegistry.match("how do I authenticate users?");
const bestMatch = matches[0];
const result = await skillRegistry.execute(bestMatch.skill.id, {
  request: "how do I authenticate users?"
});
```

## Troubleshooting

### MCP Server Not Connecting

**Symptoms:** Tools not available in your AI coding environment

**Solutions:**
1. Verify spy-code is installed: `spy-code --version`
2. Check MCP config file exists and is valid JSON
3. Ensure spy-code is in your PATH
4. Restart your AI coding environment
5. Check environment variables in MCP config

### Indexing Issues

**Symptoms:** Search returns no results, stats show 0 nodes

**Solutions:**
1. Run `spy-code index` to index your codebase
2. Check `spy.config.json` has correct language settings
3. Verify file paths in config are correct
4. Check file size limits in config
5. Run `spy-code index --full` to force full re-index

### Performance Issues

**Symptoms:** Slow queries, timeouts

**Solutions:**
1. Use search filters to reduce scope
2. Reduce graph query depth/limits
3. Check if embeddings are needed for semantic search
4. Consider using cache manager
5. Index only necessary directories

### Configuration Issues

**Symptoms:** Config not loading, wrong language detected

**Solutions:**
1. Verify `spy.config.json` is valid JSON
2. Check language settings match your codebase
3. Ensure file paths are relative to repo root
4. Check ignore patterns aren't too broad
5. Run `spy-code init` to create a fresh config

## Feature Comparison

| Feature | Cursor | Windsurf | Claude | Copilot |
|---------|--------|----------|--------|---------|
| MCP Integration | ✅ | ✅ | ✅ | ✅ |
| Integration Core | ❌ | ✅ | ❌ | ❌ |
| Agent Hooks | ❌ | ✅ | ❌ | ❌ |
| Auto-Indexing | ❌ | ✅ | ❌ | ❌ |
| Context Enrichment | ❌ | ✅ | ❌ | ❌ |
| Smart Caching | ❌ | ✅ | ❌ | ❌ |
| Skill Engine | ❌ | ✅ | ❌ | ❌ |

## Advanced Configuration

### Custom Language Roots

Edit `spy.config.json` to customize which directories to index:

```json
{
  "languages": {
    "rust": {
      "enabled": true,
      "roots": ["src/", "crates/", "examples/"],
      "ignore": ["target/", "**/*.generated.rs"]
    }
  }
}
```

### Custom Ignore Patterns

Add patterns to ignore specific files:

```json
{
  "languages": {
    "typescript": {
      "ignore": [
        "node_modules/",
        "dist/",
        "build/",
        "**/*.test.ts",
        "**/*.spec.ts"
      ]
    }
  }
}
```

### Indexing Performance

Adjust indexing performance settings:

```json
{
  "indexing": {
    "max_file_size_kb": 4096,
    "parallelism": 4,
    "fail_fast": false
  }
}
```

## Next Steps

1. **Explore Skills** - Read the skill documentation to learn patterns
2. **Try Examples** - Test the integration with example queries
3. **Customize Config** - Adjust spy.config.json for your project
4. **Use Integration Core** - Leverage hooks and skill engine in Windsurf
5. **Provide Feedback** - Report issues on GitHub

## Additional Resources

- [MCP Setup Guide](./MCP_SETUP.md)
- [Agent Usage Guide](./AGENT_USAGE.md)
- [Skill Reference](./SKILL_REFERENCE.md)
- [GitHub Repository](https://github.com/Psyborgs-git/spy-code)
- [Issue Tracker](https://github.com/Psyborgs-git/spy-code/issues)
