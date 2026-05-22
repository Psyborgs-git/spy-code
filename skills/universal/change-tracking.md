# Change Tracking Skill

## When to Use

Use this skill when you need to:
- Find code that changed since a git commit
- Understand what changed between branches
- Identify affected code after a rebase
- Track code evolution
- Find nodes that need re-reading after updates
- Analyze pull request impact

## Available Tools

### MCP Tools
- `changed_since` - List nodes whose source changed since a given git ref
- `query_graph` - Run raw GraphQL queries

### CLI Commands
- `spy-code changed <git_ref>` - Find changed nodes
- `spy-code index` - Re-index the codebase

## Example Queries

### Find changes since last commit
```bash
spy-code changed HEAD~1
```

### Find changes since a specific commit
```bash
spy-code changed abc123def456
```

### Find changes since a branch
```bash
spy-code changed origin/main
```

### GraphQL query for changed nodes
```graphql
{
  changedSince(ref: "HEAD~5") {
    name
    kind
    filePath
    startLine
    endLine
    gitSha
  }
}
```

### Find changes with full details
```graphql
{
  changedSince(ref: "origin/main") {
    name
    description
    signatures {
      params { name type }
      returns
    }
    filePath
    kind
    language
  }
}
```

## Best Practices

1. **Use after rebase/merge** - Check what changed after rebasing
2. **Combine with call graph** - Understand impact of changes
3. **Re-index when needed** - Run `spy-code index` after significant changes
4. **Use meaningful refs** - Use branch names or commit hashes, not just HEAD
5. **Track gitSha** - Compare gitSha values to verify indexing

## Common Patterns

### Pattern 1: Post-Rebase Analysis
```bash
# After rebasing, find what changed
spy-code changed origin/main

# Review the changed nodes
# Re-index if needed
spy-code index

# Verify the changes
```

### Pattern 2: Pull Request Impact
```bash
# Find changes in a PR
spy-code changed main

# Analyze the changed functions
# Check callers to understand impact
spy-code callers src:api:handlers:_:updated_function

# Review affected code
```

### Pattern 3: Branch Comparison
```bash
# Compare feature branch to main
spy-code changed main

# Understand what the feature changes
# Identify potential conflicts
# Plan integration
```

### Pattern 4: Regression Testing
```bash
# Find changed code since last release
spy-code changed v1.0.0

# Focus testing on changed areas
# Check callers of changed functions
# Prioritize testing based on impact
```

## Git Reference Formats

You can use any git reference that `git rev-parse` accepts:

- Commit hashes: `abc123def456`
- Branch names: `origin/main`, `feature/new-auth`
- Tags: `v1.0.0`
- Relative refs: `HEAD~1`, `HEAD~5`
- HEAD: `HEAD`

## Integration with Git

Spy-Code integrates with git to:
- Track the last indexed commit (`last_git_sha` in index_meta)
- Skip unchanged files during re-indexing
- Detect file renames
- Track per-node git commit (`git_sha` in nodes table)

### Check last indexed commit
```graphql
{
  stats {
    lastGitSha
  }
}
```

### Re-index only changed files
```bash
# Standard index (git-aware, incremental)
spy-code index

# Force full re-index
spy-code index --full
```

## Change Detection

Spy-Code detects changes via:
1. **Git diff** - Compares current tree to last indexed commit
2. **Content hash** - BLAKE3 hash of source code
3. **File modification time** - Fallback when git is unavailable

Files are re-indexed if:
- Git shows them as changed
- Content hash differs
- File is new or deleted

## Rename Tracking

When git detects renames, Spy-Code tracks them:
- Original node ID stored in `renamed_from` field
- Enables tracking of code across renames
- Maintains call graph continuity

### Query renamed nodes
```graphql
{
  changedSince(ref: "HEAD~10") {
    name
    renamedFrom
  }
}
```

## Performance Considerations

- `changed_since` queries are fast (uses git metadata)
- Re-indexing is incremental by default (only changed files)
- Large diffs may take time to process
- Use `--full` flag sparingly

## Error Handling

- If git ref is invalid, the query will fail
- If working tree is dirty, changes may not be detected accurately
- If last_git_sha is missing, fall back to full index
- If rename detection fails, nodes may appear as new/deleted

## Use Cases

### For AI Agents
- Re-read only changed nodes after code updates
- Focus attention on modified code
- Understand context of changes
- Validate that changes are consistent

### For Developers
- Review what changed in a branch
- Understand impact of pull requests
- Track code evolution
- Identify areas needing testing

### For Code Review
- Focus review on changed functions
- Check callers of changed code
- Verify consistency of changes
- Identify potential side effects
