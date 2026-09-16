+++

id = "task:extract-contributor-docs"
type = "task"
state = "done"

[properties]
title = "Extract source-build and release content into CONTRIBUTING.md"

[[relations]]
type = "part_of"
target = "plan:overhaul-onboarding-readme"

[docgraph_generated]
schema_version = 1

+++
<a id="s-1ER2A5JQPH"></a>
# Extract source-build and release content into CONTRIBUTING.md

Remove the "Building from source" section from the README and create a root
`CONTRIBUTING.md` (none exists today) containing the `mise run check-local`
workflow, CI notes, and cross-platform runtime detail.

Move the release-runbook link out of the README into `CONTRIBUTING.md`, and leave a
single README link: "Contributing / building from source → CONTRIBUTING.md".

<a id="s-8MPFB7GD61"></a>
## Result

The README's "Building from source" section is removed. A new root
`CONTRIBUTING.md` holds the `mise run check-local` workflow, CI notes,
cross-platform runtime detail, the release-runbook link, commit/PR-title policy,
and docs-corpus guidance. The README's Contributing section links to it.
