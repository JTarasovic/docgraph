+++

id = "task:version-portable-agent-skill"
type = "task"
state = "in_progress"

[properties]
title = "Version and verify the portable agent skill"

[[relations]]
type = "part_of"
target = "plan:complete-v1-readiness"

[docgraph_generated]
schema_version = 1

[[docgraph_generated.incoming]]
source = "task:bootstrap-docgraph-repositories"
predicate = "depends_on"
target = "task:version-portable-agent-skill"

[[docgraph_generated.inverses]]
source = "task:version-portable-agent-skill"
type = "required_by"
target = "task:bootstrap-docgraph-repositories"

+++
<a id="s-RATX5Y6EFW"></a>
# Version and verify the portable agent skill

Define the CLI/config/skill compatibility marker, package the matching portable
skill, and make repository maintenance detect missing, modified, or incompatible
managed skill content with previewable repair behavior.
