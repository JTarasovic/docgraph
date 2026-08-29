+++

id = "task:optimize-repeated-graph-computation"
type = "task"
state = "done"

[properties]
title = "Optimize repeated graph computation"

[[relations]]
type = "part_of"
target = "plan:address-post-v0-reference-work"

[[relations]]
type = "implements"
target = "plan:address-post-v0-reference-work#s-18CTD41F5E"

[[relations]]
type = "depends_on"
target = "task:add-semantic-change-review"

[[relations]]
type = "depends_on"
target = "task:add-provider-reference-adapters"

[[relations]]
type = "depends_on"
target = "task:add-vector-retrieval"

[[relations]]
type = "depends_on"
target = "task:index-searchable-markdown-content"

[docgraph_generated]
schema_version = 1

+++
<a id="s-KVKGKSYPD2"></a>
# Optimize repeated graph computation

Benchmark representative corpora and implement cross-command parse caching or persistent inferred-fact materialization only when measured costs justify their invalidation and persistence complexity.

<a id="s-12MSESX5WA"></a>
## Result

A 2,100-document, 1,900-relation corpus did not justify either persistence mechanism. Graph-only reads now skip the SQLite index, repository configuration is loaded once per command, and generated-frontmatter facts are prepared once per graph; validation fell from about 4.2 seconds to about 0.6 seconds on the benchmark corpus.
