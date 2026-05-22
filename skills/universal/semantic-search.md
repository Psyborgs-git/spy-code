# Semantic Search Skill

## When to Use

Use this skill when you need to:
- Find code by meaning rather than exact names
- Ask natural language questions about the codebase
- Discover related functionality
- Find code that implements a concept
- Search for "how to" patterns
- Locate code when you don't know the exact terminology

## Available Tools

### MCP Tools
- `ask` - Ask natural language questions about the codebase
- `semanticSearchEmbeddings` - Vector-based semantic search
- `embeddingsStatus` - Check if embeddings are generated

### CLI Commands
- `spy-code ask "natural language query"` - Ask questions
- `spy-code embed [--full]` - Generate embeddings
- `spy-code query '{ semanticSearchEmbeddings(...) }'` - GraphQL semantic search

## Prerequisites

Semantic search requires embeddings to be generated first:

```bash
# Generate embeddings
spy-code embed

# Check status
spy-code query '{ embeddingsStatus { totalNodes processedNodes status } }'
```

## Example Queries

### Natural language questions
```bash
spy-code ask "how do I authenticate users?"
spy-code ask "where is the database connection configured?"
spy-code ask "how do I handle errors in API requests?"
spy-code ask "find code that processes user input"
```

### GraphQL semantic search
```graphql
{
  semanticSearchEmbeddings(query: "authentication", limit: 10) {
    node {
      name
      description
      filePath
      kind
    }
    score
  }
}
```

### Find implementation patterns
```bash
spy-code ask "show me examples of dependency injection"
spy-code ask "where are validation functions?"
```

## Best Practices

1. **Generate embeddings first** - Run `spy-code embed` before using semantic search
2. **Use specific questions** - More specific queries yield better results
3. **Check scores** - Higher scores indicate better matches
4. **Combine with keyword search** - Use both semantic and keyword search for best results
5. **Re-embed after changes** - Run `spy-code embed --full` after significant code changes

## Common Patterns

### Pattern 1: Understand "How To"
```bash
# Ask how something is done
spy-code ask "how do I validate user input?"

# Review the results
# Look at the implementation
# Follow the pattern
```

### Pattern 2: Find Related Code
```bash
# Find code related to a concept
spy-code ask "code that handles file uploads"

# Explore the results
# Understand the architecture
# Find similar patterns
```

### Pattern 3: Discovery
```bash
# Discover functionality you didn't know existed
spy-code ask "what logging mechanisms are available?"

# Learn about the codebase
# Find useful utilities
```

### Pattern 4: Code Review
```bash
# Find similar implementations
spy-code ask "other functions that parse JSON"

# Compare implementations
# Identify best practices
# Suggest improvements
```

## Embedding Model

Spy-Code uses the `all-MiniLM-L6-v2` model:
- **Dimensions**: 384
- **Size**: ~80MB
- **Language**: Multi-lingual (optimized for English)
- **Performance**: Good balance of accuracy and speed

## Managing Embeddings

### Generate embeddings
```bash
# Initial generation
spy-code embed

# Force full re-generation
spy-code embed --full

# Use custom model path
spy-code embed --model .spy-code/models/all-MiniLM-L6-v2
```

### Check status
```graphql
{
  embeddingsStatus {
    totalNodes
    processedNodes
    status
    startedAt
    completedAt
  }
}
```

### Performance considerations
- Generation time: ~100-500ms per node
- Storage: ~1.5KB per embedding
- Memory: ~200MB RAM when model is loaded

## Query Tips

### Good queries
- "how do I authenticate users"
- "error handling in API requests"
- "database connection setup"
- "file upload processing"

### Less effective queries
- "auth" (too short, use keyword search instead)
- "the thing that does stuff" (too vague)
- Single words (use keyword search)

## Combining Searches

For best results, combine semantic and keyword search:

```bash
# Keyword search for exact matches
spy-code search "authenticate"

# Semantic search for related concepts
spy-code ask "user authentication"

# Use both to get comprehensive results
```

## Error Handling

- If semantic search returns no results, embeddings may not be generated
- If results seem irrelevant, try rephrasing your query
- If embeddings generation fails, check disk space and model files
- If queries are slow, the model may still be loading

## Limitations

- Semantic search is only as good as the code comments and names
- Generated embeddings need to be updated after code changes
- Model is optimized for English, other languages may have lower quality
- Very large codebases may take time to embed
