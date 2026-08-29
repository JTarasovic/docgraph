+++

id = "issue:expose-complete-ontology-dump"
type = "issue"
state = "open"

[properties]
title = "Expose a complete ontology dump"

[[relations]]
type = "affects"
target = "milestone:v1-0"

[docgraph_generated]
schema_version = 1

[[docgraph_generated.backlinks]]
source = "milestone:v1-0#s-XHXCWTTW9K"
target = "docs/issues/expose-complete-ontology-dump.md"

+++
<a id="s-TK7035WX02"></a>
# Expose a complete ontology dump

`docgraph describe` lists the configured vocabulary, but inspecting complete
definitions currently requires a separate invocation for every entity type,
relation, workflow, query, and repository command. That makes first-time
orientation and agent context assembly unnecessarily procedural.

Add a single structured describe operation that emits the complete configured
repository model, including property schemas, relation endpoints and inverses,
workflow states and transitions, named-query signatures, repository commands,
and project-level settings. Human-readable output should remain scannable, while
JSON output should be stable enough for agents and other tooling to consume.
