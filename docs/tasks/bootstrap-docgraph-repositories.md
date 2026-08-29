+++

id = "task:bootstrap-docgraph-repositories"
type = "task"
state = "in_progress"

[properties]
title = "Bootstrap docgraph repositories"

[[relations]]
type = "part_of"
target = "plan:complete-v1-readiness"

[[relations]]
type = "depends_on"
target = "task:version-portable-agent-skill"

[docgraph_generated]
schema_version = 1

+++
<a id="s-WMVZVXF019"></a>
# Bootstrap docgraph repositories

Add an idempotent, dry-runnable initialization command that creates or adopts a
minimal repository model, installs the compatible portable skill, and
synchronizes configured agent-instruction targets without overwriting authored
content.
