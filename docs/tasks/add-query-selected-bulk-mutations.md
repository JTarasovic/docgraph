+++

id = "task:add-query-selected-bulk-mutations"
type = "task"
state = "backlog"

[properties]
title = "Add query-selected bulk mutations"

[[relations]]
type = "part_of"
target = "plan:deliver-declarative-mutation-changesets"

[[relations]]
type = "implements"
target = "plan:deliver-declarative-mutation-changesets#s-5D0GGWWNZ4"

[[relations]]
type = "depends_on"
target = "task:add-changeset-query-assertions"

[[relations]]
type = "depends_on"
target = "task:apply-explicit-mutation-changesets"

[docgraph_generated]
schema_version = 1

+++
<a id="s-P6XTPS96VB"></a>
# Add query-selected bulk mutations

Allow a typed changeset operation to obtain its targets from a named query. The query
selects entity or section references only; the enclosing operation still determines
the transition, property, relation, or lifecycle behavior.

Require explicit minimum and maximum cardinality, deterministic result ordering, and a
preview that enumerates every resolved target. Bind apply to the preview fingerprint so
query results cannot silently widen or change between review and mutation. Refuse
operations whose query ABI cannot supply the target shape required by the mutation.

<a id="s-490VQN4HBC"></a>
## Acceptance

- Query selectors use the existing named-query ABI and typed values.
- Every selector declares enforced cardinality bounds.
- Preview lists the exact ordered target set and apply rejects snapshot drift.
- Duplicate targets are normalized deterministically or rejected by documented policy.
- Logic evaluation performs no side effect; all writes remain typed mutation operations.
