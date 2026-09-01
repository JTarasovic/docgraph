+++

id = "plan:complete-external-provider-ontology"
type = "plan"
state = "proposed"

[properties]
title = "Complete external provider ontology participation"

[[relations]]
type = "implements"
target = "reference:design#s-WCDD32CNPK"

[[relations]]
type = "implements"
target = "reference:config-grammar#s-3MQG5HQA1C"

[docgraph_generated]
schema_version = 1

[[docgraph_generated.incoming]]
source = "task:define-external-ontology-authority"
predicate = "implements"
target = "plan:complete-external-provider-ontology#s-P7C8TRJE3M"

[[docgraph_generated.incoming]]
source = "task:define-external-ontology-authority"
predicate = "part_of"
target = "plan:complete-external-provider-ontology"

[[docgraph_generated.incoming]]
source = "task:dogfood-external-ontology"
predicate = "implements"
target = "plan:complete-external-provider-ontology#s-Q26ZD9F7G3"

[[docgraph_generated.incoming]]
source = "task:dogfood-external-ontology"
predicate = "part_of"
target = "plan:complete-external-provider-ontology"

[[docgraph_generated.incoming]]
source = "task:enable-external-provider-mutations"
predicate = "implements"
target = "plan:complete-external-provider-ontology#s-MS6ZGSFZXN"

[[docgraph_generated.incoming]]
source = "task:enable-external-provider-mutations"
predicate = "part_of"
target = "plan:complete-external-provider-ontology"

[[docgraph_generated.incoming]]
source = "task:expand-github-external-entity-kinds"
predicate = "implements"
target = "plan:complete-external-provider-ontology#s-2JQ6E0559W"

[[docgraph_generated.incoming]]
source = "task:expand-github-external-entity-kinds"
predicate = "part_of"
target = "plan:complete-external-provider-ontology"

[[docgraph_generated.incoming]]
source = "task:project-external-entities-into-ontology"
predicate = "implements"
target = "plan:complete-external-provider-ontology#s-JY82FBNEZT"

[[docgraph_generated.incoming]]
source = "task:project-external-entities-into-ontology"
predicate = "part_of"
target = "plan:complete-external-provider-ontology"

[[docgraph_generated.inverses]]
source = "plan:complete-external-provider-ontology"
type = "contains"
target = "task:define-external-ontology-authority"

[[docgraph_generated.inverses]]
source = "plan:complete-external-provider-ontology"
type = "contains"
target = "task:dogfood-external-ontology"

[[docgraph_generated.inverses]]
source = "plan:complete-external-provider-ontology"
type = "contains"
target = "task:enable-external-provider-mutations"

[[docgraph_generated.inverses]]
source = "plan:complete-external-provider-ontology"
type = "contains"
target = "task:expand-github-external-entity-kinds"

[[docgraph_generated.inverses]]
source = "plan:complete-external-provider-ontology"
type = "contains"
target = "task:project-external-entities-into-ontology"

[[docgraph_generated.inverses]]
source = "plan:complete-external-provider-ontology#s-2JQ6E0559W"
type = "implemented_by"
target = "task:expand-github-external-entity-kinds"

[[docgraph_generated.inverses]]
source = "plan:complete-external-provider-ontology#s-JY82FBNEZT"
type = "implemented_by"
target = "task:project-external-entities-into-ontology"

[[docgraph_generated.inverses]]
source = "plan:complete-external-provider-ontology#s-MS6ZGSFZXN"
type = "implemented_by"
target = "task:enable-external-provider-mutations"

[[docgraph_generated.inverses]]
source = "plan:complete-external-provider-ontology#s-P7C8TRJE3M"
type = "implemented_by"
target = "task:define-external-ontology-authority"

[[docgraph_generated.inverses]]
source = "plan:complete-external-provider-ontology#s-Q26ZD9F7G3"
type = "implemented_by"
target = "task:dogfood-external-ontology"

+++
<a id="s-290A5SC8J8"></a>
# Complete external provider ontology participation

<a id="s-8RM2FEYFKT"></a>
## Objective

Extend external entities from read-only enriched references into explicitly mapped
participants in the repository ontology. GitHub issues, pull requests, milestones,
projects, and project items should be queryable and relatable through provider-neutral
semantics, with state and mutation authority kept explicit.

<a id="s-1V6A0QBGNJ"></a>
## Portfolio priority

Start the contract after the bounded CI and shipped-skill repairs. This is the next
major capability plan and should not interrupt fixes to already advertised delivery
and agent-integration behavior.

<a id="s-8VS62VZT5T"></a>
## Report coverage

This plan covers [#13](https://github.com/JTarasovic/docgraph/issues/13). It is a
separate plan because the completed external-source slice deliberately kept remote
kind and state distinct from canonical `entity_type`, `entity_state`, workflow
authority, and typed relation validation. Changing that boundary requires a product
contract followed by several independently sequenced implementation slices.

<a id="s-PPTKRXK3M4"></a>
## Contract boundary

External records remain derived and their bodies remain untrusted. Participation in
the ontology must therefore distinguish three things that the first slice kept
separate: mapping a provider kind to a repository type, projecting provider state
into a repository workflow, and authorizing a provider mutation. None may happen by
guessing from a remote string; each requires explicit configuration and advertised
provider capability.

<a id="s-HE1ERQFBBR"></a>
## Priority and sequence

1. Define the mapping, authority, identity, freshness, and failure contract.
2. Teach graph construction, logic, relation validation, and workflow inspection to
   use configured external projections without making cache data canonical.
3. Expand the GitHub adapter across the agreed kinds and pagination boundaries.
4. Add previewed, capability-gated remote mutations with honest partial-failure
   semantics.
5. Dogfood the complete model against this repository's GitHub work.

<a id="s-S1C68CV31A"></a>
## Work slices

<a id="s-P7C8TRJE3M"></a>
### Define external ontology and authority

Resolve how provider kinds, fields, states, relationships, and capabilities map into
repository-authored types and workflows.

<a id="s-JY82FBNEZT"></a>
### Project external entities into the repository ontology

Expose mapped external types, states, properties, and relation endpoints throughout
graph, query, retrieval, and validation surfaces.

<a id="s-2JQ6E0559W"></a>
### Expand GitHub external entity kinds

Implement the accepted GitHub issue, pull-request, milestone, project, and project-item
read/search shapes behind the provider-neutral contract.

<a id="s-MS6ZGSFZXN"></a>
### Enable capability-gated external mutations

Route supported state and metadata changes through explicit provider capabilities,
previews, optimistic concurrency, and actionable recovery reporting.

<a id="s-Q26ZD9F7G3"></a>
### Dogfood external ontology participation

Use real GitHub work to verify discovery, typed relations, workflow state, retrieval,
offline behavior, and safe mutations without local mirror documents.

<a id="s-SX0CFEP8E3"></a>
## Completion

The kinds selected by the accepted contract participate in configured entity types,
workflows, relations, logic, and retrieval. Supported mutations are explicit and
safe; unsupported or stale operations fail honestly. GitHub remains canonical for
remote work, and no duplicate local issue, task, plan, or milestone document is
required.
