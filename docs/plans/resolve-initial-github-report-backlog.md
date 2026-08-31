+++

id = "plan:resolve-initial-github-report-backlog"
type = "plan"
state = "proposed"

[properties]
title = "Resolve the initial GitHub report backlog"

[docgraph_generated]
schema_version = 1

[[docgraph_generated.incoming]]
source = "task:complete-logic-property-querying"
predicate = "implements"
target = "plan:resolve-initial-github-report-backlog#s-ZPVAXE0Y5D"

[[docgraph_generated.incoming]]
source = "task:complete-logic-property-querying"
predicate = "part_of"
target = "plan:resolve-initial-github-report-backlog"

[[docgraph_generated.incoming]]
source = "task:enable-safe-schema-repair"
predicate = "implements"
target = "plan:resolve-initial-github-report-backlog#s-9S79WDF4RE"

[[docgraph_generated.incoming]]
source = "task:enable-safe-schema-repair"
predicate = "part_of"
target = "plan:resolve-initial-github-report-backlog"

[[docgraph_generated.incoming]]
source = "task:fix-repository-markdown-link-resolution"
predicate = "implements"
target = "plan:resolve-initial-github-report-backlog#s-YJD856GAW5"

[[docgraph_generated.incoming]]
source = "task:fix-repository-markdown-link-resolution"
predicate = "part_of"
target = "plan:resolve-initial-github-report-backlog"

[[docgraph_generated.incoming]]
source = "task:handle-yaml-frontmatter-adoption"
predicate = "implements"
target = "plan:resolve-initial-github-report-backlog#s-QMMBHDM8E1"

[[docgraph_generated.incoming]]
source = "task:handle-yaml-frontmatter-adoption"
predicate = "part_of"
target = "plan:resolve-initial-github-report-backlog"

[[docgraph_generated.incoming]]
source = "task:harden-entity-id-validation"
predicate = "implements"
target = "plan:resolve-initial-github-report-backlog#s-BKE7HYTQ3D"

[[docgraph_generated.incoming]]
source = "task:harden-entity-id-validation"
predicate = "part_of"
target = "plan:resolve-initial-github-report-backlog"

[[docgraph_generated.incoming]]
source = "task:improve-query-and-section-inspection"
predicate = "implements"
target = "plan:resolve-initial-github-report-backlog#s-FK5WW602EZ"

[[docgraph_generated.incoming]]
source = "task:improve-query-and-section-inspection"
predicate = "part_of"
target = "plan:resolve-initial-github-report-backlog"

[[docgraph_generated.incoming]]
source = "task:make-cli-workflows-self-teaching"
predicate = "implements"
target = "plan:resolve-initial-github-report-backlog#s-N3GVK60VZC"

[[docgraph_generated.incoming]]
source = "task:make-cli-workflows-self-teaching"
predicate = "part_of"
target = "plan:resolve-initial-github-report-backlog"

[[docgraph_generated.inverses]]
source = "plan:resolve-initial-github-report-backlog"
type = "contains"
target = "task:complete-logic-property-querying"

[[docgraph_generated.inverses]]
source = "plan:resolve-initial-github-report-backlog"
type = "contains"
target = "task:enable-safe-schema-repair"

[[docgraph_generated.inverses]]
source = "plan:resolve-initial-github-report-backlog"
type = "contains"
target = "task:fix-repository-markdown-link-resolution"

[[docgraph_generated.inverses]]
source = "plan:resolve-initial-github-report-backlog"
type = "contains"
target = "task:handle-yaml-frontmatter-adoption"

[[docgraph_generated.inverses]]
source = "plan:resolve-initial-github-report-backlog"
type = "contains"
target = "task:harden-entity-id-validation"

[[docgraph_generated.inverses]]
source = "plan:resolve-initial-github-report-backlog"
type = "contains"
target = "task:improve-query-and-section-inspection"

[[docgraph_generated.inverses]]
source = "plan:resolve-initial-github-report-backlog"
type = "contains"
target = "task:make-cli-workflows-self-teaching"

[[docgraph_generated.inverses]]
source = "plan:resolve-initial-github-report-backlog#s-9S79WDF4RE"
type = "implemented_by"
target = "task:enable-safe-schema-repair"

[[docgraph_generated.inverses]]
source = "plan:resolve-initial-github-report-backlog#s-BKE7HYTQ3D"
type = "implemented_by"
target = "task:harden-entity-id-validation"

[[docgraph_generated.inverses]]
source = "plan:resolve-initial-github-report-backlog#s-FK5WW602EZ"
type = "implemented_by"
target = "task:improve-query-and-section-inspection"

[[docgraph_generated.inverses]]
source = "plan:resolve-initial-github-report-backlog#s-N3GVK60VZC"
type = "implemented_by"
target = "task:make-cli-workflows-self-teaching"

[[docgraph_generated.inverses]]
source = "plan:resolve-initial-github-report-backlog#s-QMMBHDM8E1"
type = "implemented_by"
target = "task:handle-yaml-frontmatter-adoption"

[[docgraph_generated.inverses]]
source = "plan:resolve-initial-github-report-backlog#s-YJD856GAW5"
type = "implemented_by"
target = "task:fix-repository-markdown-link-resolution"

[[docgraph_generated.inverses]]
source = "plan:resolve-initial-github-report-backlog#s-ZPVAXE0Y5D"
type = "implemented_by"
target = "task:complete-logic-property-querying"

+++
<a id="s-5F5H4ZMS36"></a>
# Resolve the initial GitHub report backlog

<a id="s-E6GV04WZ1F"></a>
## Objective

Resolve or explicitly disposition the ten GitHub reports opened from the first
large-corpus dogfood session. Preserve GitHub as the canonical issue record while
organizing implementation into repository-native tasks with clear acceptance and
dependency boundaries.

<a id="s-5C4CDHDWX5"></a>
## Structure

This is one plan rather than a plan per theme. The reports form one bounded quality
backlog, while each theme is small enough to implement and review as a task. A theme
should become its own plan only if investigation expands it into multiple independently
sequenced deliverables.

The task-to-report mapping is exhaustive:

- Safe schema repair: [#2](https://github.com/JTarasovic/docgraph/issues/2).
- Logic property querying and discovery: [#3](https://github.com/JTarasovic/docgraph/issues/3)
  and [#4](https://github.com/JTarasovic/docgraph/issues/4).
- Entity ID validation: [#5](https://github.com/JTarasovic/docgraph/issues/5).
- Repository Markdown link resolution: [#6](https://github.com/JTarasovic/docgraph/issues/6)
  and [#7](https://github.com/JTarasovic/docgraph/issues/7).
- YAML-frontmatter adoption: [#8](https://github.com/JTarasovic/docgraph/issues/8).
- Query and section inspection: [#9](https://github.com/JTarasovic/docgraph/issues/9)
  and [#10](https://github.com/JTarasovic/docgraph/issues/10).
- Self-teaching CLI workflows: [#11](https://github.com/JTarasovic/docgraph/issues/11),
  including the remaining cross-cutting usability work after #4, #9, and #10.

<a id="s-5KCP8DYN1M"></a>
## Priority and sequence

1. Correctness and blocked-adoption fixes: safe schema repair, entity ID validation,
   and repository Markdown link resolution.
2. Model authoring and adoption: complete logic property querying and make YAML
   frontmatter detectable and migratable.
3. Human retrieval: render query results readably and expose document outlines.
4. Integrative usability: finish the self-teaching CLI task after the predicate and
   retrieval tasks establish the surfaces it must document and explain.

Tasks within the first three waves are independent and may proceed in parallel. The
priority order is a triage order, not an artificial implementation dependency.

<a id="s-K2B397FNBC"></a>
## Work slices

<a id="s-9S79WDF4RE"></a>
### Enable safe schema repair

Provide a bounded, validated way to repair data after a schema constraint is tightened
without temporarily disabling the constraint.

<a id="s-ZPVAXE0Y5D"></a>
### Complete logic property querying and discovery

Expose array and entity-valued properties to restricted logic and make the complete
base-predicate vocabulary discoverable from documentation and the CLI.

<a id="s-BKE7HYTQ3D"></a>
### Harden entity ID validation

Reject unsafe local ID components consistently at every creation and adoption boundary.

<a id="s-YJD856GAW5"></a>
### Fix repository Markdown link resolution

Resolve ordinary sibling links and distinguish valid repository-file links from truly
broken targets.

<a id="s-QMMBHDM8E1"></a>
### Handle YAML frontmatter during adoption

Detect YAML frontmatter before misleading Markdown or TOML failures and provide a safe
migration path into docgraph's TOML frontmatter.

<a id="s-FK5WW602EZ"></a>
### Improve query and section inspection

Give humans readable query tables and a cheap, structured document outline operation.

<a id="s-N3GVK60VZC"></a>
### Make CLI workflows self-teaching

Put remedies, examples, output-shape guidance, and mutation sequencing at the command
and error surfaces where users encounter them.

<a id="s-A18E01YG9A"></a>
## Completion

Every report is covered by regression tests and user-facing documentation, its agreed
behavior passes the repository check suite, and the corresponding GitHub issue is
closed or explicitly split with a linked follow-up and recorded rationale. No report
is considered resolved merely because it has been grouped into this plan.
