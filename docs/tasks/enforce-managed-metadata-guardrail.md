+++

id = "task:enforce-managed-metadata-guardrail"
type = "task"
state = "done"

[properties]
title = "Enforce the managed metadata guardrail"

[[relations]]
type = "part_of"
target = "plan:enforce-managed-metadata-boundary"

[[relations]]
type = "depends_on"
target = "task:validate-supported-metadata-changes"

[[relations]]
type = "implements"
target = "plan:enforce-managed-metadata-boundary#s-SDB69MNV8J"

[docgraph_generated]
schema_version = 1

[[docgraph_generated.incoming]]
source = "issue:managed-metadata-guardrails"
predicate = "affects"
target = "task:enforce-managed-metadata-guardrail"

[[docgraph_generated.inverses]]
source = "task:enforce-managed-metadata-guardrail"
type = "affected_by"
target = "issue:managed-metadata-guardrails"

+++
<a id="s-WJD301XDB0"></a>
# Enforce the managed metadata guardrail

Add the change check to repository validation and cover safe and unsafe edits end to end.
