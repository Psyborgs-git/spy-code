# Spy-Code IDE Integrations

This document provides comprehensive documentation for integrating spy-code with various IDEs and coding agents.

## Overview

Spy-Code provides native integrations for the following IDEs and coding agents:

- **VS Code** (and VS Code-based IDEs: Cursor, Windsurf, Antigravity)
- **Claude Code**
- **GitHub Copilot**
- **OpenAI Codex**

All integrations use a shared core library (`@spy-code/integration-core`) and follow a port:adapter architecture pattern for maximum code reuse.

## Architecture

### Port:Adapter Pattern

The integration layer uses a port:adapter design pattern:

- **Core Port** (`@spy-code/integration-core`): Shared library containing common types, CLI bridge, MCP adapter, cache manager, event bus, and agent hooks
- **Adapters**: IDE-specific implementations that adapt to each IDE's API
- **Extensions**: Built extensions that use the adapters and communicate with spy-code CLI

### Components

```
spy-integration-core/          # Shared core library
├── types/                     # Common type definitions
├── cli-bridge/                # CLI communication layer
├── mcp-adapter/              # MCP protocol implementation
├── agent-hooks/              # Coding agent hooks interface
├── cache-manager/            # Result caching and state management
└── event-bus/                # Pub/sub system for IDE events

ide-adapters/                  # IDE-specific adapters
├── vscode-adapter/           # VS Code/Cursor/Windsurf/Antigravity
├── claude-code-adapter/      # Claude Code specific
├── copilot-adapter/          # GitHub Copilot specific
└── codex-adapter/            # OpenAI Codex specific

extensions/                    # Built extensions
├── vscode-extension/         # .vsix for VS Code ecosystem
├── claude-code-extension/    # Claude Code extension
├── cursor-extension/         # Cursor extension
├── windsurf-extension/       # Windsurf extension
├── antigravity-extension/    # Antigravity extension
├── copilot-extension/        # Copilot extension
└── codex-integration/        # Codex SDK integration
```

## Installation

### Prerequisites

1. Install spy-code CLI:
```bash
cargo install spy-code
```

2. Initialize spy-code in your project:
```bash
cd your-project
spy-code init
spy-code index
```

### VS Code Extension

1. Build the extension:
```bash
cd extensions/vscode-extension
npm install
npm run compile
```

2. Package as .vsix:
```bash
npm install -g vsce
vsce package
```

3. Install in VS Code:
```bash
code --install-extension spy-code-0.1.0.vsix
```

### Cursor Extension

Cursor is VS Code-based, so the installation is identical to VS Code:

```bash
cd extensions/cursor-extension
npm install
npm run compile
vsce package
cursor --install-extension spy-code-cursor-0.1.0.vsix
```

### Windsurf Extension

Windsurf is VS Code-based, so the installation is identical to VS Code:

```bash
cd extensions/windsurf-extension
npm install
npm run compile
vsce package
windsurf --install-extension spy-code-windsurf-0.1.0.vsix
```

### Antigravity Extension

Antigravity is VS Code-based, so the installation is identical to VS Code:

```bash
cd extensions/antigravity-extension
npm install
npm run compile
vsce package
antigravity --install-extension spy-code-antigravity-0.1.0.vsix
```

### Claude Code Extension

1. Build the extension:
```bash
cd extensions/claude-code-extension
npm install
npm run compile
```

2. The extension automatically configures MCP in `.claude/mcp.json`

3. Restart Claude Code to activate the integration

### GitHub Copilot Extension

1. Build the extension:
```bash
cd extensions/copilot-extension
npm install
npm run compile
```

2. The extension automatically configures MCP in `.github/copilot/mcp.json`

3. Restart GitHub Copilot to activate the integration

### OpenAI Codex Integration

1. Build the integration:
```bash
cd extensions/codex-integration
npm install
npm run compile
```

2. The integration automatically configures MCP in `.codex/mcp.json` and registers agent templates

3. Restart Codex to activate the integration

## Configuration

All extensions share common configuration options:

```json
{
  "spy-code.path": "spy-code",
  "spy-code.dbPath": ".spy-code/graph.db",
  "spy-code.enableMCP": true,
  "spy-code.enableHooks": true,
  "spy-code.cacheEnabled": true,
  "spy-code.cacheTTL": 300000
}
```

### Configuration Options

- `spy-code.path`: Path to spy-code CLI binary (default: "spy-code")
- `spy-code.dbPath`: Path to spy-code database (default: ".spy-code/graph.db")
- `spy-code.enableMCP`: Enable MCP server integration (default: true)
- `spy-code.enableHooks`: Enable coding agent hooks (default: true)
- `spy-code.cacheEnabled`: Enable result caching (default: true)
- `spy-code.cacheTTL`: Cache time-to-live in milliseconds (default: 300000)

## Features

All integrations provide the following features:

### Code Navigation

- **Go to Definition**: Navigate to function/class definitions using spy-code's graph
- **Find References**: Find all references to a function/class
- **Call Graph**: Visualize call relationships between functions

### Code Search

- **Keyword Search**: Search for functions, classes, and constants by name
- **Semantic Search**: Natural language queries using embeddings
- **Filtered Search**: Filter by node kind (function, class, constant)

### Code Analysis

- **Impact Analysis**: Identify downstream effects of code changes
- **Dependency Tracking**: Understand import and call relationships
- **Statistics**: View index statistics (node count, edge count, file count)

### Coding Agent Hooks

The agent hooks system allows external agents to interact with spy-code at key points:

- `pre_read_code`: Before agent reads a file
- `post_read_code`: After agent reads a file
- `pre_write_code`: Before agent writes a file
- `post_write_code`: After agent writes a file
- `pre_run_command`: Before agent executes a command
- `post_run_command`: After agent executes a command
- `pre_mcp_tool_use`: Before MCP tool invocation
- `post_mcp_tool_use`: After MCP tool invocation
- `pre_user_prompt`: Before processing user prompt
- `post_cascade_response`: After agent response
- `post_setup_worktree`: After setting up a git worktree

## Usage

### VS Code / Cursor / Windsurf / Antigravity

#### Commands

- `Ctrl+Shift+S` (Cmd+Shift+S on Mac): Search codebase
- `Ctrl+Shift+Alt+S` (Cmd+Shift+Alt+S on Mac): Semantic search
- Command Palette: "Spy-Code: Show Panel"
- Command Palette: "Spy-Code: Reindex Codebase"
- Command Palette: "Spy-Code: Show Index Statistics"

#### Sidebar

The Spy-Code sidebar provides:
- Search input for keyword and semantic search
- Results display with file locations
- Quick navigation to results
- Index statistics display

#### Language Features

- **Completion**: Code completion based on spy-code index
- **Definition**: Go to definition using spy-code
- **References**: Find references using spy-code
- **Hover**: Hover information from spy-code
- **Code Lens**: Show callers/callees for functions

### Claude Code

#### Skills

Claude Code skills are automatically registered:
- `spy-code-search`: Search the codebase
- `spy-code-semantic-search`: Semantic search using embeddings
- `spy-code-get-node`: Get detailed information about a node
- `spy-code-callers`: Find callers of a function
- `spy-code-callees`: Find callees of a function

#### Workflows

Claude Code workflows are automatically registered:
- `analyze-function`: Analyze a function and its relationships
- `impact-analysis`: Analyze the impact of changing a function
- `code-review`: Review code using spy-code context

### GitHub Copilot

#### MCP Tools

Copilot MCP tools are automatically registered:
- `spy-code-search`: Search the codebase
- `spy-code-semantic-search`: Semantic search using embeddings
- `spy-code-get-node`: Get node details
- `spy-code-callers`: Find callers of a function
- `spy-code-callees`: Find callees of a function

### OpenAI Codex

#### Agent Templates

Codex agent templates are automatically registered:
- `spy-code-search-agent`: Agent for searching codebase
- `spy-code-analysis-agent`: Agent for code analysis
- `spy-code-review-agent`: Agent for code review

## Development

### Building the Core Library

```bash
cd spy-integration-core
npm install
npm run build
```

### Building Adapters

```bash
cd ide-adapters/vscode-adapter
npm install
npm run build
```

### Building Extensions

```bash
cd extensions/vscode-extension
npm install
npm run compile
```

### Testing

```bash
cd spy-integration-core
npm test
```

## Troubleshooting

### CLI Not Found

If you see "Spy-Code CLI not found", ensure:
1. spy-code is installed: `cargo install spy-code`
2. The path is correct in settings
3. The binary is in your PATH

### MCP Connection Failed

If MCP connection fails:
1. Check that spy-code is running: `spy-code serve --mcp`
2. Verify MCP configuration in the appropriate config file
3. Check firewall settings

### Index Out of Date

If results seem outdated:
1. Run reindex: Command Palette → "Spy-Code: Reindex Codebase"
2. Or use CLI: `spy-code index`

### Performance Issues

If performance is slow:
1. Enable caching in settings
2. Adjust cache TTL
3. Consider using MCP instead of CLI for faster queries

## Contributing

When adding support for a new IDE:

1. Create a new adapter in `ide-adapters/`
2. Implement the `IDEAdapter` interface
3. Create a new extension in `extensions/`
4. Follow the existing patterns for maximum code reuse
5. Add documentation to this file

## License

MIT License - see LICENSE file for details
