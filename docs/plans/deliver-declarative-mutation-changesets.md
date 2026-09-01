+++

id = "plan:deliver-declarative-mutation-changesets"
type = "plan"
state = "proposed"

[properties]
title = "Deliver declarative mutation changesets"

[[relations]]
type = "implements"
target = "decision:query-selected-typed-mutations"

[[relations]]
type = "implements"
target = "reference:design#s-B7542FYPRY"

[[relations]]
type = "implements"
target = "reference:config-grammar#s-V5R4RB2AP1"

[docgraph_generated]
schema_version = 1

[[docgraph_generated.incoming]]
source = "task:add-changeset-query-assertions"
predicate = "implements"
target = "plan:deliver-declarative-mutation-changesets#s-ZFNY7HNMAZ"

[[docgraph_generated.incoming]]
source = "task:add-changeset-query-assertions"
predicate = "part_of"
target = "plan:deliver-declarative-mutation-changesets"

[[docgraph_generated.incoming]]
source = "task:add-query-selected-bulk-mutations"
predicate = "implements"
target = "plan:deliver-declarative-mutation-changesets#s-5D0GGWWNZ4"

[[docgraph_generated.incoming]]
source = "task:add-query-selected-bulk-mutations"
predicate = "part_of"
target = "plan:deliver-declarative-mutation-changesets"

[[docgraph_generated.incoming]]
source = "task:apply-explicit-mutation-changesets"
predicate = "implements"
target = "plan:deliver-declarative-mutation-changesets#s-0PKDVP6B03"

[[docgraph_generated.incoming]]
source = "task:apply-explicit-mutation-changesets"
predicate = "part_of"
target = "plan:deliver-declarative-mutation-changesets"

[[docgraph_generated.incoming]]
source = "task:define-declarative-changeset-contract"
predicate = "implements"
target = "plan:deliver-declarative-mutation-changesets#s-ANAQEQ10W4"

[[docgraph_generated.incoming]]
source = "task:define-declarative-changeset-contract"
predicate = "part_of"
target = "plan:deliver-declarative-mutation-changesets"

[[docgraph_generated.inverses]]
source = "plan:deliver-declarative-mutation-changesets"
type = "contains"
target = "task:add-changeset-query-assertions"

[[docgraph_generated.inverses]]
source = "plan:deliver-declarative-mutation-changesets"
type = "contains"
target = "task:add-query-selected-bulk-mutations"

[[docgraph_generated.inverses]]
source = "plan:deliver-declarative-mutation-changesets"
type = "contains"
target = "task:apply-explicit-mutation-changesets"

[[docgraph_generated.inverses]]
source = "plan:deliver-declarative-mutation-changesets"
type = "contains"
target = "task:define-declarative-changeset-contract"

[[docgraph_generated.inverses]]
source = "plan:deliver-declarative-mutation-changesets#s-0PKDVP6B03"
type = "implemented_by"
target = "task:apply-explicit-mutation-changesets"

[[docgraph_generated.inverses]]
source = "plan:deliver-declarative-mutation-changesets#s-5D0GGWWNZ4"
type = "implemented_by"
target = "task:add-query-selected-bulk-mutations"

[[docgraph_generated.inverses]]
source = "plan:deliver-declarative-mutation-changesets#s-ANAQEQ10W4"
type = "implemented_by"
target = "task:define-declarative-changeset-contract"

[[docgraph_generated.inverses]]
source = "plan:deliver-declarative-mutation-changesets#s-ZFNY7HNMAZ"
type = "implemented_by"
target = "task:add-changeset-query-assertions"

+++
<a id="s-7N7WS5RACB"></a>
# Deliver declarative mutation changesets

<a id="s-JJ1YCTD6VH"></a>
## Objective

Let a human or agent describe one multi-entity semantic change, preview its complete
effect, and apply it atomically without issuing one CLI command per document, property,
transition, or relationship.

<a id="s-BTV041PA9G"></a>
## Portfolio priority

Treat explicit changesets as near-term agent and authoring infrastructure after the CI
parity repair. They may proceed alongside release and portable-skill work and should
precede attempts to make large external-provider workflows conveniently mutable.

<a id="s-9J38SQCMRQ"></a>
## Principles

- Reuse typed mutation operations rather than introducing raw patches.
- Keep query evaluation pure.
- Resolve and validate one prospective graph for the complete local batch.
- Make preview output deterministic and bind apply to the previewed repository state.
- Represent remote effects as a visibly separate, non-atomic authority boundary.

<a id="s-YHMQ4GKWK8"></a>
## Sequence

1. Define the manifest, execution, output, safety, and compatibility contract.
2. Implement explicit local operations as one recoverable transaction.
3. Add named-query assertions as preconditions without allowing writes from rules.
4. Add bounded query-selected targets with resolved-target previews.

<a id="s-KH47J6TDCG"></a>
## Work slices

<a id="s-ANAQEQ10W4"></a>
### Define the declarative changeset contract

Specify typed operations, references between newly created entities, ordering,
prospective validation, fingerprints, output, failures, and authority boundaries.

<a id="s-0PKDVP6B03"></a>
### Apply explicit mutation changesets atomically

Execute explicit document, property, workflow, relation, normalization, and maintenance
operations through the existing safe mutation machinery.

<a id="s-ZFNY7HNMAZ"></a>
### Add query assertions to changesets

Allow named-query results to guard a batch while keeping queries read-only.

<a id="s-5D0GGWWNZ4"></a>
### Add query-selected bulk mutations

Resolve bounded query results into typed operation targets and require an unchanged
preview before application.

<a id="s-G1BWNERS30"></a>
## Completion

The backlog-planning scenario can be expressed in one reviewable manifest, previewed
with no writes, and applied as one local transaction. A mid-batch error leaves no
partial canonical changes. Query assertions and selections are deterministic and
bounded, and no Datalog construct performs a side effect.
