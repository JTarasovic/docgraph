+++

id = "issue:multi-file-adoption-normalize-first"
type = "issue"
state = "open"

[properties]
title = "Multi-file adoption requires normalize first"

[[relations]]
type = "affects"
target = "plan:close-initial-design-gaps"

[[relations]]
type = "affects"
target = "reference:config-grammar#s-V5R4RB2AP1"

[docgraph_generated]
schema_version = 1

+++
<a id="s-3V3RP8C4NH"></a>
# Multi-file adoption requires a normalize-first workaround

Adopting several linked, previously unmanaged documents one at a time can fail because each adoption validates the prospective corpus while the other new files still lack stable anchors. Running `docgraph normalize` across the batch before adopting each file is safe, but the required two-step sequence is surprising and should either become a first-class batch operation or produce actionable guidance.

This surfaced while creating the [initial-design gap plan](../plans/close-initial-design-gaps.md) and its tasks.
