+++

id = "task:generate-dynamic-agent-guidance"
type = "task"
state = "done"

[properties]
title = "Generate dynamic agent guidance"

[[relations]]
type = "part_of"
target = "plan:make-agent-guidance-portable"

[[relations]]
type = "implements"
target = "plan:make-agent-guidance-portable#s-GBKQP7BGTW"

[[relations]]
type = "depends_on"
target = "task:define-portable-agent-skill-contract"

[docgraph_generated]
schema_version = 1

[[docgraph_generated.incoming]]
source = "task:support-configurable-skill-targets"
predicate = "depends_on"
target = "task:generate-dynamic-agent-guidance"

[[docgraph_generated.inverses]]
source = "task:generate-dynamic-agent-guidance"
type = "required_by"
target = "task:support-configurable-skill-targets"

+++
<a id="s-G5RW5NRW2K"></a>
# Generate dynamic agent guidance

Address [#16](https://github.com/JTarasovic/docgraph/issues/16) by auditing every
list and version in the emitted skill and generated instruction block. Classify each
item as stable authored procedure, CLI-contract data, or repository-model data.

Keep procedures concise and authored. Direct agents to `docgraph describe --all` for
the built-in predicate inventory rather than copying it into the skill. Inject repository-specific types,
relations, workflows, queries, and commands only through the existing model appendix
or an equivalent deterministic context surface. Do not create a second template that
can drift independently.

<a id="s-TZR5BVNGMS"></a>
## Acceptance

- Built-in predicate names, arities, argument names, and value shapes are exposed by
  `docgraph describe --all`; the skill points there instead of duplicating them.
- Repository-specific inventories come from loaded configuration, not portable files.
- Stable behavioral guidance remains readable without expanding the root skill.
- Tests confirm `docgraph describe --all` exposes the built-in inventory for multiple
  fixtures.
- The audit records any intentional static duplication and why generation is worse.

<a id="s-8X83W2A8F3"></a>
## Audit

- The predicate inventory in `querying.md` was a duplicate of `describe --all`; the
  skill now directs agents to that command.
- The repository model appendix is already generated from loaded configuration.
  Portable files do not list a consuming repository's types or relations.
- Command names in the root skill and guides are examples within authored procedures.
  `docgraph --help` supplies the current command inventory; generating the examples
  would lose the task-oriented advice around them.
- `skill.toml` carries release compatibility versions. Release tooling updates its
  CLI version, and `instructions check` verifies installed copies.
