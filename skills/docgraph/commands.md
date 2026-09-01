# Commands

Run `docgraph --help` for workflow-grouped generic and repository-defined commands,
then `docgraph <command> --help` for scenario-derived examples. Dot-separated names
in `.docgraph/commands.toml` become command paths and map to generic queries, workflow
transitions, or relation mutations. Use `docgraph query` as the generic escape hatch
for named queries. In structured relation results, use `source` and `target` for edge
endpoints and `neighbor` for the adjacent node; `node` is a v0.x compatibility alias.
