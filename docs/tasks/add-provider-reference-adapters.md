+++

id = "task:add-provider-reference-adapters"
type = "task"
state = "done"

[properties]
title = "Add provider reference adapters"

[[relations]]
type = "part_of"
target = "plan:address-post-v0-reference-work"

[[relations]]
type = "implements"
target = "plan:address-post-v0-reference-work#s-GD85CN51TD"

[[relations]]
type = "depends_on"
target = "task:reconcile-post-v0-reference-accounting"

[docgraph_generated]
schema_version = 1

[[docgraph_generated.incoming]]
source = "task:optimize-repeated-graph-computation"
predicate = "depends_on"
target = "task:add-provider-reference-adapters"

[[docgraph_generated.inverses]]
source = "task:add-provider-reference-adapters"
type = "required_by"
target = "task:optimize-repeated-graph-computation"

+++
<a id="s-7WQSYVPEA1"></a>
# Add provider reference adapters

Normalize configured GitHub- and GitLab-style issue, change, and commit shorthand into opaque external identities entirely offline.

<a id="s-7TRKVRC8DQ"></a>
## Result

Implemented registered GitHub and GitLab reference adapters with explicit or
Git-remote-inferred repository context. Normalization is offline, canonical external
identities include the host, and the adapter boundary leaves remote entity enrichment
as a separate future capability.
