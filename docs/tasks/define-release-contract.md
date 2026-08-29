+++

id = "task:define-release-contract"
type = "task"
state = "ready"

[properties]
title = "Define the release contract"

[[relations]]
type = "part_of"
target = "plan:ship-first-release"

[[relations]]
type = "implements"
target = "plan:ship-first-release#s-KP3WHQXBZS"

[docgraph_generated]
schema_version = 1

[[docgraph_generated.incoming]]
source = "task:build-release-artifacts"
predicate = "depends_on"
target = "task:define-release-contract"

[[docgraph_generated.incoming]]
source = "task:document-installation-and-quickstart"
predicate = "depends_on"
target = "task:define-release-contract"

[[docgraph_generated.inverses]]
source = "task:define-release-contract"
type = "required_by"
target = "task:build-release-artifacts"

[[docgraph_generated.inverses]]
source = "task:define-release-contract"
type = "required_by"
target = "task:document-installation-and-quickstart"

+++
<a id="s-7QH5T048ZZ"></a>
# Define the release contract

Record the first release version, supported operating systems and architectures, Soufflé runtime delivery, archive layout, and the exact smoke-test boundary. Keep unsupported targets explicit rather than implying portability we have not tested.
