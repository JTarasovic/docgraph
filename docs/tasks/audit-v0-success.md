+++
id = "task:audit-v0-success"
type = "task"
state = "backlog"

[properties]
title = "Audit the v0 success criterion"

[[relations]]
type = "part_of"
target = "plan:complete-v0"

[[relations]]
type = "depends_on"
target = "task:generate-model-appendix"

[[relations]]
type = "implements"
target = "reference:config-grammar#s-GVPQBPMPBJ"

[[relations]]
type = "depends_on"
target = "task:support-section-path-endpoints"

[docgraph_generated]
schema_version = 1
+++
<a id="s-CV1YNY6QW7"></a>
# Audit the v0 success criterion

Exercise the complete configured ontology, workflow, inference, retrieval, impact-analysis, mutation, and agent-guidance loop against this repository and record any remaining implementation gaps.
