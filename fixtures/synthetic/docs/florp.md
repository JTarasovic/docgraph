+++

id = "florp:1"
type = "florp"
state = "queued"

[properties]
title = "Florp one"
count = 7
score = 2.5
enabled = true
observed = 2026-08-26T12:30:00Z
labels = ["odd", "novel"]

[[relations]]
type = "grommits"
target = "#123"
confidence = 0.75

[[relations]]
type = "precedes"
target = "florp:2"

[[relations]]
type = "echoes"
target = "florp:2"

[[relations]]
source = "florp:1#s-9K8J7H6G5F"
type = "annotates"
target = "florp:2#s-9D9KQWAJ82"

[docgraph_generated]
schema_version = 1

[[docgraph_generated.incoming]]
source = "florp:2"
predicate = "echoes"
target = "florp:1"

[[docgraph_generated.inverses]]
source = "florp:1"
type = "echoed_by"
target = "florp:2"

+++
<a id="s-9K8J7H6G5F"></a>
# Florp one

This fixture proves that novel ontology requires no repository-specific Rust.
