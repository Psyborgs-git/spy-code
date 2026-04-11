# Configuration

`spy.config.json` at repo root. Loaded by `spy-cli` and `spy-mcp` on startup. Hash stored in `index_meta` — config change forces full re-index.

## Schema

```json
{
  "$schema": "https://spy-code.dev/schema/v1.json",
  "version": 1,
  "db_path": ".spy-code/graph.db",
  "languages": {
    "rust": {
      "enabled": true,
      "roots": ["src/", "crates/"],
      "ignore": ["target/", "**/*.generated.rs"],
      "resolver": "builtin"
    },
    "python": {
      "enabled": true,
      "roots": ["./"],
      "ignore": [".venv/", "__pycache__/", "**/*_pb2.py"],
      "resolver": "builtin"
    },
    "typescript": {
      "enabled": true,
      "roots": ["src/", "app/"],
      "ignore": ["node_modules/", "dist/", "build/"],
      "resolver": "builtin",
      "tsconfig": "./tsconfig.json"
    },
    "go": {
      "enabled": false
    }
  },
  "git": {
    "enabled": true,
    "track_renames": true,
    "follow_symlinks": false
  },
  "indexing": {
    "max_file_size_kb": 2048,
    "parallelism": "auto",
    "fail_fast": false
  },
  "search": {
    "fts_tokenizer": "unicode61"
  }
}
```

## Field reference

| Field | Type | Default | Notes |
|---|---|---|---|
| `version` | int | required | schema version |
| `db_path` | string | `.spy-code/graph.db` | per-repo SQLite location |
| `languages.<lang>.enabled` | bool | true | gate per language |
| `languages.<lang>.roots` | string[] | `["./"]` | dirs to scan |
| `languages.<lang>.ignore` | string[] | `[]` | gitignore-style globs |
| `languages.<lang>.resolver` | string | `"builtin"` | future: custom plugins |
| `git.enabled` | bool | true | use git diff to narrow re-index |
| `git.track_renames` | bool | true | populate `renamed_from` |
| `indexing.max_file_size_kb` | int | 2048 | skip larger files |
| `indexing.parallelism` | "auto" \| int | auto | thread pool size |
| `indexing.fail_fast` | bool | false | stop on first error |

## Defaults

If `spy.config.json` is absent, `spy-code init` writes one with all four languages enabled and sensible ignores. Without init, `spy-code index` errors out and asks the user to run `init`.

## Validation

- Loaded via `serde` with strict deny_unknown_fields.
- Unknown languages → error.
- Conflicting roots/ignores → warning, not error.
