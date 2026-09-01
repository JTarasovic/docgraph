+++

id = "task:expand-github-external-entity-kinds"
type = "task"
state = "backlog"

[properties]
title = "Expand GitHub external entity kinds"

[[relations]]
type = "part_of"
target = "plan:complete-external-provider-ontology"

[[relations]]
type = "implements"
target = "plan:complete-external-provider-ontology#s-2JQ6E0559W"

[[relations]]
type = "depends_on"
target = "task:define-external-ontology-authority"

[docgraph_generated]
schema_version = 1

[[docgraph_generated.incoming]]
source = "task:dogfood-external-ontology"
predicate = "depends_on"
target = "task:expand-github-external-entity-kinds"

[[docgraph_generated.incoming]]
source = "task:enable-external-provider-mutations"
predicate = "depends_on"
target = "task:expand-github-external-entity-kinds"

[[docgraph_generated.inverses]]
source = "task:expand-github-external-entity-kinds"
type = "required_by"
target = "task:dogfood-external-ontology"

[[docgraph_generated.inverses]]
source = "task:expand-github-external-entity-kinds"
type = "required_by"
target = "task:enable-external-provider-mutations"

+++
<a id="s-SSBH56TSYF"></a>
# Expand GitHub external entity kinds

Expand the built-in GitHub adapter for the kinds accepted by the external-authority
contract in [#13](https://github.com/JTarasovic/docgraph/issues/13). Normalize issues,
pull requests, milestones, projects, and project items into provider-neutral records
and relationships rather than leaking GitHub response shapes into the graph core.

Account for REST and GraphQL boundaries, pagination, draft and closed states, deleted
or inaccessible resources, conditional refresh, rate limits, GitHub Enterprise, and
repositories without Projects enabled. Automated coverage remains network-independent.

<a id="s-YH8TAG14ND"></a>
## Acceptance

- Every accepted GitHub kind has canonical identity, read, and bounded search/list
  behavior with deterministic pagination.
- Cross-kind relationships such as issue-to-milestone and project-to-item are normalized.
- Pull requests are not conflated with issues even when an API returns issue-shaped data.
- Capability and failure reporting is accurate for GitHub.com and configured Enterprise.
- Fixture conformance covers fresh, unchanged, stale, deleted, rate-limited, malformed,
  and inaccessible responses for each supported API family.
