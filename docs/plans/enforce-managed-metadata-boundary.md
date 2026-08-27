+++

id = "plan:enforce-managed-metadata-boundary"
type = "plan"
state = "completed"

[properties]
title = "Enforce the managed metadata boundary"

[docgraph_generated]
schema_version = 1

[[docgraph_generated.incoming]]
source = "issue:managed-metadata-guardrails"
predicate = "affects"
target = "plan:enforce-managed-metadata-boundary"

[[docgraph_generated.incoming]]
source = "task:enforce-managed-metadata-guardrail"
predicate = "implements"
target = "plan:enforce-managed-metadata-boundary#s-SDB69MNV8J"

[[docgraph_generated.incoming]]
source = "task:enforce-managed-metadata-guardrail"
predicate = "part_of"
target = "plan:enforce-managed-metadata-boundary"

[[docgraph_generated.incoming]]
source = "task:model-semantic-corpus-changes"
predicate = "implements"
target = "plan:enforce-managed-metadata-boundary#s-STJG1E8M3Z"

[[docgraph_generated.incoming]]
source = "task:model-semantic-corpus-changes"
predicate = "part_of"
target = "plan:enforce-managed-metadata-boundary"

[[docgraph_generated.incoming]]
source = "task:validate-supported-metadata-changes"
predicate = "implements"
target = "plan:enforce-managed-metadata-boundary#s-9AACGEHSA0"

[[docgraph_generated.incoming]]
source = "task:validate-supported-metadata-changes"
predicate = "part_of"
target = "plan:enforce-managed-metadata-boundary"

[[docgraph_generated.inverses]]
source = "plan:enforce-managed-metadata-boundary"
type = "affected_by"
target = "issue:managed-metadata-guardrails"

[[docgraph_generated.inverses]]
source = "plan:enforce-managed-metadata-boundary"
type = "contains"
target = "task:enforce-managed-metadata-guardrail"

[[docgraph_generated.inverses]]
source = "plan:enforce-managed-metadata-boundary"
type = "contains"
target = "task:model-semantic-corpus-changes"

[[docgraph_generated.inverses]]
source = "plan:enforce-managed-metadata-boundary"
type = "contains"
target = "task:validate-supported-metadata-changes"

[[docgraph_generated.inverses]]
source = "plan:enforce-managed-metadata-boundary#s-9AACGEHSA0"
type = "implemented_by"
target = "task:validate-supported-metadata-changes"

[[docgraph_generated.inverses]]
source = "plan:enforce-managed-metadata-boundary#s-SDB69MNV8J"
type = "implemented_by"
target = "task:enforce-managed-metadata-guardrail"

[[docgraph_generated.inverses]]
source = "plan:enforce-managed-metadata-boundary#s-STJG1E8M3Z"
type = "implemented_by"
target = "task:model-semantic-corpus-changes"

+++
<a id="s-7QZCYH0M29"></a>
# Enforce the managed metadata boundary

<a id="s-EM7Y2X2DGN"></a>
## Objective

Reject corpus changes that bypass docgraph's supported semantic mutations while leaving prose freely editable.

<a id="s-85GK37GT8Y"></a>
## Steps

<a id="s-STJG1E8M3Z"></a>
### Model semantic corpus changes

Compare a repository base with the candidate corpus and classify changes to identities, workflow state, properties, relations, generated projections, and stable sections.

<a id="s-9AACGEHSA0"></a>
### Validate changes against supported mutations

Accept only changes equivalent to docgraph operations and report unsupported managed metadata edits with actionable diagnostics.

<a id="s-SDB69MNV8J"></a>
### Enforce the guardrail in this repository

Run change validation in local checks and CI, with conformance coverage for safe prose edits and unsafe managed edits.

<a id="s-T41DN23FTR"></a>
## Completion

The dogfood repository rejects unsupported managed metadata changes without blocking ordinary prose edits.
