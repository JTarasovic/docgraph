# Repository guidance

This repository builds docgraph. The product contract lives in `docs/reference/`; keep implementation and tests aligned with it. Use the pinned mise environment and run `mise run check` before handing off changes.

## Dogfooding boundary

Docgraph manages only the configured `docs/reference` corpus. Use its CLI for managed frontmatter changes; prose remains directly editable. Do not interpret ordinary links as graph policy. Generated docgraph instructions may extend this file inside their marked block; they do not replace these implementation instructions.

<!-- docgraph:agent-instructions:v1:begin -->
This repository uses docgraph.

- Edit prose directly. Use `docgraph` commands for managed frontmatter and semantic relationships.
- Inspect the repository model with `docgraph describe`; do not reconstruct semantic impact with grep.
- Preview substantial changes with `--dry-run`, then run `docgraph validate`.
- Keep generated frontmatter current with `docgraph frontmatter sync`.
- Portable guidance lives in `skills/docgraph/SKILL.md`.

Model: entities [reference]; relations []; workflows []; queries [].
<!-- docgraph:agent-instructions:end -->
