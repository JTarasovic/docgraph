+++

id = "task:dogfood-project-next"
type = "task"
state = "done"

[properties]
title = "Dogfood project-level next"

[[relations]]
type = "part_of"
target = "plan:project-aware-commands"

[[relations]]
type = "implements"
target = "plan:project-aware-commands#s-9MNKBYJ002"

[docgraph_generated]
schema_version = 1

[[docgraph_generated.backlinks]]
source = "plan:project-aware-commands#s-9MNKBYJ002"
target = "docs/tasks/dogfood-project-next.md"

+++
<a id="s-PEB9FRKNHH"></a>
# Dogfood project-level next

Configure a top-level `next` command whose repository logic returns honest candidate sets across active plans, ready unblocked tasks, proposed plans, and unresolved issues, with an optional plan filter.
