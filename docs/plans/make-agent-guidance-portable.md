+++

id = "plan:make-agent-guidance-portable"
type = "plan"
state = "proposed"

[properties]
title = "Make generated agent guidance portable"

[[relations]]
type = "implements"
target = "reference:design#s-64KP745XR0"

[[relations]]
type = "implements"
target = "reference:design#s-Q30QTKRZQ6"

[[relations]]
type = "implements"
target = "reference:design#s-Y4QFB1ZND8"

[[relations]]
type = "implements"
target = "reference:config-grammar#s-T1A2GRA1JJ"

[[relations]]
type = "implements"
target = "reference:config-grammar#s-XBB55PRA71"

[docgraph_generated]
schema_version = 1

[[docgraph_generated.incoming]]
source = "task:define-portable-agent-skill-contract"
predicate = "implements"
target = "plan:make-agent-guidance-portable#s-0CHDK124TR"

[[docgraph_generated.incoming]]
source = "task:define-portable-agent-skill-contract"
predicate = "part_of"
target = "plan:make-agent-guidance-portable"

[[docgraph_generated.incoming]]
source = "task:generate-dynamic-agent-guidance"
predicate = "implements"
target = "plan:make-agent-guidance-portable#s-GBKQP7BGTW"

[[docgraph_generated.incoming]]
source = "task:generate-dynamic-agent-guidance"
predicate = "part_of"
target = "plan:make-agent-guidance-portable"

[[docgraph_generated.incoming]]
source = "task:support-configurable-skill-targets"
predicate = "implements"
target = "plan:make-agent-guidance-portable#s-BAP0AYXWFP"

[[docgraph_generated.incoming]]
source = "task:support-configurable-skill-targets"
predicate = "part_of"
target = "plan:make-agent-guidance-portable"

[[docgraph_generated.inverses]]
source = "plan:make-agent-guidance-portable"
type = "contains"
target = "task:define-portable-agent-skill-contract"

[[docgraph_generated.inverses]]
source = "plan:make-agent-guidance-portable"
type = "contains"
target = "task:generate-dynamic-agent-guidance"

[[docgraph_generated.inverses]]
source = "plan:make-agent-guidance-portable"
type = "contains"
target = "task:support-configurable-skill-targets"

[[docgraph_generated.inverses]]
source = "plan:make-agent-guidance-portable#s-0CHDK124TR"
type = "implemented_by"
target = "task:define-portable-agent-skill-contract"

[[docgraph_generated.inverses]]
source = "plan:make-agent-guidance-portable#s-BAP0AYXWFP"
type = "implemented_by"
target = "task:support-configurable-skill-targets"

[[docgraph_generated.inverses]]
source = "plan:make-agent-guidance-portable#s-GBKQP7BGTW"
type = "implemented_by"
target = "task:generate-dynamic-agent-guidance"

+++
<a id="s-Z7JRX38EM3"></a>
# Make generated agent guidance portable

<a id="s-09NE5NTE6V"></a>
## Objective

Make the CLI-emitted docgraph skill discoverable and current in each configured agent
environment without repository-specific copies or symlinks. Keep stable operating
guidance authored, but derive model inventories and compatibility metadata from their
actual implementation or repository configuration.

<a id="s-5BF0JTA00E"></a>
## Portfolio priority

Begin after the CI parity contract is reliable. This is near-term product repair, not
optional polish: #18 means the currently emitted integration is not independently
discoverable in common agent environments.

<a id="s-33PB3BT89Q"></a>
## Report coverage

This plan covers:

- [#18](https://github.com/JTarasovic/docgraph/issues/18): missing skill discovery
  metadata plus a fixed, unverified output location.
- [#16](https://github.com/JTarasovic/docgraph/issues/16): hardcoded predicate and
  other derivable inventories in the skill bundle.

They belong together because configurable targets are useful only if every installed
copy has the same discoverable, generated payload, and generated payload is useful
only if `instructions check` verifies the locations agents actually load.

<a id="s-M9819C22H0"></a>
## Priority and sequence

1. Define the portable skill's discovery metadata, target model, ownership, and
   compatibility contract in the product references.
2. Classify skill content as authored guidance, CLI-contract material, or
   repository-specific generated context, and eliminate duplicate inventories.
3. Implement multiple configured targets and verify every managed target while
   preserving repository-owned additions.

<a id="s-D8D68VN39H"></a>
## Work slices

<a id="s-0CHDK124TR"></a>
### Define the portable agent skill contract

Specify valid `SKILL.md` metadata, default and explicit target behavior, collision
handling, and the relationship between the canonical payload and installed copies.

<a id="s-GBKQP7BGTW"></a>
### Generate dynamic agent guidance

Give predicates and other derivable inventories one source of truth and test generated
guidance against CLI introspection.

<a id="s-BAP0AYXWFP"></a>
### Support configurable agent skill targets

Install, check, and repair the canonical bundle at all configured agent discovery
paths with safe previews and precise per-target status.

<a id="s-8AA9RT3YK1"></a>
## Completion

A freshly initialized repository exposes a named, described docgraph skill to each
configured agent without manual bridging. `instructions check` detects missing,
modified, or incompatible copies at every target, and generated inventories cannot
drift from `docgraph describe` or the current repository model.
