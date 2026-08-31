+++

id = "task:add-github-external-entity-source"
type = "task"
state = "backlog"

[properties]
title = "Add the GitHub external entity source"

[[relations]]
type = "part_of"
target = "plan:deliver-external-entity-sources"

[[relations]]
type = "implements"
target = "plan:deliver-external-entity-sources#s-0WCAQFSYKB"

[[relations]]
type = "depends_on"
target = "task:define-external-entity-source-contract"

[[relations]]
type = "depends_on"
target = "task:persist-derived-external-entities"

[docgraph_generated]
schema_version = 1

[[docgraph_generated.incoming]]
source = "task:integrate-external-entities-with-retrieval"
predicate = "depends_on"
target = "task:add-github-external-entity-source"

[[docgraph_generated.incoming]]
source = "task:map-external-issues-into-project-work"
predicate = "depends_on"
target = "task:add-github-external-entity-source"

[[docgraph_generated.inverses]]
source = "task:add-github-external-entity-source"
type = "required_by"
target = "task:integrate-external-entities-with-retrieval"

[[docgraph_generated.inverses]]
source = "task:add-github-external-entity-source"
type = "required_by"
target = "task:map-external-issues-into-project-work"

+++
<a id="s-R9DK5X83PC"></a>
# Add the GitHub external entity source

Implement the first read-capable external entity source for canonical
`github:issue:<host>/<repository>:<number>` identities. Resolve configured GitHub and
GitHub Enterprise hosts, fetch issue records through the supported authenticated or
public path, and normalize them into the provider-neutral contract.

Map title, body, state, author, timestamps, labels, assignees, URL, and provider
attributes without leaking GitHub response types into graph or retrieval APIs.
Distinguish pull requests encountered through issue-shaped APIs rather than silently
misclassifying them. Use bounded requests and preserve actionable rate-limit,
authorization, not-found, timeout, and response-validation failures for fallback logic.

This task is read-only. It does not comment on, edit, label, assign, close, or reopen
remote records, and it must not make the GitHub CLI a required runtime dependency
unless the source contract explicitly selects a command-backed integration.

<a id="s-0AGRHYEAKH"></a>
## Acceptance

- Public and authenticated reads follow one documented configuration path.
- GitHub.com and configured enterprise hosts retain distinct identities.
- Conditional refresh avoids unnecessary reads when supported.
- Provider failures degrade through the cache and identity rules.
- Tests use a controlled HTTP or command boundary and never require live GitHub.
