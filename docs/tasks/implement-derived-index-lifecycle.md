+++

id = "task:implement-derived-index-lifecycle"
type = "task"
state = "done"

[properties]
title = "Implement the derived-index lifecycle"

[[relations]]
type = "part_of"
target = "plan:close-initial-design-gaps"

[[relations]]
type = "implements"
target = "plan:close-initial-design-gaps#s-0C1QAG3ZHE"

[[relations]]
type = "implements"
target = "reference:design#s-7BBMBXC9RK"

[docgraph_generated]
schema_version = 1

[[docgraph_generated.incoming]]
source = "task:complete-safe-read-mutation-boundary"
predicate = "depends_on"

[[docgraph_generated.incoming]]
source = "task:expand-initial-design-conformance"
predicate = "depends_on"

[[docgraph_generated.inverses]]
type = "required_by"
target = "task:complete-safe-read-mutation-boundary"

[[docgraph_generated.inverses]]
type = "required_by"
target = "task:expand-initial-design-conformance"

[[docgraph_generated.backlinks]]
source = "plan:close-initial-design-gaps#s-0C1QAG3ZHE"

+++
<a id="s-YEX1YPXA6S"></a>
# Implement the derived-index lifecycle

Replace the marker file with a deterministic per-worktree SQLite index for graph facts, source locations, metadata, and FTS. Read commands rebuild or refresh it when the canonical fingerprint changes and never consume stale derived state.
