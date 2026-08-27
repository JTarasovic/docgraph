+++

id = "issue:workflow-adoption-initial-state"
type = "issue"
state = "open"

[properties]
title = "Workflow adoption cannot materialize initial state"

[[relations]]
type = "affects"
target = "task:expand-initial-design-conformance"

[[relations]]
type = "affects"
target = "reference:config-grammar#s-MTVEFXHGWD"

[docgraph_generated]
schema_version = 1

+++
<a id="s-TPRDTNF4X2"></a>
# Adding a workflow cannot materialize existing initial states

When an entity type gains a workflow, existing entities without an authored state become invalid. The CLI can follow a legal transition from the implicit initial state, but it cannot materialize that initial state directly without temporarily changing state or editing managed frontmatter.

This surfaced while expanding the synthetic conformance fixture.
