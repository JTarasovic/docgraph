+++

id = "task:support-section-path-endpoints"
type = "task"
state = "done"

[properties]
title = "Support section endpoints in graph paths"

[[relations]]
type = "part_of"
target = "plan:complete-v0"

[[relations]]
type = "implements"
target = "reference:config-grammar#s-TW0V0THMJD"

[[relations]]
type = "implements"
target = "plan:complete-v0#s-87BFAA4CNY"

[docgraph_generated]
schema_version = 1

[[docgraph_generated.incoming]]
source = "task:audit-v0-success"
predicate = "depends_on"
target = "task:support-section-path-endpoints"

[[docgraph_generated.inverses]]
source = "task:support-section-path-endpoints"
type = "required_by"
target = "task:audit-v0-success"

[[docgraph_generated.backlinks]]
source = "plan:complete-v0#s-87BFAA4CNY"
target = "docs/tasks/support-section-path-endpoints.md"

+++
<a id="s-195M6YNE0Y"></a>
# Support section endpoints in graph paths

Resolve `docgraph path` arguments as canonical graph references so explicit paths between entities and stable sections can be traversed from the CLI.
