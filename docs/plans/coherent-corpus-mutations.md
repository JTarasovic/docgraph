+++

id = "plan:coherent-corpus-mutations"
type = "plan"
state = "completed"

[properties]
title = "Make corpus-wide mutations coherent"

[docgraph_generated]
schema_version = 1

[[docgraph_generated.incoming]]
source = "issue:multi-file-adoption-normalize-first"
predicate = "affects"
target = "plan:coherent-corpus-mutations"

[[docgraph_generated.incoming]]
source = "issue:workflow-adoption-initial-state"
predicate = "affects"
target = "plan:coherent-corpus-mutations"

[[docgraph_generated.incoming]]
source = "task:adopt-linked-document-batches"
predicate = "implements"
target = "plan:coherent-corpus-mutations#s-8R16RRY4GN"

[[docgraph_generated.incoming]]
source = "task:adopt-linked-document-batches"
predicate = "part_of"
target = "plan:coherent-corpus-mutations"

[[docgraph_generated.incoming]]
source = "task:initialize-workflow-states"
predicate = "implements"
target = "plan:coherent-corpus-mutations#s-D27VK82Z90"

[[docgraph_generated.incoming]]
source = "task:initialize-workflow-states"
predicate = "part_of"
target = "plan:coherent-corpus-mutations"

[[docgraph_generated.inverses]]
source = "plan:coherent-corpus-mutations"
type = "affected_by"
target = "issue:multi-file-adoption-normalize-first"

[[docgraph_generated.inverses]]
source = "plan:coherent-corpus-mutations"
type = "affected_by"
target = "issue:workflow-adoption-initial-state"

[[docgraph_generated.inverses]]
source = "plan:coherent-corpus-mutations"
type = "contains"
target = "task:adopt-linked-document-batches"

[[docgraph_generated.inverses]]
source = "plan:coherent-corpus-mutations"
type = "contains"
target = "task:initialize-workflow-states"

[[docgraph_generated.inverses]]
source = "plan:coherent-corpus-mutations#s-8R16RRY4GN"
type = "implemented_by"
target = "task:adopt-linked-document-batches"

[[docgraph_generated.inverses]]
source = "plan:coherent-corpus-mutations#s-D27VK82Z90"
type = "implemented_by"
target = "task:initialize-workflow-states"

+++
<a id="s-31CHY4XV5B"></a>
# Make corpus-wide mutations coherent

<a id="s-2W8ZXKHYF2"></a>
## Objective

Let docgraph stage related document changes, validate their final corpus once, and commit them atomically without exposing invalid intermediate states.

<a id="s-3J1MWRV39F"></a>
## Steps

<a id="s-D27VK82Z90"></a>
### Initialize workflow states atomically

[Initialize workflow states atomically](../tasks/initialize-workflow-states.md) for existing entities when their type gains a workflow.

<a id="s-8R16RRY4GN"></a>
### Adopt linked documents as a batch

[Adopt linked documents as a batch](../tasks/adopt-linked-document-batches.md), normalizing and adopting every candidate before validating the combined result.

<a id="s-RNVKXCPAVC"></a>
## Completion

Both operations support dry runs, preserve recovery guarantees, pass final-state validation, and resolve their corresponding dogfooding issues.
