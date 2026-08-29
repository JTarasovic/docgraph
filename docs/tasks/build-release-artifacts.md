+++

id = "task:build-release-artifacts"
type = "task"
state = "backlog"

[properties]
title = "Build release artifacts"

[[relations]]
type = "part_of"
target = "plan:ship-first-release"

[[relations]]
type = "depends_on"
target = "task:define-release-contract"

[[relations]]
type = "implements"
target = "plan:ship-first-release#s-BB402CJXXE"

[docgraph_generated]
schema_version = 1

[[docgraph_generated.incoming]]
source = "task:automate-tagged-releases"
predicate = "depends_on"
target = "task:build-release-artifacts"

[[docgraph_generated.inverses]]
source = "task:build-release-artifacts"
type = "required_by"
target = "task:automate-tagged-releases"

+++
<a id="s-5NYP2VCD7A"></a>
# Build release artifacts

Build versioned archives and checksums for every supported target. Ensure the CLI can locate its required logic runtime from the installed layout and reports a useful error when that runtime is unavailable.
