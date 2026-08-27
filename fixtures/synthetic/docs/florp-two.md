+++

id = "florp:2"
type = "florp"
state = "queued"

[properties]
title = "Florp two"

[[relations]]
type = "precedes"
target = "florp:3"

[[relations]]
type = "echoes"
target = "florp:1"

[docgraph_generated]
schema_version = 1

[[docgraph_generated.incoming]]
source = "florp:1"
predicate = "echoes"
target = "florp:2"

[[docgraph_generated.incoming]]
source = "florp:1"
predicate = "precedes"
target = "florp:2"

[[docgraph_generated.incoming]]
source = "florp:1#s-9K8J7H6G5F"
predicate = "annotates"
target = "florp:2#s-9D9KQWAJ82"

[[docgraph_generated.inverses]]
source = "florp:2"
type = "echoed_by"
target = "florp:1"

[[docgraph_generated.inverses]]
source = "florp:2"
type = "follows"
target = "florp:1"

[[docgraph_generated.inverses]]
source = "florp:2#s-9D9KQWAJ82"
type = "annotated_by"
target = "florp:1#s-9K8J7H6G5F"

+++
<a id="s-X9XBNG1EHK"></a>
# Florp two

The middle of a transitive precedence chain.

<a id="s-9D9KQWAJ82"></a>
## Detail

This section participates in an exact section relation.
