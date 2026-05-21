# Embeddings

Vector embeddings for semantic search enable natural language queries over your codebase.

## Overview

The embedding layer generates vector representations of code nodes (functions, classes, constants) using open-source embedding models. These embeddings enable semantic search that understands the meaning of code rather than just matching keywords.

## Embedding Model

We use the `all-MiniLM-L6-v2` model from sentence-transformers:
- **Dimensions**: 384
- **Model Type**: Sentence transformer
- **Language**: Multi-lingual (optimized for English)
- **Size**: ~80MB
- **License**: Apache 2.0

This model provides a good balance between accuracy and performance for code-related text.

## Storage

Embeddings are stored in SQLite alongside the graph data:

- `node_embeddings` - stores embeddings for node names and descriptions
- `source_embeddings` - stores embeddings for full source code (planned)
- `embedding_progress` - tracks embedding generation progress

## Usage

### Generate embeddings

```bash
# Generate embeddings using default model
spy-code embed

# Generate embeddings with specific model path
spy-code embed --model .spy-code/models/all-MiniLM-L6-v2

# Force full re-embedding (ignore existing embeddings)
spy-code embed --full
```

### Check embedding status

```bash
# Via GraphQL
spy-code query '{ embeddingsStatus { totalNodes processedNodes status } }'

# Via CLI (planned)
spy-code embed status
```

### Semantic search

```bash
# Natural language query
spy-code ask "how do I authenticate users?"

# Via GraphQL
spy-code query '{ semanticSearchEmbeddings(query: "authentication", limit: 10) { node { name filePath } score } }'
```

## Embedding generation process

1. **Load model**: Load the sentence-transformer model from disk
2. **Fetch nodes**: Retrieve all nodes from the database
3. **Generate embeddings**: For each node, generate embedding from name + description
4. **Store embeddings**: Save embeddings to SQLite as BLOB
5. **Track progress**: Update progress table every 10 nodes
6. **Complete**: Mark as complete when all nodes processed

## Performance considerations

- **Generation time**: ~100-500ms per node (depends on hardware)
- **Storage**: ~1.5KB per embedding (384 dimensions × 4 bytes)
- **Memory**: Model requires ~200MB RAM when loaded
- **Batch size**: Processes nodes one at a time to minimize memory usage

## Troubleshooting

### Model not found

If you see "Failed to load model", ensure the model files are present:
```bash
# Download model to expected location
mkdir -p .spy-code/models/all-MiniLM-L6-v2
# Download model files (config.json, tokenizer.json, model.safetensors)
```

### Out of memory

If you encounter memory issues:
- Use a smaller model
- Process embeddings in smaller batches
- Close other applications

### Slow embedding generation

To speed up embedding generation:
- Use a machine with more CPU cores
- Consider GPU acceleration (not yet implemented)
- Generate embeddings only for changed files (planned)

## Future improvements

- [ ] Incremental embedding generation (only changed nodes)
- [ ] Source code embeddings (full function bodies)
- [ ] GPU acceleration for faster generation
- [ ] Hybrid search (keyword + semantic)
- [ ] Embedding caching across runs
- [ ] Support for custom embedding models
