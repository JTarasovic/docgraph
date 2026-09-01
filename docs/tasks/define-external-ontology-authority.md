+++

id = "task:define-external-ontology-authority"
type = "task"
state = "backlog"

[properties]
title = "Define external ontology and authority"

[[relations]]
type = "part_of"
target = "plan:complete-external-provider-ontology"

[[relations]]
type = "implements"
target = "plan:complete-external-provider-ontology#s-P7C8TRJE3M"

[docgraph_generated]
schema_version = 1

[[docgraph_generated.incoming]]
source = "task:expand-github-external-entity-kinds"
predicate = "depends_on"
target = "task:define-external-ontology-authority"

[[docgraph_generated.incoming]]
source = "task:project-external-entities-into-ontology"
predicate = "depends_on"
target = "task:define-external-ontology-authority"

[[docgraph_generated.inverses]]
source = "task:define-external-ontology-authority"
type = "required_by"
target = "task:expand-github-external-entity-kinds"

[[docgraph_generated.inverses]]
source = "task:define-external-ontology-authority"
type = "required_by"
target = "task:project-external-entities-into-ontology"

+++
<a id="s-Q9XEQEW9TC"></a>
# Define external ontology and authority

Turn [#13](https://github.com/JTarasovic/docgraph/issues/13) into an executable
provider-neutral contract. Define how a configured provider kind maps to a repository
entity type, how provider fields map to typed properties, and how provider states map
to workflow states without making cached bytes canonical repository content.

Specify identity, freshness, deletion, inaccessible records, unsupported fields,
provider relationships, relation endpoint validation, and precedence when remote and
repository-authored facts disagree. Define separate read, search, relation, transition,
and property-mutation capabilities and the user confirmation or preview boundary for
remote writes.

<a id="s-4DGJCW2WF3"></a>
## Acceptance

- Product references contain provider-neutral mapping and authority grammar.
- External type, property, state, and relation projections are explicit and validated.
- Workflow participation distinguishes observed remote state from mutation authority.
- Fresh, stale, missing, deleted, and inaccessible records have deterministic semantics.
- GitHub issues, pull requests, milestones, projects, and project items are each
  dispositioned as supported, deferred, or intentionally out of scope with rationale.
