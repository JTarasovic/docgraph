+++

id = "task:define-portable-agent-skill-contract"
type = "task"
state = "backlog"

[properties]
title = "Define the portable agent skill contract"

[[relations]]
type = "part_of"
target = "plan:make-agent-guidance-portable"

[[relations]]
type = "implements"
target = "plan:make-agent-guidance-portable#s-0CHDK124TR"

[docgraph_generated]
schema_version = 1

[[docgraph_generated.incoming]]
source = "task:generate-dynamic-agent-guidance"
predicate = "depends_on"
target = "task:define-portable-agent-skill-contract"

[[docgraph_generated.incoming]]
source = "task:support-configurable-skill-targets"
predicate = "depends_on"
target = "task:define-portable-agent-skill-contract"

[[docgraph_generated.inverses]]
source = "task:define-portable-agent-skill-contract"
type = "required_by"
target = "task:generate-dynamic-agent-guidance"

[[docgraph_generated.inverses]]
source = "task:define-portable-agent-skill-contract"
type = "required_by"
target = "task:support-configurable-skill-targets"

+++
<a id="s-65B9RX0AV2"></a>
# Define the portable agent skill contract

Define the product contract needed by
[#18](https://github.com/JTarasovic/docgraph/issues/18) before changing installation
behavior. Specify required `SKILL.md` discovery metadata, canonical payload ownership,
default target behavior, multiple explicit targets, and compatibility across agents
whose discovery directories differ.

Resolve how the portable bundle path mentioned by generated repository instructions
relates to configured install targets. Specify duplicate or overlapping targets,
symlinks, path escape, case sensitivity, target collisions, repository-owned files,
and migration from the fixed `skills/docgraph` default.

<a id="s-J3EZ9TQ85P"></a>
## Acceptance

- Product references require valid skill name and description metadata.
- Configuration supports zero or more explicit repository-relative skill targets and
  defines backward-compatible defaults.
- Canonical payload, installed copies, and generated instruction links have one clear
  ownership model.
- Check/sync behavior is specified per target, including conflicts and safe migration.
- The contract covers Claude, Codex, and a custom target without hardcoding them as the
  only possible agents.
