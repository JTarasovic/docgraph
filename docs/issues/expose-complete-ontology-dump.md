+++

id = "issue:expose-complete-ontology-dump"
type = "issue"
state = "resolved"

[properties]
title = "Expose a complete ontology dump"

[[relations]]
type = "affects"
target = "milestone:v1-0"

[docgraph_generated]
schema_version = 1

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

<a id="s-X4TW79XPF9"></a>
## Resolution

Added `docgraph describe --all`. It emits one stable JSON object containing the
schema version; complete project, corpus, frontmatter, validation, reference,
embedding, and logic settings; all entity property schemas and allowed values;
all relation endpoints, inverses, acyclicity, and properties; all workflow states
and transitions; all query signatures; and all repository-command operations.

Bare `docgraph describe` remains the compact inventory, and scoped descriptions
remain backward-compatible. The same property-schema serializer now exposes
allowed values in scoped output as well. Synthetic CLI coverage verifies the
complete shape, readable output, and incompatible-argument rejection; the full
102-test repository suite passes.
