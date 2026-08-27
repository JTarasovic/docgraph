+++

id = "task:implement-repository-commands"
type = "task"
state = "done"

[properties]
title = "Implement repository-defined commands"

[[relations]]
type = "part_of"
target = "plan:project-aware-commands"

[[relations]]
type = "implements"
target = "plan:project-aware-commands#s-4703EK4457"

[docgraph_generated]
schema_version = 1

[[docgraph_generated.backlinks]]
source = "plan:project-aware-commands#s-4703EK4457"
target = "docs/tasks/implement-repository-commands.md"

+++
<a id="s-7794ZS0VDF"></a>
# Implement repository-defined commands

Load and validate `commands.toml`, dispatch configured query and mutation operations through the generic engine, and expose command names, descriptions, and inputs through project-aware help.
