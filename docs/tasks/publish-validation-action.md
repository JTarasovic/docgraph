+++

id = "task:publish-validation-action"
type = "task"
state = "backlog"

[properties]
title = "Publish a docgraph validation action"

[[relations]]
type = "part_of"
target = "plan:complete-v1-readiness"

[docgraph_generated]
schema_version = 1

+++
<a id="s-HR6SRXQA4V"></a>
# Publish a docgraph validation action

Provide and document a small versioned GitHub Action that installs a released
docgraph archive, verifies it, and validates a consuming repository's corpus.
