+++

id = "task:define-declarative-changeset-contract"
type = "task"
state = "backlog"

[properties]
title = "Define the declarative changeset contract"

[[relations]]
type = "part_of"
target = "plan:deliver-declarative-mutation-changesets"

[[relations]]
type = "implements"
target = "plan:deliver-declarative-mutation-changesets#s-ANAQEQ10W4"

[docgraph_generated]
schema_version = 1

[[docgraph_generated.incoming]]
source = "task:apply-explicit-mutation-changesets"
predicate = "depends_on"
target = "task:define-declarative-changeset-contract"

[[docgraph_generated.inverses]]
source = "task:define-declarative-changeset-contract"
type = "required_by"
target = "task:apply-explicit-mutation-changesets"

+++
<a id="s-CJTYA8XPA4"></a>
# Define the declarative changeset contract

Extend the product references with a provider-neutral declarative changeset format and
an `apply` command contract. Cover explicit creates, adopts, moves, deletes, property
changes, workflow transitions, relations, section operations, normalization, and
maintenance operations without reducing any operation to a raw text patch.

Specify manifest-local references to newly created entities and sections, deterministic
operation ordering, duplicate and contradictory operations, prospective validation,
dry-run output, canonical input fingerprints, optimistic concurrency, recovery,
idempotency, schema versioning, and structured diagnostics. Define which operations
are local-atomic and how future provider operations declare a separate execution phase.

<a id="s-FR7EPJKW58"></a>
## Acceptance

- The grammar and design references include a complete versioned manifest schema.
- Every operation maps to an existing or explicitly proposed typed mutation contract.
- A batch has one prospective graph and one all-or-nothing local commit boundary.
- Preview/apply freshness and manifest idempotency have deterministic semantics.
- Remote effects cannot be mistaken for members of the local atomic transaction.
