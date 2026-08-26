# Troubleshooting

Run `docgraph validate --json` for source-located diagnostics. Resolve malformed
managed markers manually rather than asking sync to guess. If recovery reports an
unknown file state, reconcile that file before retrying the mutation.
