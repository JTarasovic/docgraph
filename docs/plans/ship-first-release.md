+++

id = "plan:ship-first-release"
type = "plan"
state = "active"

[properties]
title = "Ship the first public release"

[[relations]]
type = "implements"
target = "decision:first-release-contract"

[docgraph_generated]
schema_version = 1

[[docgraph_generated.incoming]]
source = "task:automate-tagged-releases"
predicate = "implements"
target = "plan:ship-first-release#s-MDRVBRAK28"

[[docgraph_generated.incoming]]
source = "task:automate-tagged-releases"
predicate = "part_of"
target = "plan:ship-first-release"

[[docgraph_generated.incoming]]
source = "task:build-release-artifacts"
predicate = "implements"
target = "plan:ship-first-release#s-BB402CJXXE"

[[docgraph_generated.incoming]]
source = "task:build-release-artifacts"
predicate = "part_of"
target = "plan:ship-first-release"

[[docgraph_generated.incoming]]
source = "task:define-release-contract"
predicate = "implements"
target = "plan:ship-first-release#s-KP3WHQXBZS"

[[docgraph_generated.incoming]]
source = "task:define-release-contract"
predicate = "part_of"
target = "plan:ship-first-release"

[[docgraph_generated.incoming]]
source = "task:document-installation-and-quickstart"
predicate = "implements"
target = "plan:ship-first-release#s-290288YSKB"

[[docgraph_generated.incoming]]
source = "task:document-installation-and-quickstart"
predicate = "part_of"
target = "plan:ship-first-release"

[[docgraph_generated.incoming]]
source = "task:validate-first-release"
predicate = "implements"
target = "plan:ship-first-release#s-VX0QVYTCY6"

[[docgraph_generated.incoming]]
source = "task:validate-first-release"
predicate = "part_of"
target = "plan:ship-first-release"

[[docgraph_generated.inverses]]
source = "plan:ship-first-release"
type = "contains"
target = "task:automate-tagged-releases"

[[docgraph_generated.inverses]]
source = "plan:ship-first-release"
type = "contains"
target = "task:build-release-artifacts"

[[docgraph_generated.inverses]]
source = "plan:ship-first-release"
type = "contains"
target = "task:define-release-contract"

[[docgraph_generated.inverses]]
source = "plan:ship-first-release"
type = "contains"
target = "task:document-installation-and-quickstart"

[[docgraph_generated.inverses]]
source = "plan:ship-first-release"
type = "contains"
target = "task:validate-first-release"

[[docgraph_generated.inverses]]
source = "plan:ship-first-release#s-290288YSKB"
type = "implemented_by"
target = "task:document-installation-and-quickstart"

[[docgraph_generated.inverses]]
source = "plan:ship-first-release#s-BB402CJXXE"
type = "implemented_by"
target = "task:build-release-artifacts"

[[docgraph_generated.inverses]]
source = "plan:ship-first-release#s-KP3WHQXBZS"
type = "implemented_by"
target = "task:define-release-contract"

[[docgraph_generated.inverses]]
source = "plan:ship-first-release#s-MDRVBRAK28"
type = "implemented_by"
target = "task:automate-tagged-releases"

[[docgraph_generated.inverses]]
source = "plan:ship-first-release#s-VX0QVYTCY6"
type = "implemented_by"
target = "task:validate-first-release"

+++
<a id="s-3QCKV4TPPC"></a>
# Ship the first public release

<a id="s-5XHA6VH7DT"></a>
## Objective

Produce an installable, documented, and reproducible first release that a new user can exercise without a development checkout.

<a id="s-40387QKDC0"></a>
## Steps

<a id="s-KP3WHQXBZS"></a>
### Define the release contract

Choose the release version, supported targets, Soufflé delivery model, artifact contents, and acceptance boundary.

<a id="s-290288YSKB"></a>
### Document installation and the quickstart

Add the missing public-facing repository metadata and a short path from installation through adopting and querying a corpus.

<a id="s-BB402CJXXE"></a>
### Build release artifacts

Produce versioned archives and checksums for the supported targets, including everything required to execute configured logic.

<a id="s-MDRVBRAK28"></a>
### Automate tagged releases

Turn an intentional version tag into tested GitHub release artifacts without duplicating ordinary CI work.

<a id="s-VX0QVYTCY6"></a>
### Validate the first release

Install only the produced artifacts in clean environments and run the representative user workflow before publishing the release as ready.

<a id="s-SAZWJXKD9J"></a>
## Completion

A supported user can install docgraph from a release artifact, follow the quickstart, and complete the core workflow with the documented platform and runtime constraints.
