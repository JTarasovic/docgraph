+++

id = "task:bulletize-validation-action-usage"
type = "task"
state = "done"

[properties]
title = "Convert validation-action usage to bullets and examples"

[[relations]]
type = "part_of"
target = "plan:overhaul-onboarding-readme"

[[relations]]
type = "implements"
target = "reference:validation-action"

[docgraph_generated]
schema_version = 1

+++
<a id="s-5APYH9BZMC"></a>
# Convert validation-action usage to bullets and examples

Replace the validation-action prose with headings, bullets, and minimal examples:

- **Basic usage** — the existing YAML snippet.
- **Pin to a SHA / select a version** — bullet notes on the `version` input.
- **Install only / runtime only** — a short bulleted list of `install`,
  `install:false`, `.../install@`, and `.../install-runtime@`.

Link to the validation-action contract in `docs/reference/validation-action.md`
for the full option and output reference instead of restating it.

<a id="s-WW9PJ1HVFY"></a>
## Result

The validation-action section is now three subsections — Basic usage (YAML
snippet), Pin to a SHA or select a version, and Install only or runtime only
(bulleted `install: false`, `install@`, and `install-runtime@`) — with the prose
replaced and a link to the validation-action contract for full options and
outputs.
