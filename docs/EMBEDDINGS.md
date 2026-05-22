# Embeddings

Vector embeddings for semantic search enable natural language queries over your codebase.

## Overview

The embedding layer generates vector representations of code nodes (functions, classes, constants) using pluggable embedding models. These embeddings enable semantic search that understands the meaning of code rather than just matching keywords.

## Swappable Model Architecture

The embedding system supports multiple model types through a pluggable architecture:

- **Local models**: Rust-based implementations (TF-IDF, Candle ML, ONNX)
- **Python models**: Sentence-transformers via Python subprocess
- **Remote models**: API-based embeddings (OpenAI, Cohere, etc.)

### Default Model

The default model is a simple TF-IDF implementation:
- **Dimensions**: 100
- **Model Type**: Local TF-IDF
- **Advantages**: No external dependencies, fast, works offline
- **Use case**: Quick semantic search without setup

### Configuration

Embedding models are configured in `.spy-code/embedding.config.json`:

```json
{
  "version": 1,
  "default_model": "simple-tfidf",
  "models": {
    "simple-tfidf": {
      "type": "local",
      "implementation": "tfidf",
      "dimension": 100
    },
    "all-MiniLM-L6-v2": {
      "type": "local",
      "implementation": "candle",
      "model_path": ".spy-code/models/all-MiniLM-L6-v2",
      "download_url": "https://huggingface.co/sentence-transformers/all-MiniLM-L6-v2",
      "dimension": 384
    },
    "openai-text-embedding-3-small": {
      "type": "remote",
      "provider": "openai",
      "model": "text-embedding-3-small",
      "api_key_env": "OPENAI_API_KEY",
      "dimension": 1536
    }
  }
}
```

Copy `embedding.config.example.json` to `.spy-code/embedding.config.json` to customize.

## Storage

Embeddings are stored in SQLite alongside the graph data:

- `node_embeddings` - stores embeddings for node names and descriptions
- `source_embeddings` - stores embeddings for full source code (planned)
- `embedding_progress` - tracks embedding generation progress

## Usage

### List available models

```bash
spy-code model list
```

### Generate embeddings

```bash
# Generate embeddings using default model
spy-code embed

# Generate embeddings with specific model
spy-code embed --model simple-tfidf

# Force full re-embedding (ignore existing embeddings)
spy-code embed --full
```

### Check embedding status

```bash
# Via GraphQL
spy-code query '{ embeddingsStatus { totalNodes processedNodes status } }'
```

### Semantic search

```bash
# Natural language query (uses default model)
spy-code ask "how do I authenticate users?"

# Via GraphQL
spy-code query '{ semanticSearchEmbeddings(query: "authentication", limit: 10) { node { name filePath } score } }'
```

## Embedding generation process

1. **Load configuration**: Read `.spy-code/embedding.config.json`
2. **Resolve model**: Get model instance from ModelRegistry
3. **Fetch nodes**: Retrieve all nodes from the database
4. **Generate embeddings**: For each node, generate embedding from name + description
5. **Store embeddings**: Save embeddings to SQLite as BLOB with model name
6. **Track progress**: Update progress table every 10 nodes
7. **Complete**: Mark as complete when all nodes processed

## Performance considerations

### TF-IDF (default)
- **Generation time**: ~1-5ms per node
- **Storage**: ~400 bytes per embedding (100 dimensions × 4 bytes)
- **Memory**: Minimal (<10MB)
- **Accuracy**: Basic keyword-based semantic understanding

### Sentence-transformers (e.g., all-MiniLM-L6-v2)
- **Generation time**: ~100-500ms per node (depends on hardware)
- **Storage**: ~1.5KB per embedding (384 dimensions × 4 bytes)
- **Memory**: Model requires ~200MB RAM when loaded
- **Accuracy**: High-quality semantic understanding

### Remote API models (e.g., OpenAI)
- **Generation time**: Network latency dependent
- **Storage**: Varies by model (e.g., 6KB for 1536 dimensions)
- **Memory**: Minimal (no local model)
- **Accuracy**: State-of-the-art semantic understanding
- **Cost**: Per-token API pricing

## Troubleshooting

### Model not found

If you see "Model 'X' not found in configuration":
- Check that the model is defined in `.spy-code/embedding.config.json`
- Run `spy-code model list` to see available models
- Copy `embedding.config.example.json` as a template

### Implementation not available

If you see "Candle models not yet implemented":
- The model type is configured but not yet implemented
- Use `simple-tfidf` for now (always available)
- Check the documentation for implementation status

### Out of memory

If you encounter memory issues:
- Use the TF-IDF model (lowest memory)
- Process embeddings in smaller batches
- Close other applications

### Slow embedding generation

To speed up embedding generation:
- Use TF-IDF for faster generation
- Use a machine with more CPU cores
- Consider remote API models for large codebases

## Future improvements

- [ ] Candle ML model implementation for local high-quality embeddings
- [ ] Python sentence-transformers integration
- [ ] Remote API model implementation (OpenAI, Cohere)
- [ ] Model download automation
- [ ] Incremental embedding generation (only changed nodes)
- [ ] Source code embeddings (full function bodies)
- [ ] GPU acceleration for local models
- [ ] Hybrid search (keyword + semantic)
- [ ] Embedding caching across runs
