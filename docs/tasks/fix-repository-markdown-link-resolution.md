+++

id = "task:fix-repository-markdown-link-resolution"
type = "task"
state = "backlog"

[properties]
title = "Fix repository Markdown link resolution"

[[relations]]
type = "part_of"
target = "plan:resolve-initial-github-report-backlog"

[[relations]]
type = "implements"
target = "plan:resolve-initial-github-report-backlog#s-YJD856GAW5"

[docgraph_generated]
schema_version = 1

+++
<a id="s-6MCRJRYPS2"></a>
# Fix repository Markdown link resolution

Address [GitHub issue #6](https://github.com/JTarasovic/docgraph/issues/6) and
[GitHub issue #7](https://github.com/JTarasovic/docgraph/issues/7).

<a id="s-V1FS650YY1"></a>
## Outcome

Markdown links follow ordinary repository-relative semantics, while graph warnings
remain reserved for targets that are genuinely missing or malformed.

<a id="s-1JMARY2RJ4"></a>
## Scope

- Resolve bare sibling targets relative to the containing document exactly as `./`
  targets are resolved.
- Recognize existing files inside the repository but outside the configured documents
  root as valid informational repository links.
- Preserve graph-node and stable-section resolution for managed targets.
- Keep missing files, escaping paths, and invalid fragments distinguishable in
  diagnostics.

<a id="s-H8KTFKF2BN"></a>
## Acceptance

- `x.md` and `./x.md` resolve to the same sibling managed document.
- Links to existing repository files outside the documents root do not emit
  `broken-internal-link`.
- Missing managed and repository files still warn with actionable classification.
- Cross-platform path and percent-encoding cases have regression coverage.
