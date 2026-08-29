+++

id = "task:expose-complete-ontology-dump"
type = "task"
state = "done"

[properties]
title = "Expose a complete ontology dump"

[[relations]]
type = "part_of"
target = "plan:complete-v1-readiness"

[[relations]]
type = "implements"
target = "reference:design"

[[relations]]
type = "implements"
target = "reference:config-grammar"

[docgraph_generated]
schema_version = 1

+++
<a id="s-J2S05DNF48"></a>
# Expose a complete ontology dump

Add a single describe operation that returns the full configured model in
human-readable and stable JSON forms, with reference documentation and CLI
regression coverage.

<a id="s-6754NR6EJC"></a>
## Result

Implemented `docgraph describe --all`, documented it in the design and reference
grammar, and added synthetic coverage for every top-level definition family and
project settings. The full repository check suite passes with 102 tests.
