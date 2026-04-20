# Git Integration

Implemented in `crates/spy-git` using `std::process::Command` shelling out to `git`. Optional but on by default.

## Goals

1. Skip re-indexing files that haven't changed since the last index commit.
2. Track which nodes belong to which commit so `changedSince(ref)` works.
3. Detect file renames so node IDs migrate cleanly.

## Mechanics

### On `spy-code index`

1. Read `index_meta.last_git_sha`. If absent → full index, store current `HEAD` SHA.
2. If present → run `git diff --name-status <last_sha>..HEAD`:
   - `M` (modified) → re-parse, upsert nodes, recompute edges.
   - `A` (added) → parse, insert nodes.
   - `D` (deleted) → cascade-delete nodes for that file.
   - `R<score>` (renamed) → update `file_path` and `node_id` for affected nodes; set `renamed_from` to prior ID.
3. Combine the candidate set with content-hash check: even if git says modified, skip if hash matches stored hash. (Handles whitespace-only or `.gitattributes` smudge cases.)
4. After indexing, write new `HEAD` SHA to `index_meta.last_git_sha`.

### Outside a git repo

Falls back to pure content-hash mode. Walks all files matching language roots, hashes, compares, re-parses what changed. No `changedSince` query support — returns an error.

### Dirty working tree

`spy-code index` indexes the working tree as-is. The stored `git_sha` reflects HEAD plus a `+dirty` suffix when the working tree differs from HEAD. `changedSince` queries against a dirty index emit a warning.

## `changedSince` query

```graphql
query { changedSince(ref: "HEAD~10") { id name filePath } }
```

Implementation:

1. Resolve `ref` via `git` to a commit SHA.
2. Run `git diff --name-only <ref>..HEAD` for the set of changed files.
3. Return all nodes whose `file_path` is in that set.

This is a file-granularity answer, not signature-granularity. A future enhancement: hash signatures (params+returns+description) and detect node-level changes by comparing signature hashes across commits — out of scope for v1.

## Rename detection

Rename detection uses similarity index with a default 50% threshold. When triggered:

- Old node IDs are NOT deleted immediately.
- New node IDs are inserted with `renamed_from` pointing at the old ID.
- A background sweep at end of indexing deletes old IDs whose `renamed_from` successor exists. This gives downstream tools a single transaction window to capture the rename.

## Edge cases

- **Submodules**: skipped in v1.
- **Symlinks**: controlled by `git.follow_symlinks` config flag (default off).
- **Shallow clones**: `changedSince` errors if the requested ref is outside the shallow boundary.
- **Force-push / rebased history**: if `last_git_sha` is no longer reachable, fall back to a full re-index automatically and log a warning.
