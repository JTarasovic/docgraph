+++

id = "issue:publish-validation-action"
type = "issue"
state = "resolved"

[properties]
title = "Publish a docgraph validation action"

[[relations]]
type = "affects"
target = "milestone:v1-0"

[docgraph_generated]
schema_version = 1

+++
<a id="s-JMW4ZDJMTB"></a>
# Publish a docgraph validation action

Provide a small GitHub Action that lets consuming repositories install a released docgraph version and validate their corpus in CI. This is deliberately post-release: it depends on a stable versioned artifact and installation contract from `plan:ship-first-release`, but should not require consumers to understand this repository's build or Soufflé setup.

<a id="s-8N59APGSBJ"></a>
## Resolution

Published a root composite action that installs an exact Windows or Linux x86-64
release, downloads private assets through the GitHub API when a token is
available, verifies the adjacent SHA-256 file and packaged version/runtime
layout, exports the installed executable, and runs corpus validation.

The action accepts working-directory and change-base inputs, requires no Rust,
mise, source checkout of docgraph, or separately installed logic runtime, and is
documented with full-commit-SHA pinning guidance. The CI workflow smoke-tests the
local action on both supported runner operating systems. A clean Windows smoke
test downloaded the private `v0.1.0` release, verified it, and validated the
synthetic corpus; the released binary also validates the current repository.
