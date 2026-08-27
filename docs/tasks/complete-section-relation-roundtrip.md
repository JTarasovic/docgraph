+++

id = "task:complete-section-relation-roundtrip"
type = "task"
state = "ready"

[properties]
title = "Complete stable-section relation round trips"

[[relations]]
type = "part_of"
target = "plan:close-initial-design-gaps"

[[relations]]
type = "implements"
target = "plan:close-initial-design-gaps#s-40M5GN7XPN"

[[relations]]
type = "implements"
target = "reference:config-grammar#s-Q9K2W13EGT"

[[relations]]
type = "implements"
target = "reference:config-grammar#s-KNXSZ8RYR4"

[docgraph_generated]
schema_version = 1

[[docgraph_generated.incoming]]
source = "task:expand-initial-design-conformance"
predicate = "depends_on"

[[docgraph_generated.inverses]]
type = "required_by"
target = "task:expand-initial-design-conformance"

[[docgraph_generated.backlinks]]
source = "plan:close-initial-design-gaps#s-40M5GN7XPN"

+++
<a id="s-Q79M28EF01"></a>
# Complete stable-section relation round trips

Allow explicit stable sections as relation sources in CLI mutations and preserve exact source and target section identities in graph output and generated frontmatter without collapsing distinct edges.
