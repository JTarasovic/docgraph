+++

id = "task:project-external-entities-into-ontology"
type = "task"
state = "backlog"

[properties]
title = "Project external entities into the repository ontology"

[[relations]]
type = "part_of"
target = "plan:complete-external-provider-ontology"

[[relations]]
type = "implements"
target = "plan:complete-external-provider-ontology#s-JY82FBNEZT"

[[relations]]
type = "depends_on"
target = "task:define-external-ontology-authority"

[docgraph_generated]
schema_version = 1

[[docgraph_generated.incoming]]
source = "task:dogfood-external-ontology"
predicate = "depends_on"
target = "task:project-external-entities-into-ontology"

[[docgraph_generated.incoming]]
source = "task:enable-external-provider-mutations"
predicate = "depends_on"
target = "task:project-external-entities-into-ontology"

[[docgraph_generated.inverses]]
source = "task:project-external-entities-into-ontology"
type = "required_by"
target = "task:dogfood-external-ontology"

[[docgraph_generated.inverses]]
source = "task:project-external-entities-into-ontology"
type = "required_by"
target = "task:enable-external-provider-mutations"

+++
<a id="s-W0EF77Q1F1"></a>
# Project external entities into the repository ontology

Implement the accepted external mapping contract for
[#13](https://github.com/JTarasovic/docgraph/issues/13). Mapped remote nodes should
participate in the same generic inspection, logic, typed-property, workflow-state, and
relation-endpoint surfaces as repository documents while retaining derived provenance
and freshness.

Do not silently alias external facts into canonical ones. Structured output and logic
must expose enough authority metadata for callers to distinguish an authored fact from
a fresh or stale provider projection.

<a id="s-PFA3ZB27R5"></a>
## Acceptance

- `get`, `describe`, `context`, traversal, search, and named queries expose configured
  external type, property, state, and relation projections consistently.
- Typed relation validation accepts mapped external endpoints and rejects unmapped or
  incompatible ones.
- Workflow inspection can explain projected state and whether transitions are supported.
- Stale or unavailable enrichment never invents a current type, state, or property.
- Existing repositories without mappings preserve their current external behavior.
