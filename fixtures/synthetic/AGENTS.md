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
- `florp`; workflow `florp` — A deliberately unfamiliar entity type.
- `grommit`; workflow `grommit` — A second unfamiliar entity type with an independent workflow.

Relations:
- `annotated_by`: `section` → `section`; inverse `annotates` — The source section is annotated by the target section.
- `annotates`: `section` → `section`; inverse `annotated_by` — The source section annotates the target section.
- `echoed_by`: `florp` → `florp`; inverse `echoes` — The source florp is echoed by the target.
- `echoes`: `florp` → `florp`; inverse `echoed_by` — The source florp echoes the target; cycles are allowed.
- `follows`: `florp` → `florp`; inverse `precedes`; acyclic — The source florp follows the target.
- `grommits`: `florp` → `external`; inverse `grommitted_by` — The source grommits an external target.
- `grommitted_by`: `external` → `florp`; inverse `grommits` — The source external target is grommitted by the target florp.
- `precedes`: `florp` → `florp`; inverse `follows`; acyclic — The source florp must precede the target.

Workflows:
- `florp`; initial `queued`: `queued` → `active`; `active` → `queued`, `done`; `done` (terminal)
- `grommit`; initial `idle`: `idle` → `running`; `finished` (terminal); `running` → `finished`

Named queries:
- `docgraph query florp_details --arg florp=<value>` — Returns typed entity properties and expanded array items.
- `docgraph query grommit_confidence --arg florp=<value>` — Returns a typed relation property.
- `docgraph query grommit_targets --arg florp=<value>` — Targets grommitted by a florp.
- `docgraph query reachable_florps --arg source=<value>` — Returns the transitive precedence closure from a florp.
- `docgraph query ready_florps` — Returns florps with no incoming precedence edge.
- `docgraph query scalar_values` — Exercises typed result transport.

Common operations:
- Inspect: `docgraph describe`, `docgraph get`, `docgraph search`, `docgraph neighbors`, and `docgraph path`.
- Mutate: `docgraph transition`, `docgraph property`, `docgraph relate`, `docgraph unrelate`, and `docgraph normalize`.
- Maintain: `docgraph validate`, `docgraph frontmatter`, and `docgraph instructions`.
<!-- docgraph:agent-instructions:end -->
