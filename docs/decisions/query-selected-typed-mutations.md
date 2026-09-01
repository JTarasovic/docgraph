+++

id = "decision:query-selected-typed-mutations"
type = "decision"
state = "accepted"

[properties]
title = "Keep queries pure and apply typed mutation changesets"

[docgraph_generated]
schema_version = 1

[[docgraph_generated.incoming]]
source = "plan:deliver-declarative-mutation-changesets"
predicate = "implements"
target = "decision:query-selected-typed-mutations"

[[docgraph_generated.inverses]]
source = "decision:query-selected-typed-mutations"
type = "implemented_by"
target = "plan:deliver-declarative-mutation-changesets"

+++
<a id="s-5AHG1GD0KP"></a>
# Keep queries pure and apply typed mutation changesets

<a id="s-A275FQHWKA"></a>
## Context

Docgraph's individual mutation commands are intentionally safe: each command resolves
typed graph references, previews exact patches, validates the prospective repository,
uses optimistic concurrency and recovery journals, and converges generated state.
That boundary becomes cumbersome when one conceptual operation creates and wires many
entities. Building the second GitHub backlog required sixteen document creations and
forty-eight relation mutations. A wrapper script reduced interactive typing, but each
CLI process still loaded and validated separately, the overall operation was not
atomic, and an intermediate failure could leave a valid but incomplete plan.

Restricted Datalog already provides a declarative way to identify graph facts. It is
tempting to add insertion or deletion syntax to that language, but a fact does not
uniquely identify canonical storage. A state fact may come from authored frontmatter,
inference, or an external provider projection; a relationship may be explicit,
generated, informational, or remote. Side effects during rule evaluation would also
make repeated queries unsafe and would couple the logic runtime to filesystems and
providers.

<a id="s-E3E2402NWJ"></a>
## Decision

Repository logic remains pure and side-effect-free. Queries decide what matches;
typed mutation operations decide what changes and how it is validated.

Docgraph will add a declarative changeset format and an `apply` operation. A changeset
can contain the same semantic operations exposed by the ordinary CLI, including
document lifecycle, properties, workflow transitions, relations, normalization, and
supported maintenance operations. The complete local changeset is resolved and
validated against one prospective graph, previewed as one deterministic result, and
then applied atomically or not at all.

The first version requires explicit operation targets. Later versions may use named
queries for assertions and target selection, but selection is an input to a typed
operation rather than a side effect of evaluating a rule. Query-selected operations
must declare cardinality bounds. Preview records the resolved targets and a canonical
input fingerprint; apply refuses if the repository no longer matches that preview.

<a id="s-PPGCHW09HP"></a>
## Authority and remote operations

Canonical repository mutations and provider mutations do not share an atomic commit
boundary. A changeset may eventually plan both, but it must expose separate execution
phases and never claim that a remote write can be rolled back with local files.
Providers advertise capabilities independently. Remote operations require explicit
preconditions, preview without writes, optimistic concurrency where available, refresh
after success, and recoverable per-operation results after partial failure.

<a id="s-SYE8J4HYCC"></a>
## Consequences

- Agents can express one intended graph change without orchestrating dozens of CLI
  processes or hand-editing managed frontmatter.
- Ordinary mutation commands remain the simple path and share their typed operation
  implementations with changeset execution.
- Query and changeset formats remain independently evolvable and testable.
- Dry-run output becomes a durable review boundary rather than advisory console text.
- Scripts remain useful for generating manifests, but correctness and atomicity belong
  to docgraph.

<a id="s-MYSH6A6G1D"></a>
## Rejected alternatives

<a id="s-MK1HMCPXVE"></a>
### Side-effecting Datalog

Rejected because projected and inferred facts do not identify writable authority,
rule evaluation must remain repeatable, and provider effects cannot obey local graph
transaction semantics.

<a id="s-BJ2KAEGMZA"></a>
### Directly generated frontmatter

Rejected because it bypasses typed validation, safe mutation, concurrency checks,
generated projections, and recovery.

<a id="s-A277G4CD9M"></a>
### Repository-specific orchestration scripts

Rejected as the product boundary. A script can reduce typing, but it cannot provide a
single prospective graph, all-or-nothing application, or a portable agent workflow.
