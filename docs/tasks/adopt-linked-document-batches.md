+++

id = "task:adopt-linked-document-batches"
type = "task"
state = "done"

[properties]
title = "Adopt linked documents as a batch"

[[relations]]
type = "part_of"
target = "plan:coherent-corpus-mutations"

[[relations]]
type = "implements"
target = "plan:coherent-corpus-mutations#s-8R16RRY4GN"

[[relations]]
type = "depends_on"
target = "task:initialize-workflow-states"

[docgraph_generated]
schema_version = 1

[[docgraph_generated.incoming]]
source = "issue:multi-file-adoption-normalize-first"
predicate = "affects"
target = "task:adopt-linked-document-batches"

[[docgraph_generated.inverses]]
source = "task:adopt-linked-document-batches"
type = "affected_by"
target = "issue:multi-file-adoption-normalize-first"

[[docgraph_generated.backlinks]]
source = "plan:coherent-corpus-mutations#s-8R16RRY4GN"
target = "docs/tasks/adopt-linked-document-batches.md"

+++
<a id="s-HAYP5TA3WV"></a>
# Adopt linked documents as a batch

Allow one `docgraph adopt` invocation to normalize and adopt multiple linked documents, validating and writing the complete batch atomically.
