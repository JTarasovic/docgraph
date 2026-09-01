# docgraph

Use docgraph for repository-native document graphs. In an unconfigured Git
repository, preview `docgraph init --dry-run`, then run `docgraph init` to create
the minimal configuration and install compatible guidance.

Choose the workflow by what you are trying to do:

| If you need to... | Start with... | Then read... |
| --- | --- | --- |
| Learn the repository's types, properties, relations, states, queries, or commands | `docgraph describe --all` | `config-authorship.md`, `commands.md` |
| Find entities, dependencies, inferred results, or a long document's structure | `get`, `outline`, `neighbors`, `context`, `search`, or `query` | `querying.md`, `relationships.md` |
| Create, adopt, move, or remove a document | `docgraph document --help` or `docgraph adopt --help` | `document-authoring.md`, `mutations.md` |
| Edit prose or add Markdown headings | Edit the Markdown directly | `document-authoring.md` |
| Change state or typed metadata | `transition`, `workflow`, or `property` | `workflows.md`, `mutations.md` |
| Add or remove semantic edges | `relate` or `unrelate` | `relationships.md` |
| Convert imported YAML frontmatter | `frontmatter migrate --dry-run` | `document-authoring.md`, `mutations.md` |
| Diagnose a failed command or invalid repository | Read the full diagnostic, then `docgraph validate` | `troubleshooting.md` |
| Change configuration, logic, generated guidance, or repository maintenance | Inspect first with `describe --all` | `config-authorship.md`, `repository-maintenance.md` |

Sequencing rules that prevent common stalls:

- After adding headings directly, if those headings should remain, preview the
  repository-wide ID pass with `docgraph normalize --dry-run`, then apply it.
- After direct Markdown-link or relation-source edits, use `docgraph frontmatter
  sync --dry-run` when generated incoming/inverse projections may have changed.
  Docgraph mutation commands already converge generated frontmatter themselves.
- Preview substantial mutations with `--dry-run`. After managed or structural
  changes, run `docgraph validate`; before committing, also run `docgraph validate
  --changes <git-ref>` when reviewing supported managed changes.
- Use graph/query commands instead of reconstructing semantic dependencies with
  filesystem search. Edit prose directly, but use docgraph commands for managed
  frontmatter, states, properties, relations, normalization, and lifecycle changes.
