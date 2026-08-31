+++

id = "task:dogfood-github-external-issues"
type = "task"
state = "done"

[properties]
title = "Conform and dogfood GitHub external issues"

[[relations]]
type = "part_of"
target = "plan:deliver-external-entity-sources"

[[relations]]
type = "implements"
target = "plan:deliver-external-entity-sources#s-2PX8WMMF3A"

[[relations]]
type = "depends_on"
target = "task:integrate-external-entities-with-retrieval"

[[relations]]
type = "depends_on"
target = "task:map-external-issues-into-project-work"

[docgraph_generated]
schema_version = 1

+++
<a id="s-0VDW36D45Y"></a>
# Conform and dogfood GitHub external issues

Add provider-neutral and GitHub-specific conformance coverage for live-shaped records,
cache refresh, conditional reads, stale fallback, missing credentials, rate limits,
timeouts, malformed responses, deleted issues, and complete derived-store rebuilds.
All automated tests use deterministic fixtures or a controlled provider boundary and
must pass without network access.

Configure this repository to consume its GitHub issue source, then exercise the ten
currently open reports as the first real corpus. Confirm agents can discover them
through `docgraph next`, retrieve the relevant issue through `get`, find issue content
through search, and traverse between canonical work and external nodes where declared.

Document the operational path for authentication, refresh, offline work, and provider
failure. Retain GitHub as the sole canonical home of these reports; do not create local
issue documents merely to make the dogfood scenario pass.

<a id="s-7PYBHBJXGP"></a>
## Acceptance

- Conformance is network-independent and covers every documented fallback state.
- The repository's actual open GitHub issues appear through the explicit logic mapping.
- Retrieval labels remote provenance and freshness consistently.
- A warm cache remains useful offline and an empty cache preserves external identities.
- No remote mutation or duplicate local issue record is required.

<a id="s-Z11Z2HE7NP"></a>
## Result

Configured the repository's GitHub source and credential-helper path, then exercised
all ten open reports through `docgraph next`. Exact retrieval, context, full-text
search, and semantic fallback return derived records from GitHub without local issue
documents or remote mutation; automated provider tests remain network-independent.
