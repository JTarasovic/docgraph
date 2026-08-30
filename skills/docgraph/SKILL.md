# docgraph

Use docgraph for repository-native document graphs. In an unconfigured Git
repository, preview `docgraph init --dry-run`, then run `docgraph init` to create
the minimal configuration and install compatible guidance.

- Inspect the model with `docgraph describe` before changing managed semantics.
- Edit prose directly. Use docgraph commands for state, relationships, normalization, and generated frontmatter.
- Use `--dry-run` for substantial mutations and run `docgraph validate` afterward.
- Use graph/query commands instead of reconstructing semantic dependencies with filesystem search.

Read the task guide that matches the work: `config-authorship.md`, `querying.md`,
`mutations.md`, `workflows.md`, `relationships.md`, `document-authoring.md`,
`troubleshooting.md`, or `repository-maintenance.md`.
