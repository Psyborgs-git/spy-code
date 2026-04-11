# Node ID Specification

## Format

```
<dir>:<file>:<class>:<symbol>
```

Components are colon-separated. Empty segments collapse to `_` to keep arity fixed at 4.

## Rules

- `<dir>`: repo-relative directory path, slash-normalized. Root = `.`.
- `<file>`: filename **with** extension.
- `<class>`: enclosing class/struct/impl/trait. `_` if top-level.
- `<symbol>`: function, method, or constant name.

## Examples

| Code location | Node ID |
|---|---|
| `src/lib.rs` top-level fn `parse` | `src:lib.rs:_:parse` |
| `src/foo.rs` `impl Bar { fn new() }` | `src:foo.rs:Bar:new` |
| `src/foo.rs` const `MAX_SIZE` | `src:foo.rs:_:MAX_SIZE` |
| `app/utils/db.py` `class DB: def query` | `app/utils:db.py:DB:query` |
| `pkg/handler.go` func `Handle` | `pkg:handler.go:_:Handle` |

## Nested functions

Skipped. Per project decision: nested fns are anti-pattern. Resolver walks past them and only emits edges from the enclosing top-level function.

## Trait impls (Rust)

When the same struct has multiple `impl` blocks (one inherent, multiple trait impls), and a method name collides:

- Inherent `impl Foo { fn new }` → `src:foo.rs:Foo:new`
- `impl Bar for Foo { fn new }` → `src:foo.rs:Foo<Bar>:new`
- `impl Baz for Foo { fn new }` → `src:foo.rs:Foo<Baz>:new`

The `<TraitName>` qualifier is only added when needed to disambiguate.

## Overloads (TypeScript) and `@overload` (Python)

Collapsed into a single node. The node's `signatures` field becomes an array of `{params, returns}` objects. The underlying implementation's name is the symbol.

## Constraints

- Node IDs are stable across runs as long as path + names don't change.
- Renaming a file or moving a dir changes the ID. Git tracking detects renames and emits a `renamed_from` field on the node (see `GIT_INTEGRATION.md`).
- Node IDs are case-sensitive.
- Max length: 512 chars. Longer IDs are an error and the node is skipped with a warning.
