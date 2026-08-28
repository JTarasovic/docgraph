# Repository maintenance

Run `docgraph validate` on merged worktrees and in CI. Derived state is per-worktree
and disposable; canonical Markdown and `.docgraph` configuration remain authoritative.
Keep agent guidance current with `docgraph instructions check` and `sync`.

Use `docgraph review <git-ref>` to inspect graph-level impact separately from the
ordinary Markdown diff. Add `--json` when another tool or agent will consume it.
