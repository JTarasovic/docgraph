+++

id = "task:apply-explicit-mutation-changesets"
type = "task"
state = "backlog"

[properties]
title = "Apply explicit mutation changesets atomically"

[[relations]]
type = "part_of"
target = "plan:deliver-declarative-mutation-changesets"

[[relations]]
type = "implements"
target = "plan:deliver-declarative-mutation-changesets#s-0PKDVP6B03"

[[relations]]
type = "depends_on"
target = "task:define-declarative-changeset-contract"

[docgraph_generated]
schema_version = 1

[[docgraph_generated.incoming]]
source = "task:add-changeset-query-assertions"
predicate = "depends_on"
target = "task:apply-explicit-mutation-changesets"

[[docgraph_generated.incoming]]
source = "task:add-query-selected-bulk-mutations"
predicate = "depends_on"
target = "task:apply-explicit-mutation-changesets"

[[docgraph_generated.inverses]]
source = "task:apply-explicit-mutation-changesets"
type = "required_by"
target = "task:add-changeset-query-assertions"

[[docgraph_generated.inverses]]
source = "task:apply-explicit-mutation-changesets"
type = "required_by"
target = "task:add-query-selected-bulk-mutations"

+++
<a id="s-DY9F95FHMP"></a>
# Apply explicit mutation changesets atomically

Implement explicit local changesets through shared typed mutation primitives. Parse and
resolve the complete manifest, build one prospective repository, validate once, emit a
deterministic aggregate preview, and use the existing lock, optimistic hash, journal,
atomic replacement, and derived-index recovery mechanisms for the complete batch.

Avoid spawning the CLI recursively or applying each operation as an independently
committed mutation. Ordinary commands and changesets must call the same underlying
operation implementations so their validation and diagnostics cannot drift.

<a id="s-C90203PKCQ"></a>
## Acceptance

- One manifest can create multiple documents and wire properties, states, and relations
  among both new and existing graph nodes.
- A dry run writes nothing and shows every canonical and generated-file patch.
- Parse, resolution, validation, or write failure leaves no partial canonical batch.
- Apply refuses when the repository differs from the previewed input fingerprint.
- Network-independent integration coverage replaces the prior planning orchestration
  with one changeset and verifies idempotent replay behavior.
