+++

id = "task:handle-yaml-frontmatter-adoption"
type = "task"
state = "done"

[properties]
title = "Handle YAML frontmatter during adoption"

[[relations]]
type = "part_of"
target = "plan:resolve-initial-github-report-backlog"

[[relations]]
type = "implements"
target = "plan:resolve-initial-github-report-backlog#s-QMMBHDM8E1"

[docgraph_generated]
schema_version = 1

+++
<a id="s-XA6CTQTQM5"></a>
# Handle YAML frontmatter during adoption

Address [GitHub issue #8](https://github.com/JTarasovic/docgraph/issues/8).

<a id="s-4C7RNK71QX"></a>
## Outcome

Existing Markdown with YAML frontmatter is recognized before generic Markdown or TOML
parsing produces misleading errors, and repositories have a safe path to docgraph TOML
frontmatter.

<a id="s-B8YZMYPZ03"></a>
## Scope

- Detect opening `---` YAML frontmatter before heading normalization, validation, and
  adoption.
- Emit one diagnostic naming YAML, the required `+++` TOML format, and the applicable
  migration command.
- Add a previewable migration that preserves representable and unrecognized keys,
  refuses ambiguous or lossy conversions, and composes with batch adoption.
- Document recovery and rollback behavior for multi-file migrations.

<a id="s-HMK424HFC8"></a>
## Acceptance

- YAML-fronted documents no longer cascade into false section-ID or TOML syntax errors.
- Migration dry runs show exact file changes and never partially convert a corpus.
- Scalars, arrays, dates, nested/unrecognized data, malformed YAML, and mixed corpora
  have explicit tested behavior.
- Normalization and adoption work normally after a successful conversion.
