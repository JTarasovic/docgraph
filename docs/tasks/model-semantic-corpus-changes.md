+++

id = "task:model-semantic-corpus-changes"
type = "task"
state = "done"

[properties]
title = "Model semantic corpus changes"

[[relations]]
type = "part_of"
target = "plan:enforce-managed-metadata-boundary"

[[relations]]
type = "implements"
target = "plan:enforce-managed-metadata-boundary#s-STJG1E8M3Z"

[docgraph_generated]
schema_version = 1

[[docgraph_generated.incoming]]
source = "issue:managed-metadata-guardrails"
predicate = "affects"
target = "task:model-semantic-corpus-changes"

[[docgraph_generated.incoming]]
source = "task:validate-supported-metadata-changes"
predicate = "depends_on"
target = "task:model-semantic-corpus-changes"

[[docgraph_generated.inverses]]
source = "task:model-semantic-corpus-changes"
type = "affected_by"
target = "issue:managed-metadata-guardrails"

[[docgraph_generated.inverses]]
source = "task:model-semantic-corpus-changes"
type = "required_by"
target = "task:validate-supported-metadata-changes"

+++
<a id="s-H5WWBJEZGM"></a>
# Model semantic corpus changes

Build a deterministic comparison between a Git base corpus and the current candidate corpus.
