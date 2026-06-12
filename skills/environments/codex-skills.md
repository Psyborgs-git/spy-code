# Codex-Specific Skills

## Codex Integration Overview

OpenAI Codex integration allows the model to utilize `spy-code`'s static analysis to generate highly accurate and context-aware code. Codex can leverage `spy-code` via MCP (Model Context Protocol) or directly via CLI outputs.

## Configuration & Setup

### Running MCP Locally
To expose the codebase to Codex via an MCP-compatible client:
```bash
spy-code serve --mcp
```
Configure your client to execute the `spy-code` binary with `serve --mcp` and provide the workspace config via the `SPY_CODE_CONFIG_PATH` environment variable.

### Using as a CLI Tool / Downloadable Skill
When building Codex wrappers or prompts, run `spy-code` CLI commands to gather required context:
```bash
# Get context for a function and pipe to your Codex script
spy-code get src:main.rs:_:main | codex-cli "Refactor this"
```

## When to Use

Use this skill when you need to:
- Generate tests with exact understanding of all dependencies
- Complete complex functions that rely on specific internal APIs
- Understand legacy codebase structure before making suggestions
- Find examples of how a specific library or module is used within the project
- Navigate call graphs to figure out edge cases

## Available Tools

### MCP / Programmatic Tools
- `search` - Locate functions, classes, or variables
- `get_node` - Retrieve exact signatures and source code for a node
- `callers` - Get functions calling the current node
- `callees` - Get functions called by the current node

### CLI Commands
- `spy-code search <query>`
- `spy-code callers <node_id> --depth 2`
- `spy-code callees <node_id>`
- `spy-code query <custom_graphql>`

## Best Practices

1. **Pre-fetch Signatures**: Before asking Codex to write a function, pre-fetch the signatures of the utilities it will need using `spy-code get`.
2. **Provide Caller Context**: If Codex is rewriting a function, give it the list of callers so it knows how the output is being consumed.
3. **Minimize Noise**: Instead of providing whole files, use the precise data from `spy-code` to keep token counts low and Codex focus high.
4. **Iterative Refinement**: Query the graph, give Codex the summary, and then use deeper queries based on its initial plan.
