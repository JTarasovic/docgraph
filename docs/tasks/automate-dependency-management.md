+++

id = "task:automate-dependency-management"
type = "task"
state = "done"

[properties]
title = "Automate dependency management"

[[relations]]
type = "part_of"
target = "plan:complete-v1-readiness"

[[relations]]
type = "implements"
target = "reference:dependency-management"

[docgraph_generated]
schema_version = 1

+++
<a id="s-JRZS211H6Y"></a>
# Automate dependency management

Adopt Renovate as the single dependency-update path. Cover Rust crates and the
lockfile, digest-pinned GitHub Actions, mise tools, and the pinned native
logic-runtime inputs. Group compatible updates, keep major updates reviewable,
and rely on the normal pull-request checks rather than bypassing validation.

Done when the configuration is valid, every dependency surface is either
managed or explicitly documented as a coordinated manual update, and the
dependency-management issue records the resulting policy.

<a id="s-KVW196A29N"></a>
## Result

Implemented in `renovate.json` and `reference:dependency-management`. Renovate's
current validator accepts the configuration; the custom runtime managers match
three Souffle pins, two vcpkg pins, and two winflexbison pins. `mise run check`
passes with 102 tests after rebuilding the Windows runtime from the unchanged
pinned inputs.
