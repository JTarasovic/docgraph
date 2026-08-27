+++

id = "task:generate-model-appendix"
type = "task"
state = "ready"

[properties]
title = "Generate the repository-model appendix"

[[relations]]
type = "part_of"
target = "plan:complete-v0"

[[relations]]
type = "implements"
target = "reference:design#s-DD5NS2HR0R"

[docgraph_generated]
schema_version = 1

[[docgraph_generated.incoming]]
source = "task:audit-v0-success"
predicate = "depends_on"

[[docgraph_generated.inverses]]
type = "required_by"
target = "task:audit-v0-success"

+++
<a id="s-395T841P69"></a>
# Generate the repository-model appendix

Implement the concise generated appendix required by the v0 contract so agents can inspect the repository's configured entity types, relations, workflows, and common operations.
