+++

id = "task:support-configurable-skill-targets"
type = "task"
state = "backlog"

[properties]
title = "Support configurable agent skill targets"

[[relations]]
type = "part_of"
target = "plan:make-agent-guidance-portable"

[[relations]]
type = "implements"
target = "plan:make-agent-guidance-portable#s-BAP0AYXWFP"

[[relations]]
type = "depends_on"
target = "task:define-portable-agent-skill-contract"

[[relations]]
type = "depends_on"
target = "task:generate-dynamic-agent-guidance"

[docgraph_generated]
schema_version = 1

+++
<a id="s-HJGMAX5RM8"></a>
# Support configurable agent skill targets

Implement the target portion of
[#18](https://github.com/JTarasovic/docgraph/issues/18). Extend
`[agent_instructions]` with the accepted skill-target grammar and make init,
`instructions sync`, and `instructions check` operate on every configured destination.

All writes use the safe mutation protocol. Reporting identifies each path and status;
a missing or drifted target must fail check even when another copy is current. Preserve
unrecognized repository files and refuse ambiguous overlaps rather than following a
path blindly.

<a id="s-77KDJYGJ1Q"></a>
## Acceptance

- One sync installs a discoverable bundle in multiple configured agent directories.
- Dry-run shows exact per-target patches and applying it is idempotent.
- Check reports current, missing, modified, incompatible, and conflicting targets.
- Broken symlinks, path escapes, overlapping targets, and concurrent changes have
  deterministic safe behavior.
- Existing repositories using the default path continue to work with a documented
  migration path.
