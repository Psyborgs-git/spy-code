# spy-code

> GraphQL-style codebase intelligence CLI — index, query, and analyze any codebase.

[![PyPI version](https://img.shields.io/pypi/v/spy-code.svg)](https://pypi.org/p/spy-code)
[![License: MIT](https://img.shields.io/badge/License-MIT-blue.svg)](LICENSE)

## Installation

```bash
pip install spy-code
```

On first run (or install), the package automatically downloads the appropriate
native binary for your platform from the GitHub Releases page. No Rust compiler
required.

## Usage

```bash
# Initialize a project
spy-code init

# Index your codebase
spy-code index

# Search for symbols
spy-code search "my_function"

# Get call graph callers
spy-code callers <node_id>

# Start the GraphQL HTTP playground
spy-code serve --http

# Start as an MCP server (for LLM clients)
spy-code serve --mcp
```

## Supported Platforms

| Platform | Architecture | Support |
|----------|-------------|---------|
| Linux    | x86_64      | ✅ |
| macOS    | x86_64      | ✅ |
| macOS    | Apple Silicon (arm64) | ✅ |
| Windows  | x86_64      | ✅ |

## How It Works

This package is a thin Python wrapper around the native `spy-code` binary built
with Rust. On installation, `spy-code` detects your OS and CPU architecture,
then downloads the matching pre-compiled binary from the GitHub Releases page.
All subsequent `spy-code` commands are forwarded to this binary.

## Links

- [GitHub Repository](https://github.com/Psyborgs-git/spy-code)
- [Issue Tracker](https://github.com/Psyborgs-git/spy-code/issues)
- [npm Package](https://www.npmjs.com/package/spy-code)
