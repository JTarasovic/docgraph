+++

id = "issue:next-hides-promotable-backlog-tasks"
type = "issue"
state = "resolved"

[properties]
title = "Next hides promotable backlog tasks"

[[relations]]
type = "affects"
target = "plan:project-aware-commands"

[[relations]]
type = "affects"
target = "plan:address-post-v0-reference-work"

[docgraph_generated]
schema_version = 1

+++
<a id="s-NAN8ABRGET"></a>
# Next hides promotable backlog tasks

When an active plan has no in-progress or ready task, `docgraph next` returns the
plan even when one or more backlog tasks have satisfied dependencies. This hides
the concrete work that can be promoted and makes the plan appear exhausted.

Update the repository's `next_work` logic to return dependency-ready backlog tasks
with a reason such as `ready to promote`. Keep backlog inventory and prioritization
out of scope until they are needed.

<a id="s-FYZ8ZBK6KJ"></a>
## Resolution

`next_work` now returns dependency-ready backlog tasks as `ready to promote` and
suppresses the opaque active-plan fallback when such tasks exist.
