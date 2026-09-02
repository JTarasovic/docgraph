+++

id = "task:dogfood-external-ontology"
type = "task"
state = "backlog"

[properties]
title = "Dogfood external ontology participation"

[[relations]]
type = "part_of"
target = "plan:complete-external-provider-ontology"

[[relations]]
type = "implements"
target = "plan:complete-external-provider-ontology#s-Q26ZD9F7G3"

[[relations]]
type = "depends_on"
target = "task:project-external-entities-into-ontology"

[[relations]]
type = "depends_on"
target = "task:expand-github-external-entity-kinds"

[[relations]]
type = "depends_on"
target = "task:enable-external-provider-mutations"

[docgraph_generated]
schema_version = 1

+++
<a id="s-E62ZKQH075"></a>
# Dogfood external ontology participation

Exercise [#13](https://github.com/JTarasovic/docgraph/issues/13) against this
repository after the provider-neutral model and GitHub adapter are complete. Configure
explicit mappings for the remote work kinds actually in use and remove any temporary
logic that treats every open GitHub issue as an untyped special case.

Verify the agent workflow end to end: discover remote work, inspect its projected type
and state, relate it to canonical plans or tasks, query it through repository policy,
perform one safely previewed supported mutation, refresh, and continue from a warm
cache offline.

Include the current #12 case: a direct lookup should reveal that
`task:attest-release-artifacts` addresses the GitHub issue, rather than returning the
local task and remote issue as unrelated work candidates or requiring a prose grep.

<a id="s-6J89RYBQJF"></a>
## Acceptance

- Real GitHub issues and milestones or project items participate in configured ontology
  and workflow queries without local mirror documents.
- Typed cross-boundary relations survive refresh and remain inspectable offline.
- At least one supported remote mutation is previewed, applied in a controlled dogfood
  case, refreshed, and verified.
- Agent guidance covers authority, freshness, authentication, and failure recovery.
- The special-case `external_issue` project logic is removed or retained only with a
  documented compatibility reason.
