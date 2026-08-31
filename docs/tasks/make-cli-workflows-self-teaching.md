+++

id = "task:make-cli-workflows-self-teaching"
type = "task"
state = "backlog"

[properties]
title = "Make CLI workflows self-teaching"

[[relations]]
type = "part_of"
target = "plan:resolve-initial-github-report-backlog"

[[relations]]
type = "implements"
target = "plan:resolve-initial-github-report-backlog#s-N3GVK60VZC"

[[relations]]
type = "depends_on"
target = "task:complete-logic-property-querying"

[[relations]]
type = "depends_on"
target = "task:improve-query-and-section-inspection"

[docgraph_generated]
schema_version = 1

+++
<a id="s-0MDGTGF0KW"></a>
# Make CLI workflows self-teaching

Address [GitHub issue #11](https://github.com/JTarasovic/docgraph/issues/11) after the
predicate-discovery and retrieval-inspection tasks establish their final interfaces.

<a id="s-018ZDPDDQ9"></a>
## Outcome

Common authoring failures offer context-aware next-step guidance at the point of
failure, and organized command help is sufficient to perform routine mutations without
guessing syntax or opening a secondary guide.

<a id="s-6A18M4SSSE"></a>
## Scope

- Add actionable, conditional next-step suggestions to missing-section-ID, query-arity,
  and related diagnostics. Use unconditional wording only when the command can prove
  the remedy is always appropriate; otherwise hedge it and explain when it applies.
- Let `normalize` accept a bounded target or clearly explain and exemplify its
  repository-wide behavior.
- Add scenario-derived examples for document creation, adoption, properties, relations,
  queries, normalization, frontmatter synchronization, and validation. Draw from the
  repository's scenario corpora and include enough representative variations to avoid
  forcing syntax rediscovery.
- Normalize surprising structured-output field names across related commands, with an
  explicit compatibility strategy, and document each command's stable JSON shape.
- Reorganize top-level and grouped command help so the command surface is scannable and
  related workflows are apparent rather than presented as one intimidating flat list.
- Replace the root portable skill's terse guide-name router with a concise,
  scenario-driven "if this, then that" table. Include the critical sequencing rules
  there, with the same conditional guidance standard used by CLI diagnostics.

<a id="s-1ZPP7MQ5HH"></a>
## Acceptance

- The issue's failed create-edit-relate sequence suggests `docgraph normalize` when
  appropriate without presenting a conditional repair as universally required.
- Help output is organized by recognizable workflows and demonstrates repeatable
  property syntax, multiple scenario-derived examples, and the scope of maintenance
  commands.
- Related structured outputs use predictable field names, and per-command
  documentation prevents `node`/`target` ambiguity during compatibility transitions.
- The root skill routes by user scenario and makes mutation sequencing discoverable
  without requiring the reader to infer which guide name applies.
- A clean-room agent test can complete the representative workflows using command help
  and the root skill without filesystem reconstruction or hand-written frontmatter.
