+++

id = "issue:configurable-skill-frontmatter"
type = "issue"
state = "open"

[properties]
title = "Allow configured metadata in generated agent skills"

[[relations]]
type = "affects"
target = "reference:design#s-64KP745XR0"

[docgraph_generated]
schema_version = 1

+++
<a id="s-CNBGFD9VST"></a>
# Allow configured metadata in generated agent skills

The release supplies a fixed `SKILL.md` frontmatter with `name` and
`description`. `docgraph instructions sync` writes the same managed file to every
configured skill target, so a consuming repository cannot add agent-specific
frontmatter such as `model` or `reasoning` without `instructions check` reporting
the file as modified and the next sync replacing that edit.

Provide a way to configure additional frontmatter for installed skills. Resolve
whether settings belong to the whole repository or to individual skill targets,
and how to keep required discovery metadata and sync/check behavior consistent.
