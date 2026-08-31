+++

id = "task:persist-derived-external-entities"
type = "task"
state = "backlog"

[properties]
title = "Persist derived external entity records"

[[relations]]
type = "part_of"
target = "plan:deliver-external-entity-sources"

[[relations]]
type = "implements"
target = "plan:deliver-external-entity-sources#s-PX3AX3JRAW"

[[relations]]
type = "depends_on"
target = "task:define-external-entity-source-contract"

[docgraph_generated]
schema_version = 1

[[docgraph_generated.incoming]]
source = "task:add-github-external-entity-source"
predicate = "depends_on"
target = "task:persist-derived-external-entities"

[[docgraph_generated.incoming]]
source = "task:integrate-external-entities-with-retrieval"
predicate = "depends_on"
target = "task:persist-derived-external-entities"

[[docgraph_generated.incoming]]
source = "task:map-external-issues-into-project-work"
predicate = "depends_on"
target = "task:persist-derived-external-entities"

[[docgraph_generated.inverses]]
source = "task:persist-derived-external-entities"
type = "required_by"
target = "task:add-github-external-entity-source"

[[docgraph_generated.inverses]]
source = "task:persist-derived-external-entities"
type = "required_by"
target = "task:integrate-external-entities-with-retrieval"

[[docgraph_generated.inverses]]
source = "task:persist-derived-external-entities"
type = "required_by"
target = "task:map-external-issues-into-project-work"

+++
<a id="s-1YD5E2MTAB"></a>
# Persist derived external entity records

Extend the per-worktree derived store with provider-neutral external records keyed by
canonical external identity and provider identity. Persist the normalized payload,
fetch time, freshness information, provider version, and any conditional-refresh
token needed by the source.

Keep external refresh state separate from canonical corpus fingerprints. Deleting or
rebuilding the derived store must lose no authored facts or relationships. Schema
changes need the same disposable rebuild behavior as the existing search index.

Reads use fresh cached data when allowed, label stale cached data explicitly when a
refresh cannot run, and fall back to the bare canonical identity when neither live nor
cached data is available. Authentication failures, rate limits, timeouts, unavailable
networks, missing records, and malformed provider responses must remain distinguishable.

<a id="s-EA1WW3VSCR"></a>
## Acceptance

- Cache keys prevent collisions across providers, hosts, and repositories.
- Fresh, stale, missing, and deleted records have deterministic behavior.
- Offline reads use labeled cached data or the canonical identity without failing the graph.
- Derived-store deletion and rebuild preserve all canonical semantics.
- Tests use deterministic clocks and fake sources rather than live services.
