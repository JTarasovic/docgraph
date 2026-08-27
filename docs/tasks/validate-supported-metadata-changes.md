+++

id = "task:validate-supported-metadata-changes"
type = "task"
state = "done"

[properties]
title = "Validate supported metadata changes"

[[relations]]
type = "part_of"
target = "plan:enforce-managed-metadata-boundary"

[[relations]]
type = "depends_on"
target = "task:model-semantic-corpus-changes"

[[relations]]
type = "implements"
target = "plan:enforce-managed-metadata-boundary#s-9AACGEHSA0"

[docgraph_generated]
schema_version = 1

[[docgraph_generated.incoming]]
source = "issue:managed-metadata-guardrails"
predicate = "affects"
target = "task:validate-supported-metadata-changes"

[[docgraph_generated.incoming]]
source = "task:enforce-managed-metadata-guardrail"
predicate = "depends_on"
target = "task:validate-supported-metadata-changes"

[[docgraph_generated.inverses]]
source = "task:validate-supported-metadata-changes"
type = "affected_by"
target = "issue:managed-metadata-guardrails"

[[docgraph_generated.inverses]]
source = "task:validate-supported-metadata-changes"
type = "required_by"
target = "task:enforce-managed-metadata-guardrail"

+++
<a id="s-655ZW0QJBG"></a>
# Validate supported metadata changes

Classify managed changes against the operations docgraph supports and reject everything else.
