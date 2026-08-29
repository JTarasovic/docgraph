+++

id = "issue:version-portable-agent-skill"
type = "issue"
state = "open"

[properties]
title = "Version and verify the portable agent skill"

[[relations]]
type = "affects"
target = "milestone:v1-0"

[docgraph_generated]
schema_version = 1

[[docgraph_generated.backlinks]]
source = "milestone:v1-0#s-XHXCWTTW9K"
target = "docs/issues/version-portable-agent-skill.md"

+++
<a id="s-DC7HPKJNQH"></a>
# Version and verify the portable agent skill

The checked-in portable skill has no independently verifiable compatibility
marker, and `docgraph instructions check` verifies generated instruction blocks
without proving that the referenced skill exists or matches the installed CLI.
A repository can therefore present current instructions while silently carrying
missing or stale operational guidance.

Version the skill/config/CLI contract and make repository maintenance detect a
missing, modified, or incompatible portable skill. Initialization and upgrades
should install the skill version associated with the CLI, preserve safe
repository-local customization boundaries, and provide an explicit preview
before replacing managed skill content.
