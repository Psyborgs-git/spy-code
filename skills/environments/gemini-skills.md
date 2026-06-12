# Gemini-Specific Skills

## Gemini Integration Overview

Gemini integrates with spy-code to understand codebase structure and provide more contextually relevant suggestions and completions. This can be achieved through MCP (Model Context Protocol) where supported, or by using `spy-code` as a standalone CLI tool or downloadable skill.

## Configuration & Setup

### Running MCP Locally
If you are using Gemini within an environment that supports MCP, you can serve the spy-code MCP server locally. To start it:
```bash
spy-code serve --mcp
```
Then, point your MCP client to the command `spy-code` with arguments `["serve", "--mcp"]` and environment variable `SPY_CODE_CONFIG_PATH="${workspaceFolder}/spy.config.json"`.

### Using as a CLI Tool / Downloadable Skill
You can use `spy-code` via the command line to feed context directly to Gemini:
```bash
# Example: Pass callers to your Gemini prompt
spy-code callers <node_id> > context.txt
```

## When to Use

Use this skill when you need to:
- Trace code execution and call flows for Gemini's reasoning
- Determine what files/functions need updates when adding new features
- Retrieve function signatures and exact context instead of dumping whole files into prompts
- Search for specific symbols semantically rather than relying on regex or `grep`
- Diagnose complex bugs across multiple modules

## Available Tools

### MCP / Programmatic Tools
- `search` - Find code nodes by name or description
- `get_node` - Retrieve details of a specific node
- `callers` - List all callers of a node
- `callees` - List all callees of a node
- `changed_since` - Check modified files

### CLI Commands
- `spy-code search <query>`
- `spy-code get <node_id>`
- `spy-code callers <node_id>`
- `spy-code callees <node_id>`
- `spy-code changed <git_ref>`

## Best Practices

1. **Leverage the Call Graph**: Instead of asking Gemini to guess relationships, use `spy-code callers` or `spy-code callees` to explicitly pass the structural data.
2. **Narrow Down Context**: Do not dump entire repositories. Use `search` to pinpoint the function, and `get` to grab just what you need.
3. **Automate with Scripts**: Build a small bash script that wraps Gemini's CLI and `spy-code` commands to stream exact context for complex refactoring tasks.
4. **Use Semantic Search**: For conceptually complex code, rely on the embeddings feature to find related patterns before prompting Gemini.
