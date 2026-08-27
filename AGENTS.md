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

## Docgraph repository model

Entity types:
- `decision`; workflow `decision` — A recorded architectural or product decision.
- `plan`; workflow `plan` — A bounded outcome comprising related work.
- `reference` — A docgraph product reference document.
- `task`; workflow `task` — A concrete unit of work.

Relations:
- `contains`: `plan` → `task`; inverse `part_of` — The source plan contains the target task.
- `depends_on`: `task` → `task`; inverse `required_by`; acyclic — The source cannot be completed before the target.
- `implemented_by`: `reference`, `decision`, `section` → `plan`, `task`; inverse `implements` — The source requirement or decision is implemented by the target work.
- `implements`: `plan`, `task` → `reference`, `decision`, `section`; inverse `implemented_by` — The source performs work required by the target.
- `part_of`: `task` → `plan`; inverse `contains` — The source task contributes to the target plan.
- `required_by`: `task` → `task`; inverse `depends_on`; acyclic — The source must be completed before the target.

Workflows:
- `decision`; initial `proposed`: `proposed` → `accepted`, `rejected`; `accepted` → `superseded`; `rejected` (terminal); `superseded` (terminal)
- `plan`; initial `proposed`: `proposed` → `active`, `abandoned`; `abandoned` (terminal); `active` → `completed`, `abandoned`; `completed` (terminal)
- `task`; initial `backlog`: `backlog` → `ready`, `dropped`; `blocked` → `ready`, `dropped`; `done` (terminal); `dropped` (terminal); `in_progress` → `done`, `blocked`, `dropped`; `ready` → `in_progress`, `dropped`

Named queries:
- None configured.

Common operations:
- Inspect: `docgraph describe`, `docgraph get`, `docgraph search`, `docgraph neighbors`, and `docgraph path`.
- Mutate: `docgraph transition`, `docgraph property`, `docgraph relate`, `docgraph unrelate`, and `docgraph normalize`.
- Maintain: `docgraph validate`, `docgraph frontmatter`, and `docgraph instructions`.
<!-- docgraph:agent-instructions:end -->
