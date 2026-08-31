+++

id = "task:improve-query-and-section-inspection"
type = "task"
state = "done"

[properties]
title = "Improve query and section inspection"

[[relations]]
type = "part_of"
target = "plan:resolve-initial-github-report-backlog"

[[relations]]
type = "implements"
target = "plan:resolve-initial-github-report-backlog#s-FK5WW602EZ"

[docgraph_generated]
schema_version = 1

[[docgraph_generated.incoming]]
source = "task:make-cli-workflows-self-teaching"
predicate = "depends_on"
target = "task:improve-query-and-section-inspection"

[[docgraph_generated.inverses]]
source = "task:improve-query-and-section-inspection"
type = "required_by"
target = "task:make-cli-workflows-self-teaching"

+++
<a id="s-A0N327724R"></a>
# Improve query and section inspection

Address [GitHub issue #9](https://github.com/JTarasovic/docgraph/issues/9) and
[GitHub issue #10](https://github.com/JTarasovic/docgraph/issues/10).

<a id="s-FCBVZJFHME"></a>
## Outcome

Humans can read named-query and query-backed custom-command results and inspect a long
document's structure directly, while automation retains complete stable structured
output.

<a id="s-B9GR3QQ3FT"></a>
## Scope

- Render non-JSON named-query and query-backed custom-command output as a table using
  declared column names and types, including the repository's `docgraph next` command.
- Preserve the current envelope for `--json` and document its stability contract.
- Add an outline operation returning section ID, heading, level, parent, and line span.
- Support a bounded way to inspect section content without returning an entire large
  subtree by default.
- Use the commit-pinned
  [measurement plan reproduction](https://github.com/neutrinos-os/neutrinos/blob/dd88eff9c2abd68a756a5009eb9b7a26392d941e/docs/plans/measurement.md)
  from issue #10 to derive a representative deep-document regression fixture.

<a id="s-GK2ZSPEE1T"></a>
## Acceptance

- Default output from both `docgraph query` and query-backed custom commands is a
  readable deterministic table for empty, narrow, and wide result sets.
- JSON output remains machine-readable and contains query, column, and row metadata.
- A document outline can be obtained without reading or grepping its Markdown file.
- Outline and bounded-section results preserve stable IDs and hierarchy in text and
  JSON modes.
- Regression coverage exercises a document comparable in size and hierarchy to the
  reported measurement plan, not only minimal synthetic sections.
