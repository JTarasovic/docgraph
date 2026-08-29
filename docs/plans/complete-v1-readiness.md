+++

id = "plan:complete-v1-readiness"
type = "plan"
state = "active"

[properties]
title = "Complete v1 readiness"

[[relations]]
type = "required_for"
target = "milestone:v1-0"

[docgraph_generated]
schema_version = 1

[[docgraph_generated.incoming]]
source = "task:automate-dependency-management"
predicate = "part_of"
target = "plan:complete-v1-readiness"

[[docgraph_generated.incoming]]
source = "task:bootstrap-docgraph-repositories"
predicate = "part_of"
target = "plan:complete-v1-readiness"

[[docgraph_generated.incoming]]
source = "task:expose-complete-ontology-dump"
predicate = "part_of"
target = "plan:complete-v1-readiness"

[[docgraph_generated.incoming]]
source = "task:publish-validation-action"
predicate = "part_of"
target = "plan:complete-v1-readiness"

[[docgraph_generated.incoming]]
source = "task:version-portable-agent-skill"
predicate = "part_of"
target = "plan:complete-v1-readiness"

[[docgraph_generated.inverses]]
source = "plan:complete-v1-readiness"
type = "contains"
target = "task:automate-dependency-management"

[[docgraph_generated.inverses]]
source = "plan:complete-v1-readiness"
type = "contains"
target = "task:bootstrap-docgraph-repositories"

[[docgraph_generated.inverses]]
source = "plan:complete-v1-readiness"
type = "contains"
target = "task:expose-complete-ontology-dump"

[[docgraph_generated.inverses]]
source = "plan:complete-v1-readiness"
type = "contains"
target = "task:publish-validation-action"

[[docgraph_generated.inverses]]
source = "plan:complete-v1-readiness"
type = "contains"
target = "task:version-portable-agent-skill"

+++
<a id="s-Y7D9FBB9AF"></a>
# Complete v1 readiness

Resolve every open issue that affects `milestone:v1-0`, starting with the
smallest independent changes and leaving repository initialization until its
portable-skill dependency is complete.

Execution order:

1. Configure grouped dependency updates across crates, actions, mise, and the
   pinned logic-runtime inputs.
2. Add one complete, structured ontology-description operation.
3. Publish a reusable validation action for consuming repositories.
4. Version and verify the portable agent skill.
5. Add idempotent repository initialization using that versioned skill contract.

Each task closes its corresponding issue only after reference documentation,
regression coverage, `docgraph validate`, and the repository check suite pass.
