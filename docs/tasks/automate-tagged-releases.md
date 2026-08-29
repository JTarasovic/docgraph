+++

id = "task:automate-tagged-releases"
type = "task"
state = "backlog"

[properties]
title = "Automate tagged releases"

[[relations]]
type = "part_of"
target = "plan:ship-first-release"

[[relations]]
type = "depends_on"
target = "task:document-installation-and-quickstart"

[[relations]]
type = "depends_on"
target = "task:build-release-artifacts"

[[relations]]
type = "implements"
target = "plan:ship-first-release#s-MDRVBRAK28"

[docgraph_generated]
schema_version = 1

[[docgraph_generated.incoming]]
source = "task:validate-first-release"
predicate = "depends_on"
target = "task:automate-tagged-releases"

[[docgraph_generated.inverses]]
source = "task:automate-tagged-releases"
type = "required_by"
target = "task:validate-first-release"

+++
<a id="s-FB3725AAYS"></a>
# Automate tagged releases

Add a tag-triggered GitHub Actions workflow that verifies the intended version, builds the supported artifacts, runs artifact-level smoke tests, and publishes one GitHub release with checksums and concise release notes. Give every job an explicit, conservative timeout, cancel superseded runs, and consume the pinned logic-runtime artifacts rather than rebuilding Soufflé.
