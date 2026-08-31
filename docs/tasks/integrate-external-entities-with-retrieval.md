+++

id = "task:integrate-external-entities-with-retrieval"
type = "task"
state = "backlog"

[properties]
title = "Integrate external entities with retrieval"

[[relations]]
type = "part_of"
target = "plan:deliver-external-entity-sources"

[[relations]]
type = "implements"
target = "plan:deliver-external-entity-sources#s-HR8FQ9NYF4"

[[relations]]
type = "depends_on"
target = "task:persist-derived-external-entities"

[[relations]]
type = "depends_on"
target = "task:add-github-external-entity-source"

[docgraph_generated]
schema_version = 1

[[docgraph_generated.incoming]]
source = "task:dogfood-github-external-issues"
predicate = "depends_on"
target = "task:integrate-external-entities-with-retrieval"

[[docgraph_generated.inverses]]
source = "task:integrate-external-entities-with-retrieval"
type = "required_by"
target = "task:dogfood-github-external-issues"

+++
<a id="s-HYDV4AZQ4P"></a>
# Integrate external entities with retrieval

Teach ordinary retrieval to enrich canonical external graph nodes from the derived
store and configured source. `get` must return the provider-neutral record;
`context` must include enriched external neighbors; full-text search must index
external titles and bodies; vector retrieval must include the same content when an
embedding provider is configured.

Human and JSON output must distinguish canonical identity from derived metadata and
report provider, URL, fetched time, freshness, and fallback status consistently.
Remote content must never be presented as repository-authored facts. Refresh policy
must avoid turning graph-only reads into unbounded network fan-out.

<a id="s-QK1NV6J9YA"></a>
## Acceptance

- The same external identity is stable across identity-only, cached, and live reads.
- `get`, `context`, search, and vectors expose consistent record identity and provenance.
- Stale and unavailable states are visible in structured and human-readable output.
- Search refreshes incrementally and does not duplicate records after cache updates.
- Repository-only retrieval remains deterministic and usable offline.
