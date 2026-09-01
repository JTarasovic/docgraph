+++

id = "task:add-changeset-query-assertions"
type = "task"
state = "backlog"

[properties]
title = "Add query assertions to changesets"

[[relations]]
type = "part_of"
target = "plan:deliver-declarative-mutation-changesets"

[[relations]]
type = "implements"
target = "plan:deliver-declarative-mutation-changesets#s-ZFNY7HNMAZ"

[[relations]]
type = "depends_on"
target = "task:apply-explicit-mutation-changesets"

[docgraph_generated]
schema_version = 1

[[docgraph_generated.incoming]]
source = "task:add-query-selected-bulk-mutations"
predicate = "depends_on"
target = "task:add-changeset-query-assertions"

[[docgraph_generated.inverses]]
source = "task:add-changeset-query-assertions"
type = "required_by"
target = "task:add-query-selected-bulk-mutations"

+++
<a id="s-39S0TT4MW3"></a>
# Add query assertions to changesets

Allow a changeset to declare named-query preconditions such as an empty blocker set,
an expected entity state, or an exact result count. Assertions run against the same
snapshot used to construct the prospective graph and cannot invoke mutations or
provider effects.

Use the existing named-query ABI and typed arguments. Diagnostics should identify the
query, supplied arguments, expected condition, and bounded actual result without
requiring an agent to reconstruct the failed predicate manually.

<a id="s-9930ND1STZ"></a>
## Acceptance

- Changesets support typed named-query assertions with equality and cardinality checks.
- Assertions are evaluated once against the transaction input snapshot.
- A failed assertion produces no writes and returns actionable structured output.
- Query rules remain syntactically and operationally side-effect-free.
- Tests cover empty, singleton, multiple, stale-preview, and ill-typed results.
