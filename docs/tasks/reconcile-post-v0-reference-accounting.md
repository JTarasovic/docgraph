+++

id = "task:reconcile-post-v0-reference-accounting"
type = "task"
state = "backlog"

[properties]
title = "Reconcile post-v0 reference accounting"

[[relations]]
type = "part_of"
target = "plan:address-post-v0-reference-work"

[[relations]]
type = "implements"
target = "plan:address-post-v0-reference-work#s-K6ZPQ3E59H"

[docgraph_generated]
schema_version = 1

[[docgraph_generated.incoming]]
source = "task:add-provider-reference-adapters"
predicate = "depends_on"
target = "task:reconcile-post-v0-reference-accounting"

[[docgraph_generated.incoming]]
source = "task:complete-structured-retrieval-surface"
predicate = "depends_on"
target = "task:reconcile-post-v0-reference-accounting"

[[docgraph_generated.incoming]]
source = "task:implement-managed-document-lifecycle"
predicate = "depends_on"
target = "task:reconcile-post-v0-reference-accounting"

[[docgraph_generated.inverses]]
source = "task:reconcile-post-v0-reference-accounting"
type = "required_by"
target = "task:add-provider-reference-adapters"

[[docgraph_generated.inverses]]
source = "task:reconcile-post-v0-reference-accounting"
type = "required_by"
target = "task:complete-structured-retrieval-surface"

[[docgraph_generated.inverses]]
source = "task:reconcile-post-v0-reference-accounting"
type = "required_by"
target = "task:implement-managed-document-lifecycle"

+++
<a id="s-1J1RFT5A09"></a>
# Reconcile post-v0 reference accounting

Correct stale deferred-work lists, especially repository-defined commands and command introspection that are already implemented, and establish one authoritative accounting of the remaining roadmap.
