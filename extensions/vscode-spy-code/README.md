# Spy-Code VS Code Extension

Codebase intelligence for VS Code - index, query, and analyze your code with graph-based navigation.

## Features

- **Search Codebase**: Search for functions, classes, and constants
- **Find Callers**: See what functions call a selected function
- **Find Callees**: See what functions a selected function calls
- **Node Details**: View detailed information about code symbols
- **Graph Visualization**: Interactive graph view of code relationships
- **Statistics**: View codebase statistics
- **Auto-Index**: Automatically re-index after file changes

## Installation

1. Install spy-code globally:
   ```bash
   npm install -g spy-code
   ```

2. Initialize spy-code in your project:
   ```bash
   cd your-project
   spy-code init
   spy-code index
   ```

3. Install this extension (when published to marketplace)

## Usage

### Search Codebase

1. Open Command Palette (Cmd/Ctrl + Shift + P)
2. Run "Spy-Code: Search Codebase"
3. Enter your search query
4. Select a result to view details

### Find Callers/Callees

1. Place cursor on a function or class name
2. Run "Spy-Code: Find Callers" or "Spy-Code: Find Callees"
3. View the relationships in a new document

### Graph Visualization

1. Run "Spy-Code: Open Graph Visualization"
2. View the graph in the Spy-Code sidebar
3. Click nodes to navigate

### Index Codebase

1. Run "Spy-Code: Index Codebase"
2. Wait for indexing to complete
3. Search and navigate your code

## Configuration

Add to your VS Code settings:

```json
{
  "spy-code.configPath": "spy.config.json",
  "spy-code.autoIndex": true,
  "spy-code.graph.maxNodes": 100
}
```

## Requirements

- spy-code CLI installed
- spy-code initialized in your workspace
- MCP server running (automatically started by extension)

## Development

```bash
# Install dependencies
npm install

# Compile
npm run compile

# Watch for changes
npm run watch

# Run tests
npm test
```

## License

MIT
