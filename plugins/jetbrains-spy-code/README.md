# Spy-Code JetBrains Plugin

Codebase intelligence for JetBrains IDEs - index, query, and analyze your code with graph-based navigation.

## Features

- **Search Codebase**: Search for functions, classes, and constants
- **Find Callers**: See what functions call a selected symbol
- **Find Callees**: See what functions a selected symbol calls
- **Tool Window**: Dedicated Spy-Code tool window for search and navigation
- **Codebase Statistics**: View codebase statistics
- **Context Menu Actions**: Quick access from editor context menu

## Installation

1. Install spy-code globally:
   ```bash
   npm install -g spy-code
   # or
   pip install spy-code
   ```

2. Initialize spy-code in your project:
   ```bash
   cd your-project
   spy-code init
   spy-code index
   ```

3. Install the plugin from JetBrains Marketplace (when published)

## Usage

### Search Codebase

1. Open Spy-Code tool window (View → Tool Windows → Spy-Code)
2. Enter search query
3. View results in the list
4. Click a result to see details

### Find Callers/Callees

1. Right-click on a symbol in the editor
2. Select "Find Callers" or "Find Callees"
3. View the relationships

### Index Codebase

1. Go to Tools → Index Codebase
2. Wait for indexing to complete
3. Search and navigate your code

## Keyboard Shortcuts

- **Ctrl+Shift+S** (Windows/Linux) / **Cmd+Shift+S** (Mac): Search Codebase

## Supported IDEs

- IntelliJ IDEA
- PyCharm
- WebStorm
- PhpStorm
- GoLand
- CLion
- Rider

## Requirements

- spy-code CLI installed
- spy-code initialized in your workspace
- JetBrains IDE 2023.2 or later

## Development

```bash
# Build plugin
./gradlew buildPlugin

# Run plugin in IDE
./gradlew runIde

# Publish plugin
./gradlew publishPlugin
```

## License

MIT
