+++

id = "task:harden-entity-id-validation"
type = "task"
state = "done"

[properties]
title = "Harden entity ID validation"

[[relations]]
type = "part_of"
target = "plan:resolve-initial-github-report-backlog"

[[relations]]
type = "implements"
target = "plan:resolve-initial-github-report-backlog#s-BKE7HYTQ3D"

[docgraph_generated]
schema_version = 1

+++
<a id="s-0K3N9WK80D"></a>
# Harden entity ID validation

Address [GitHub issue #5](https://github.com/JTarasovic/docgraph/issues/5).

<a id="s-HNAN1WB3QX"></a>
## Outcome

Every canonical entity ID has a portable, unambiguous local component suitable for CLI
arguments, frontmatter, references, query results, and derived-state keys.

<a id="s-VZCX8HEKHW"></a>
## Scope

- Specify the local-component grammar and compatibility behavior for existing corpora.
- Apply the same validation at document creation, adoption, parsing, and mutation
  boundaries.
- Report the invalid ID and offending character or construct in diagnostics.
- Cover IDs produced from common titles without silently rewriting user input.

<a id="s-89PEG162TG"></a>
## Acceptance

- Spaces, path separators, empty local components, and other disallowed characters are
  rejected before a managed write.
- Existing valid IDs using RFC 3986 unreserved ASCII characters continue to work
  across platforms; the local component begins with a letter or digit.
- Validation and CLI tests exercise both authored files and mutation commands.
