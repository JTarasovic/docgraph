+++

id = "task:enable-external-provider-mutations"
type = "task"
state = "backlog"

[properties]
title = "Enable capability-gated external mutations"

[[relations]]
type = "part_of"
target = "plan:complete-external-provider-ontology"

[[relations]]
type = "implements"
target = "plan:complete-external-provider-ontology#s-MS6ZGSFZXN"

[[relations]]
type = "depends_on"
target = "task:project-external-entities-into-ontology"

[[relations]]
type = "depends_on"
target = "task:expand-github-external-entity-kinds"

[docgraph_generated]
schema_version = 1

[[docgraph_generated.incoming]]
source = "task:dogfood-external-ontology"
predicate = "depends_on"
target = "task:enable-external-provider-mutations"

[[docgraph_generated.inverses]]
source = "task:enable-external-provider-mutations"
type = "required_by"
target = "task:dogfood-external-ontology"

+++
<a id="s-BNEJ4YXZA7"></a>
# Enable capability-gated external mutations

Complete the mutation side of
[#13](https://github.com/JTarasovic/docgraph/issues/13) for operations accepted by the
authority contract. Extend generic property and workflow operations, or add an equally
coherent provider-neutral mutation surface, without embedding GitHub-specific verbs in
the graph model.

Remote writes are not repository-atomic. Each operation must therefore show an exact
preview, use provider concurrency controls when available, refresh derived state after
success, and return a recoverable per-target result when a batch partially succeeds.
Unsupported capabilities fail before any write.

<a id="s-G159D98WG6"></a>
## Acceptance

- Providers advertise transition, property, and relationship mutation capabilities
  independently.
- Dry-run performs no remote writes and describes the intended provider operations.
- Stale preconditions and unsupported operations fail before mutation.
- Success refreshes the derived record; partial failure is explicit and recoverable.
- GitHub mutation tests use a controlled boundary and never require live network access.
