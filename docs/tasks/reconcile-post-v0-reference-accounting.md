+++

id = "task:reconcile-post-v0-reference-accounting"
type = "task"
state = "done"

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

The reconciliation found no untracked implementation gap. Repository-defined nested
commands, project-aware help, query and mutation dispatch, configuration validation,
and command introspection are delivered and covered. Design section 15.2 now owns the
remaining-work list; the grammar and scenarios link to it instead of maintaining
conflicting copies. Semantic merge is explicitly omitted pending evidence that Git,
whole-corpus validation, and semantic review are insufficient.
