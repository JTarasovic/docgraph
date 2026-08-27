# Mutations

Preview with `--dry-run`, apply the same command, then validate. Mutations use
per-worktree locks, prospective validation, optimistic hashes, and recovery journals;
do not reproduce their frontmatter edits by hand.

Use `docgraph adopt --batch <manifest.toml> [--dry-run]` when adopting multiple
unnormalized documents together; the manifest contains one `[[document]]` table per
path, ID, type, and optional `property = ["name=value"]` list.
