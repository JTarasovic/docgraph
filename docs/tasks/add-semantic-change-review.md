+++

id = "task:add-semantic-change-review"
type = "task"
state = "done"

[properties]
title = "Add semantic change review"

[[relations]]
type = "part_of"
target = "plan:address-post-v0-reference-work"

[[relations]]
type = "implements"
target = "plan:address-post-v0-reference-work#s-DDADARDJPM"

[[relations]]
type = "depends_on"
target = "task:implement-managed-document-lifecycle"

[[relations]]
type = "depends_on"
target = "task:implement-stable-section-lifecycle"

[docgraph_generated]
schema_version = 1

[[docgraph_generated.incoming]]
source = "task:optimize-repeated-graph-computation"
predicate = "depends_on"
target = "task:add-semantic-change-review"

[[docgraph_generated.inverses]]
source = "task:add-semantic-change-review"
type = "required_by"
target = "task:optimize-repeated-graph-computation"

+++
<a id="s-QGDC8CTKM0"></a>
# Add semantic change review

Report graph-level changes between Git states in text and JSON so humans and agents can review semantic impact independently of Markdown line diffs.

Implemented as `docgraph review <git-ref>`. The deterministic text and JSON reports
cover entity lifecycle and moves, workflow states, individual properties,
stable-section structure, and managed or Markdown-link relation changes. Generated
projections and prose-only edits are excluded; managed-change diagnostics are
included without replacing `docgraph validate`.
