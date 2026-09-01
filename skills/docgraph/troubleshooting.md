# Troubleshooting

Run `docgraph validate --json` for source-located diagnostics. Resolve malformed
managed markers manually rather than asking sync to guess. If recovery reports an
unknown file state, reconcile that file before retrying the mutation.

For `missing-section-id`, if the heading should remain, preview the repository-wide
repair with `docgraph normalize --dry-run`; `normalize` intentionally accepts no path.
For named-query predicate or arity failures, compare `docgraph describe query <name>`
with the predicate vocabulary in `docgraph describe --all` before deciding whether to
change the query declaration or the rule. Follow conditional diagnostic wording: a
suggested command applies only when its stated condition matches the intended corpus.
