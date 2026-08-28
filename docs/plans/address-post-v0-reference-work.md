+++

id = "plan:address-post-v0-reference-work"
type = "plan"
state = "active"

[properties]
title = "Address post-v0 reference work"

[[relations]]
type = "implements"
target = "reference:design#s-DRW3RR84VS"

[[relations]]
type = "implements"
target = "reference:config-grammar#s-P73QA8YDQB"

[[relations]]
type = "implements"
target = "reference:scenarios#s-N6Z4YKP9M0"

[docgraph_generated]
schema_version = 1

[[docgraph_generated.incoming]]
source = "task:add-provider-reference-adapters"
predicate = "implements"
target = "plan:address-post-v0-reference-work#s-GD85CN51TD"

[[docgraph_generated.incoming]]
source = "task:add-provider-reference-adapters"
predicate = "part_of"
target = "plan:address-post-v0-reference-work"

[[docgraph_generated.incoming]]
source = "task:add-semantic-change-review"
predicate = "implements"
target = "plan:address-post-v0-reference-work#s-DDADARDJPM"

[[docgraph_generated.incoming]]
source = "task:add-semantic-change-review"
predicate = "part_of"
target = "plan:address-post-v0-reference-work"

[[docgraph_generated.incoming]]
source = "task:add-vector-retrieval"
predicate = "implements"
target = "plan:address-post-v0-reference-work#s-WMVD1SYHND"

[[docgraph_generated.incoming]]
source = "task:add-vector-retrieval"
predicate = "part_of"
target = "plan:address-post-v0-reference-work"

[[docgraph_generated.incoming]]
source = "task:complete-structured-retrieval-surface"
predicate = "implements"
target = "plan:address-post-v0-reference-work#s-Y29SFYQYFQ"

[[docgraph_generated.incoming]]
source = "task:complete-structured-retrieval-surface"
predicate = "part_of"
target = "plan:address-post-v0-reference-work"

[[docgraph_generated.incoming]]
source = "task:implement-managed-document-lifecycle"
predicate = "implements"
target = "plan:address-post-v0-reference-work#s-9FHDT151FB"

[[docgraph_generated.incoming]]
source = "task:implement-managed-document-lifecycle"
predicate = "part_of"
target = "plan:address-post-v0-reference-work"

[[docgraph_generated.incoming]]
source = "task:implement-stable-section-lifecycle"
predicate = "implements"
target = "plan:address-post-v0-reference-work#s-RDNDG7T5KN"

[[docgraph_generated.incoming]]
source = "task:implement-stable-section-lifecycle"
predicate = "part_of"
target = "plan:address-post-v0-reference-work"

[[docgraph_generated.incoming]]
source = "task:optimize-repeated-graph-computation"
predicate = "implements"
target = "plan:address-post-v0-reference-work#s-18CTD41F5E"

[[docgraph_generated.incoming]]
source = "task:optimize-repeated-graph-computation"
predicate = "part_of"
target = "plan:address-post-v0-reference-work"

[[docgraph_generated.incoming]]
source = "task:reconcile-post-v0-reference-accounting"
predicate = "implements"
target = "plan:address-post-v0-reference-work#s-K6ZPQ3E59H"

[[docgraph_generated.incoming]]
source = "task:reconcile-post-v0-reference-accounting"
predicate = "part_of"
target = "plan:address-post-v0-reference-work"

[[docgraph_generated.inverses]]
source = "plan:address-post-v0-reference-work"
type = "contains"
target = "task:add-provider-reference-adapters"

[[docgraph_generated.inverses]]
source = "plan:address-post-v0-reference-work"
type = "contains"
target = "task:add-semantic-change-review"

[[docgraph_generated.inverses]]
source = "plan:address-post-v0-reference-work"
type = "contains"
target = "task:add-vector-retrieval"

[[docgraph_generated.inverses]]
source = "plan:address-post-v0-reference-work"
type = "contains"
target = "task:complete-structured-retrieval-surface"

[[docgraph_generated.inverses]]
source = "plan:address-post-v0-reference-work"
type = "contains"
target = "task:implement-managed-document-lifecycle"

[[docgraph_generated.inverses]]
source = "plan:address-post-v0-reference-work"
type = "contains"
target = "task:implement-stable-section-lifecycle"

[[docgraph_generated.inverses]]
source = "plan:address-post-v0-reference-work"
type = "contains"
target = "task:optimize-repeated-graph-computation"

[[docgraph_generated.inverses]]
source = "plan:address-post-v0-reference-work"
type = "contains"
target = "task:reconcile-post-v0-reference-accounting"

[[docgraph_generated.inverses]]
source = "plan:address-post-v0-reference-work#s-18CTD41F5E"
type = "implemented_by"
target = "task:optimize-repeated-graph-computation"

[[docgraph_generated.inverses]]
source = "plan:address-post-v0-reference-work#s-9FHDT151FB"
type = "implemented_by"
target = "task:implement-managed-document-lifecycle"

[[docgraph_generated.inverses]]
source = "plan:address-post-v0-reference-work#s-DDADARDJPM"
type = "implemented_by"
target = "task:add-semantic-change-review"

[[docgraph_generated.inverses]]
source = "plan:address-post-v0-reference-work#s-GD85CN51TD"
type = "implemented_by"
target = "task:add-provider-reference-adapters"

[[docgraph_generated.inverses]]
source = "plan:address-post-v0-reference-work#s-K6ZPQ3E59H"
type = "implemented_by"
target = "task:reconcile-post-v0-reference-accounting"

[[docgraph_generated.inverses]]
source = "plan:address-post-v0-reference-work#s-RDNDG7T5KN"
type = "implemented_by"
target = "task:implement-stable-section-lifecycle"

[[docgraph_generated.inverses]]
source = "plan:address-post-v0-reference-work#s-WMVD1SYHND"
type = "implemented_by"
target = "task:add-vector-retrieval"

[[docgraph_generated.inverses]]
source = "plan:address-post-v0-reference-work#s-Y29SFYQYFQ"
type = "implemented_by"
target = "task:complete-structured-retrieval-surface"

+++
<a id="s-35SBAPSX9V"></a>
# Address post-v0 reference work

<a id="s-YCZRKSEE4D"></a>
## Objective

Resolve every capability explicitly deferred by the initial reference set through implementation or an evidence-based decision to leave it out.

<a id="s-TTBX988WB7"></a>
## Scope

This is the post-v0 roadmap, not a reopening of v0. The sequence favors safe semantic authoring and review before integrations and performance work.

<a id="s-7JSKG1RP56"></a>
## Steps

<a id="s-K6ZPQ3E59H"></a>
### Reconcile the reference accounting

Update the reference set to distinguish already-delivered repository commands and command introspection from work that actually remains.

<a id="s-9FHDT151FB"></a>
### Complete managed document lifecycle operations

Add coherent create, move, and delete operations for managed documents and entities, including inbound-reference checks, generated projections, recovery, and change validation.

<a id="s-RDNDG7T5KN"></a>
### Complete stable-section lifecycle operations

Add safe section split, merge, and delete operations that preserve stable identities where possible and require explicit disposition of durable inbound references.

<a id="s-DDADARDJPM"></a>
### Add semantic change review

Explain entity, section, property, workflow, and relation changes between Git states in human-readable and structured output. Keep Git and whole-corpus validation as the merge model unless concrete failures justify semantic merge machinery.

<a id="s-Y29SFYQYFQ"></a>
### Complete the structured retrieval surface

Add dedicated directional traversal and expanded context commands over the existing graph primitives.

<a id="s-GD85CN51TD"></a>
### Add offline provider reference adapters

Normalize configured repository-host shorthand without requiring network access or remote enrichment.

<a id="s-WMVD1SYHND"></a>
### Add pluggable vector retrieval

Introduce embedding-provider configuration, changed-chunk indexing, and semantic retrieval without coupling the core binary to one model or service.

<a id="s-18CTD41F5E"></a>
### Measure and optimize repeated computation

Benchmark realistic corpora after the functional work and add cross-command parse caching or persistent inferred-fact materialization only where measurements justify the added invalidation complexity.

<a id="s-5FM8Y51PQS"></a>
## Completion

The initial reference set accurately accounts for delivered behavior, every retained post-v0 capability has conformance coverage, and deliberately omitted machinery is recorded with its rationale.
