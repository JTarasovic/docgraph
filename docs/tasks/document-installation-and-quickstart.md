+++

id = "task:document-installation-and-quickstart"
type = "task"
state = "backlog"

[properties]
title = "Document installation and the quickstart"

[[relations]]
type = "part_of"
target = "plan:ship-first-release"

[[relations]]
type = "depends_on"
target = "task:define-release-contract"

[[relations]]
type = "implements"
target = "plan:ship-first-release#s-290288YSKB"

[docgraph_generated]
schema_version = 1

[[docgraph_generated.incoming]]
source = "task:automate-tagged-releases"
predicate = "depends_on"
target = "task:document-installation-and-quickstart"

[[docgraph_generated.inverses]]
source = "task:document-installation-and-quickstart"
type = "required_by"
target = "task:automate-tagged-releases"

+++
<a id="s-QK7J84VQYB"></a>
# Document installation and the quickstart

Add the repository README, license and package metadata needed for a public release. Document installation, project adoption, configuration, validation, retrieval, mutation, and the external runtime requirements in one short runnable path.
