+++

id = "task:map-external-issues-into-project-work"
type = "task"
state = "done"

[properties]
title = "Map external issues into project work"

[[relations]]
type = "part_of"
target = "plan:deliver-external-entity-sources"

[[relations]]
type = "implements"
target = "plan:deliver-external-entity-sources#s-0J00WB5MDA"

[[relations]]
type = "depends_on"
target = "task:persist-derived-external-entities"

[[relations]]
type = "depends_on"
target = "task:add-github-external-entity-source"

[docgraph_generated]
schema_version = 1

[[docgraph_generated.incoming]]
source = "task:dogfood-github-external-issues"
predicate = "depends_on"
target = "task:map-external-issues-into-project-work"

[[docgraph_generated.inverses]]
source = "task:map-external-issues-into-project-work"
type = "required_by"
target = "task:dogfood-github-external-issues"

+++
<a id="s-9WZ7KM1MAB"></a>
# Map external issues into project work

Expose provider-neutral external entity facts to restricted repository logic, including
identity, provider, remote kind, state, title, URL, freshness, and provider-defined
string attributes where safe. Keep these predicates distinct from canonical
`entity_type`, `entity_state`, and authored properties.

Update this repository's logic so open GitHub issues from the configured origin appear
in project-level work discovery without becoming local `issue` documents. The mapping
must be explicit and repository-authored: remote state alone must not satisfy local
workflow transitions, validation rules, milestone requirements, or typed relation
constraints.

Define deterministic behavior for cached, stale, missing, closed, and inaccessible
issues. A remote issue may remain discoverable by canonical identity when enrichment
is unavailable, but an unknown state must not be guessed to be open or closed.

<a id="s-BN92GZX7SS"></a>
## Acceptance

- Restricted logic can query provider-neutral external facts without GitHub-specific predicates.
- `docgraph next` can include explicitly mapped open remote issues.
- Closed remote issues leave the actionable set after refresh without mutating canonical files.
- Unavailable or stale state is represented honestly and follows documented repository policy.
- No local issue mirror is required for project-level discovery.

<a id="s-AT2BSYAVM5"></a>
## Result

Added provider-neutral external facts to restricted repository logic. This repository
explicitly maps open GitHub issue records from `JTarasovic/docgraph` into `next_work`;
unknown records have no state fact, while fresh and honestly labeled stale cache facts
remain queryable without changing local workflows.
