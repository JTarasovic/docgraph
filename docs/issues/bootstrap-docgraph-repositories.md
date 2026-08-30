+++

id = "issue:bootstrap-docgraph-repositories"
type = "issue"
state = "resolved"

[properties]
title = "Bootstrap docgraph repositories"

[[relations]]
type = "affects"
target = "milestone:v1-0"

[docgraph_generated]
schema_version = 1

[[docgraph_generated.backlinks]]
source = "milestone:v1-0#s-XHXCWTTW9K"
target = "docs/issues/bootstrap-docgraph-repositories.md"

+++
<a id="s-DX6TTD2CND"></a>
# Bootstrap docgraph repositories

Installing the docgraph binary does not currently make an existing repository
ready to use. The portable agent skill must be copied from another checkout,
`.docgraph` configuration must be assembled manually, and agent instructions must
then be synchronized as a separate operation.

Provide an idempotent initialization command that can bootstrap an existing
repository without overwriting authored files. It should install the
version-compatible portable skill, create or adopt a minimal project
configuration, synchronize configured agent-instruction targets, support a dry
run, and refuse ambiguous or conflicting existing state with actionable
diagnostics.

<a id="s-9EAV05SBS2"></a>
## Resolution

Implemented `docgraph init` with fresh-repository and existing-configuration paths,
complete dry-run output, embedded skill installation, authored-guidance preservation,
document-root creation, idempotence, and pre-write conflict checks. End-to-end tests
exercise successful initialization, validation, repeated application, configuration
adoption, and refusal boundaries.
