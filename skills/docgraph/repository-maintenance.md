# Repository maintenance

Run `docgraph validate` on merged worktrees and in CI. Derived state is per-worktree
and disposable; canonical Markdown and `.docgraph` configuration remain authoritative.
Keep agent guidance current with `docgraph instructions check` and `sync`.

`instructions check` also verifies the version-compatible portable skill at
`skills/docgraph`. It reports missing, modified, and incompatible bundles.
Preview repairs with `docgraph instructions sync --dry-run`; applying `sync`
replaces only the versioned managed skill files embedded in the CLI. Additional
repository-owned files in that directory are preserved.

Use `docgraph review <git-ref>` to inspect graph-level impact separately from the
ordinary Markdown diff. Add `--json` when another tool or agent will consume it.
