+++

id = "decision:cli-managed-metadata"
type = "decision"
state = "accepted"

[properties]
title = "Route managed metadata changes through the CLI"

[docgraph_generated]
schema_version = 1

[[docgraph_generated.incoming]]
source = "task:expand-dogfood-ontology"
predicate = "implements"

[[docgraph_generated.inverses]]
type = "implemented_by"
target = "task:expand-dogfood-ontology"

+++
<a id="s-QKJWD5B1Z3"></a>
# Route managed metadata changes through the CLI

<a id="s-T6DA114R0V"></a>
## Context

Agents are good at editing prose but can miss inverse relationships, generated projections, validation rules, and coordinated metadata changes.

<a id="s-KS8VJ6VKG6"></a>
## Decision

Humans and agents may edit prose directly. Changes to docgraph-managed identity, properties, workflow state, and semantic relations go through docgraph commands. Generated frontmatter is a read-only projection maintained by docgraph.

<a id="s-VT672QKD7V"></a>
## Consequences

Semantic mutations can be previewed, validated, journaled, and applied atomically. The CLI must expose every supported managed mutation; missing commands cannot be worked around with direct frontmatter edits.
