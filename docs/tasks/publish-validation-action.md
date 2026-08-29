+++

id = "task:publish-validation-action"
type = "task"
state = "done"

[properties]
title = "Publish a docgraph validation action"

[[relations]]
type = "part_of"
target = "plan:complete-v1-readiness"

[[relations]]
type = "implements"
target = "reference:validation-action"

[docgraph_generated]
schema_version = 1

+++
<a id="s-HR6SRXQA4V"></a>
# Publish a docgraph validation action

Provide and document a small versioned GitHub Action that installs a released
docgraph archive, verifies it, and validates a consuming repository's corpus.

<a id="s-N5RMSP556Y"></a>
## Result

Implemented the root composite action, checksum-verifying PowerShell installer,
private-release token support, consumer contract, README example, and Linux and
Windows CI smoke steps. The clean released-binary smoke test and all 102 repository
tests pass.
