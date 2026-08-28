+++

id = "task:implement-managed-document-lifecycle"
type = "task"
state = "done"

[properties]
title = "Implement managed document lifecycle"

[[relations]]
type = "part_of"
target = "plan:address-post-v0-reference-work"

[[relations]]
type = "implements"
target = "plan:address-post-v0-reference-work#s-9FHDT151FB"

[[relations]]
type = "depends_on"
target = "task:reconcile-post-v0-reference-accounting"

[docgraph_generated]
schema_version = 1

[[docgraph_generated.incoming]]
source = "issue:document-create-title-does-not-set-property"
predicate = "affects"
target = "task:implement-managed-document-lifecycle"

[[docgraph_generated.incoming]]
source = "task:add-semantic-change-review"
predicate = "depends_on"
target = "task:implement-managed-document-lifecycle"

[[docgraph_generated.incoming]]
source = "task:implement-stable-section-lifecycle"
predicate = "depends_on"
target = "task:implement-managed-document-lifecycle"

[[docgraph_generated.inverses]]
source = "task:implement-managed-document-lifecycle"
type = "affected_by"
target = "issue:document-create-title-does-not-set-property"

[[docgraph_generated.inverses]]
source = "task:implement-managed-document-lifecycle"
type = "required_by"
target = "task:add-semantic-change-review"

[[docgraph_generated.inverses]]
source = "task:implement-managed-document-lifecycle"
type = "required_by"
target = "task:implement-stable-section-lifecycle"

+++
<a id="s-2QCK3Z34N4"></a>
# Implement managed document lifecycle

Add prospective, recoverable create, move, and delete operations for managed documents and entities, including exact impact reporting and inbound-reference safety.

Implemented through `docgraph document create|move|delete`. Creation initializes a
validated document inside the configured corpus; moves preserve stable identity and
rewrite resolvable relative Markdown links; deletion reports and refuses inbound
managed or Markdown references. File creation and absence are journaled states, so
create, move, and delete recover through the same transaction path as existing
mutations. Dry-run and JSON output expose every affected path.
