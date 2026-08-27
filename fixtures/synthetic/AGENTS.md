# Synthetic fixture guidance

This user-authored text must survive generated instruction updates.

<!-- docgraph:agent-instructions:v1:begin -->
This repository uses docgraph.

- Edit prose directly. Use `docgraph` commands for managed frontmatter and semantic relationships.
- Inspect the repository model with `docgraph describe`; do not reconstruct semantic impact with grep.
- Preview substantial changes with `--dry-run`, then run `docgraph validate`.
- Keep generated frontmatter current with `docgraph frontmatter sync`.
- Portable guidance lives in `skills/docgraph/SKILL.md`.

## Docgraph repository model

Entity types:
- `florp` — A deliberately unfamiliar entity type.

Relations:
- `grommits`: `florp` → `external` — The source grommits an external target.

Workflows:
- None configured.

Named queries:
- `docgraph query florp_details --arg florp=<value>` — Returns typed entity properties and expanded array items.
- `docgraph query grommit_confidence --arg florp=<value>` — Returns a typed relation property.
- `docgraph query grommit_targets --arg florp=<value>` — Targets grommitted by a florp.
- `docgraph query scalar_values` — Exercises typed result transport.

Common operations:
- Inspect: `docgraph describe`, `docgraph get`, `docgraph search`, `docgraph neighbors`, and `docgraph path`.
- Mutate: `docgraph transition`, `docgraph property`, `docgraph relate`, `docgraph unrelate`, and `docgraph normalize`.
- Maintain: `docgraph validate`, `docgraph frontmatter`, and `docgraph instructions`.
<!-- docgraph:agent-instructions:end -->
