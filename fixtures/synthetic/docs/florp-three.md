+++

id = "florp:3"
type = "florp"
state = "queued"

[properties]
title = "Florp three"

[docgraph_generated]
schema_version = 1

[[docgraph_generated.incoming]]
source = "florp:2"
predicate = "precedes"
target = "florp:3"

[[docgraph_generated.inverses]]
source = "florp:3"
type = "follows"
target = "florp:2"

+++
<a id="s-YW5MA33J8W"></a>
# Florp three

The end of a transitive precedence chain.
