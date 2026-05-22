# Spy-Code IDE Integrations

This directory contains the complete implementation of spy-code integrations for multiple IDEs and coding agents.

## Quick Start

1. **Install spy-code CLI**:
```bash
cargo install spy-code
```

2. **Initialize in your project**:
```bash
cd your-project
spy-code init
spy-code index
```

3. **Install the extension for your IDE**:
   - **VS Code**: See `extensions/vscode-extension/`
   - **Cursor**: See `extensions/cursor-extension/`
   - **Windsurf**: See `extensions/windsurf-extension/`
   - **Antigravity**: See `extensions/antigravity-extension/`
   - **Claude Code**: See `extensions/claude-code-extension/`
   - **GitHub Copilot**: See `extensions/copilot-extension/`
   - **OpenAI Codex**: See `extensions/codex-integration/`

## Architecture

```
spy-integration-core/          # Shared core library (TypeScript)
├── src/
│   ├── types/                 # Common types and interfaces
│   ├── cli-bridge/            # CLI communication layer
│   ├── mcp-adapter/          # MCP protocol implementation
│   ├── agent-hooks/          # Coding agent hooks interface
│   ├── cache-manager/        # Result caching and state management
│   └── event-bus/            # Pub/sub system for IDE events
├── package.json
└── tsconfig.json

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

## Building

### Build Core Library
```bash
cd spy-integration-core
npm install
npm run build
```

### Build Adapters
```bash
cd ide-adapters/vscode-adapter
npm install
npm run build
```

### Build Extensions
```bash
cd extensions/vscode-extension
npm install
npm run compile
```

## Features

All integrations provide:

- **Code Navigation**: Go to definition, find references
- **Code Search**: Keyword search and semantic search
- **Call Graph Visualization**: Display function call relationships
- **Impact Analysis**: Identify downstream effects of changes
- **Coding Agent Hooks**: Lifecycle hooks for AI agents
- **MCP Integration**: Model Context Protocol support

## Documentation

See `docs/IDE_INTEGRATIONS.md` for comprehensive documentation.

## Supported IDEs

- ✅ VS Code
- ✅ Cursor (VS Code-based)
- ✅ Windsurf (VS Code-based)
- ✅ Antigravity (VS Code-based)
- ✅ Claude Code
- ✅ GitHub Copilot
- ✅ OpenAI Codex

## Configuration

All extensions share common configuration:

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

## License

MIT
