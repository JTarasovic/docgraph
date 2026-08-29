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

<a id="s-7NCE7Y7K4F"></a>
## Implementation

The portable bundle now has a versioned `skill.toml` contract tied to the exact
CLI version. The CLI embeds every managed skill file, and `instructions check`
reports missing, modified, or incompatible bundles. `instructions sync
--dry-run` previews repairs; applying sync replaces only managed files and
preserves repository-local extensions.

Release archives carry the same bundle, packaging rejects a mismatched skill/CLI
version, and the release smoke test installs and verifies the skill in a clean
fixture. Regression coverage exercises every status and repair boundary. The
issue remains open until repository initialization is implemented and proves it
uses this embedded installation contract.
