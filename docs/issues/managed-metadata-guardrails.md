+++

id = "issue:managed-metadata-guardrails"
type = "issue"
state = "resolved"

[properties]
title = "Managed metadata guardrails are easy to bypass"

[[relations]]
type = "affects"
target = "plan:close-initial-design-gaps"

[[relations]]
type = "affects"
target = "decision:cli-managed-metadata"

[[relations]]
type = "affects"
target = "plan:enforce-managed-metadata-boundary"

[[relations]]
type = "affects"
target = "task:model-semantic-corpus-changes"

[[relations]]
type = "affects"
target = "task:validate-supported-metadata-changes"

[[relations]]
type = "affects"
target = "task:enforce-managed-metadata-guardrail"

[docgraph_generated]
schema_version = 1

+++
<a id="s-44777ZK3XR"></a>
# Managed metadata guardrails are easy to bypass

While editing reference prose, an agent can accidentally hand-author a stable anchor or managed frontmatter before remembering that docgraph owns those fields. Validation can confirm consistency after the fact, but the editing boundary is convention rather than an active guardrail.

This surfaced during the [initial-design gap audit](../plans/close-initial-design-gaps.md).

Resolved by semantic change validation against a Git base. The repository now allows
prose and supported mutation outcomes while rejecting unsupported managed changes in
local checks and CI.
