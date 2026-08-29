+++

id = "issue:publish-validation-action"
type = "issue"
state = "open"

[properties]
title = "Publish a docgraph validation action"

[[relations]]
type = "affects"
target = "milestone:v1-0"

[docgraph_generated]
schema_version = 1

[[docgraph_generated.backlinks]]
source = "milestone:v1-0#s-XHXCWTTW9K"
target = "docs/issues/publish-validation-action.md"

+++
<a id="s-JMW4ZDJMTB"></a>
# Publish a docgraph validation action

Provide a small GitHub Action that lets consuming repositories install a released docgraph version and validate their corpus in CI. This is deliberately post-release: it depends on a stable versioned artifact and installation contract from `plan:ship-first-release`, but should not require consumers to understand this repository's build or Soufflé setup.
