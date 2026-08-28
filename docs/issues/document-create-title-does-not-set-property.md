+++

id = "issue:document-create-title-does-not-set-property"
type = "issue"
state = "resolved"

[properties]
title = "Document create title does not set the title property"

[[relations]]
type = "affects"
target = "task:implement-managed-document-lifecycle"

[[relations]]
type = "affects"
target = "reference:config-grammar"

[docgraph_generated]
schema_version = 1

+++
<a id="s-SCAQTD5R6T"></a>
# Document create title does not set the title property

`docgraph document create` accepts the required `--title` option and uses it for
the Markdown heading, but does not populate the entity's required `title`
property. Creation therefore fails validation unless callers redundantly pass
`--property title=...`, contrary to the documented command contract. Existing
tests hide the defect by supplying both values.

Make `--title` populate `properties.title`, reject a conflicting explicit title
property, and update the tests to exercise the documented invocation.

<a id="s-6AFTAYBJ72"></a>
## Resolution

Document creation now derives a declared `title` property from `--title`, rejects
conflicting values, and tests the documented invocation without redundant options.
