+++

id = "task:initialize-workflow-states"
type = "task"
state = "done"

[properties]
title = "Initialize workflow states atomically"

[[relations]]
type = "part_of"
target = "plan:coherent-corpus-mutations"

[[relations]]
type = "implements"
target = "plan:coherent-corpus-mutations#s-D27VK82Z90"

[docgraph_generated]
schema_version = 1

[[docgraph_generated.incoming]]
source = "issue:workflow-adoption-initial-state"
predicate = "affects"
target = "task:initialize-workflow-states"

[[docgraph_generated.incoming]]
source = "task:adopt-linked-document-batches"
predicate = "depends_on"
target = "task:initialize-workflow-states"

[[docgraph_generated.inverses]]
source = "task:initialize-workflow-states"
type = "affected_by"
target = "issue:workflow-adoption-initial-state"

[[docgraph_generated.inverses]]
source = "task:initialize-workflow-states"
type = "required_by"
target = "task:adopt-linked-document-batches"

[[docgraph_generated.backlinks]]
source = "plan:coherent-corpus-mutations#s-D27VK82Z90"
target = "docs/tasks/initialize-workflow-states.md"

+++
<a id="s-DZ8YHD6B4P"></a>
# Initialize workflow states atomically

Add `docgraph workflow initialize <entity-type>` to materialize the configured initial state for every affected entity in one validated, recoverable mutation.
