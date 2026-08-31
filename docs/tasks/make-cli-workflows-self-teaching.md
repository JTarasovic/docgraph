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

Common authoring failures identify their remedy at the point of failure, and command
help is sufficient to perform routine mutations without guessing syntax or opening a
secondary guide.

<a id="s-6A18M4SSSE"></a>
## Scope

- Add actionable remedies to missing-section-ID, query-arity, and related diagnostics.
- Let `normalize` accept a bounded target or clearly explain and exemplify its
  repository-wide behavior.
- Add concise examples for document creation, adoption, properties, relations, queries,
  normalization, frontmatter synchronization, and validation.
- Document stable JSON fields and align inconsistent names where compatibility permits.
- Put the three critical sequencing rules directly in the root portable skill:
  normalize after headings, sync generated frontmatter after semantic links or
  relations, and validate after mutations.

<a id="s-1ZPP7MQ5HH"></a>
## Acceptance

- The issue's failed create-edit-relate sequence names `docgraph normalize` as the
  recovery action.
- Help output demonstrates repeatable property syntax and the scope of maintenance
  commands.
- Structured output documentation prevents `node`/`target` ambiguity.
- A clean-room agent test can complete the representative workflows using command help
  and the root skill without filesystem reconstruction or hand-written frontmatter.
