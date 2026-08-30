+++

id = "task:bootstrap-docgraph-repositories"
type = "task"
state = "done"

[properties]
title = "Bootstrap docgraph repositories"

[[relations]]
type = "part_of"
target = "plan:complete-v1-readiness"

[[relations]]
type = "depends_on"
target = "task:version-portable-agent-skill"

[[relations]]
type = "implements"
target = "reference:design"

[[relations]]
type = "implements"
target = "reference:config-grammar"

[[relations]]
type = "implements"
target = "reference:scenarios"

[docgraph_generated]
schema_version = 1

+++
<a id="s-WMVZVXF019"></a>
# Bootstrap docgraph repositories

Add an idempotent, dry-runnable initialization command that creates or adopts a
minimal repository model, installs the compatible portable skill, and
synchronizes configured agent-instruction targets without overwriting authored
content.

<a id="s-AQP3S2CTMM"></a>
## Result

`docgraph init` now previews and bootstraps fresh Git repositories, adopts valid
existing configuration byte-for-byte, installs the CLI-compatible portable skill,
synchronizes configured guidance while preserving authored prose, and converges
idempotently. Conflicting options, unsafe paths, malformed guidance, invalid
configuration, and ambiguous partial `.docgraph` state are refused before writes.
