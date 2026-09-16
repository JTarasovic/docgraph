+++

id = "task:add-quickstart-section"
type = "task"
state = "done"

[properties]
title = "Add a short quickstart under installation"

[[relations]]
type = "part_of"
target = "plan:overhaul-onboarding-readme"

[[relations]]
type = "depends_on"
target = "task:restructure-installation-guidance"

[docgraph_generated]
schema_version = 1

+++
<a id="s-TEMBAZ1BVD"></a>
# Add a short quickstart under installation

Place a short quickstart directly under installation covering the minimal path to a
working repo, not a tour: `docgraph init`, then `docgraph describe` and
`docgraph validate`, then one create example.

Move `adopt`, external-entity sources, and the deeper narrative to the reference
docs and link out to them rather than expanding the quickstart.

<a id="s-WNX2403N3W"></a>
## Result

The Quickstart sits directly under Installation and covers only the minimal path:
`init`, `describe`/`validate`, and one `document create` example. Adopt, external-
entity sources, and the deeper narrative are removed and replaced with a link to
the config reference.
