+++

id = "task:validate-first-release"
type = "task"
state = "backlog"

[properties]
title = "Validate the first release"

[[relations]]
type = "part_of"
target = "plan:ship-first-release"

[[relations]]
type = "depends_on"
target = "task:automate-tagged-releases"

[[relations]]
type = "implements"
target = "plan:ship-first-release#s-VX0QVYTCY6"

[docgraph_generated]
schema_version = 1

+++
<a id="s-4MSDJZ9GGX"></a>
# Validate the first release

Exercise the packaged CLI from clean supported environments without relying on the repository toolchain or build tree. Cover installation, help, adoption, validation, search, graph retrieval, configured logic, and one safe mutation before declaring the release ready.
