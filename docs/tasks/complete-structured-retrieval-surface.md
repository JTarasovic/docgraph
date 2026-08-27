+++

id = "task:complete-structured-retrieval-surface"
type = "task"
state = "backlog"

[properties]
title = "Complete the structured retrieval surface"

[[relations]]
type = "part_of"
target = "plan:address-post-v0-reference-work"

[[relations]]
type = "implements"
target = "plan:address-post-v0-reference-work#s-Y29SFYQYFQ"

[[relations]]
type = "depends_on"
target = "task:reconcile-post-v0-reference-accounting"

[docgraph_generated]
schema_version = 1

[[docgraph_generated.incoming]]
source = "task:add-vector-retrieval"
predicate = "depends_on"
target = "task:complete-structured-retrieval-surface"

[[docgraph_generated.inverses]]
source = "task:complete-structured-retrieval-surface"
type = "required_by"
target = "task:add-vector-retrieval"

+++
<a id="s-REAFWN1PV7"></a>
# Complete the structured retrieval surface

Add dedicated incoming, outgoing, arbitrary-depth traversal, and expanded context commands over the existing indexed graph.
