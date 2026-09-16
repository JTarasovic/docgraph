+++

id = "task:rewrite-project-overview"
type = "task"
state = "done"

[properties]
title = "Rewrite the project overview"

[[relations]]
type = "part_of"
target = "plan:overhaul-onboarding-readme"

[[relations]]
type = "implements"
target = "reference:design#s-YNPWHT3H8T"

[[relations]]
type = "implements"
target = "reference:design#s-FEFSK4BQTV"

[docgraph_generated]
schema_version = 1

+++
<a id="s-5GKKKX8WRT"></a>
# Rewrite the project overview

Rewrite the README overview to lead with the general capability, then the problem
it solves, kept digestible as a few sentences and tight bullets rather than a wall
of prose.

- **What it is.** A repository-native engine for codifying an ontology — entity
  types, typed properties, relationships, and workflow states — over Markdown
  documentation, then querying, validating, and safely mutating that structure.
  Mechanism lives in the tool; the ontology and policy are defined by the
  repository.
- **The problem it solves.** Structured document semantics drift; humans and
  especially agents do not reliably keep entity state, cross-references, and
  relationships consistent, and reconstruct meaning with repo-wide grep and
  inconsistent hand-edits. State that lives only in an agent's context gets
  compacted, cleared, or ignored, and the user cannot edit, configure, or query
  it.
- **What docgraph does about it.** It makes the codified semantics canonical in
  Git alongside the prose — durable, user-editable, configurable, and queryable —
  and answers impact through the tool instead of grep, for any ontology the
  repository declares.
- **How it works.** Markdown and frontmatter in Git are canonical; the graph,
  index, and derived state are disposable and rebuildable. Humans and agents edit
  prose freely but change managed state through docgraph commands that validate
  impact before writing. Link to `docs/reference/design.md` for the full model and
  the quickstart for hands-on.

Do not frame the tool around any single use case; issue/task/plan tracking is one
example of what can be modeled, not the definition of the tool.

<a id="s-W0JCRGNS31"></a>
## Result

The README now opens with a one-line capability statement and a four-bullet
what/problem/what-it-does/how-it-works overview grounded in the mechanism-versus-
policy framing, with a pointer to `docs/reference/design.md` and the quickstart. No
single use case frames the tool.
