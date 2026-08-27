+++

id = "plan:project-aware-commands"
type = "plan"
state = "completed"

[properties]
title = "Make repository work discoverable"

[[relations]]
type = "implements"
target = "reference:design#s-DAR1R6WHJE"

[[relations]]
type = "implements"
target = "reference:config-grammar#s-3B3J65MSQN"

[docgraph_generated]
schema_version = 1

[[docgraph_generated.incoming]]
source = "task:dogfood-project-next"
predicate = "implements"
target = "plan:project-aware-commands#s-9MNKBYJ002"

[[docgraph_generated.incoming]]
source = "task:dogfood-project-next"
predicate = "part_of"
target = "plan:project-aware-commands"

[[docgraph_generated.incoming]]
source = "task:implement-repository-commands"
predicate = "implements"
target = "plan:project-aware-commands#s-4703EK4457"

[[docgraph_generated.incoming]]
source = "task:implement-repository-commands"
predicate = "part_of"
target = "plan:project-aware-commands"

[[docgraph_generated.inverses]]
source = "plan:project-aware-commands"
type = "contains"
target = "task:dogfood-project-next"

[[docgraph_generated.inverses]]
source = "plan:project-aware-commands"
type = "contains"
target = "task:implement-repository-commands"

[[docgraph_generated.inverses]]
source = "plan:project-aware-commands#s-4703EK4457"
type = "implemented_by"
target = "task:implement-repository-commands"

[[docgraph_generated.inverses]]
source = "plan:project-aware-commands#s-9MNKBYJ002"
type = "implemented_by"
target = "task:dogfood-project-next"

+++
<a id="s-C46KWN6VCE"></a>
# Make repository work discoverable

<a id="s-MHKD6W39HB"></a>
## Objective

Let repositories expose domain-shaped commands over docgraph primitives, beginning with a project-level answer to “what’s next?”

<a id="s-9RXDWW8FGZ"></a>
## Steps

<a id="s-4703EK4457"></a>
### Implement repository-defined commands

[Implement repository-defined commands](../tasks/implement-repository-commands.md) from `commands.toml`, including nested command paths, project-aware help, query dispatch, and generic mutation mappings.

<a id="s-9MNKBYJ002"></a>
### Dogfood project-level next

[Dogfood project-level next](../tasks/dogfood-project-next.md) so this repository reports actionable tasks across active plans, proposed plans, and open issues without inventing priority.

<a id="s-3KGYRA0ARX"></a>
## Completion

The repository can run `docgraph next` and optionally filter by plan, command configuration is validated, and the behavior is covered by runtime-backed tests.

<a id="s-7KFVRXQC54"></a>
## Result

Implemented. `commands.toml` supports query, transition, and relation commands with nested paths and project-aware help. This repository’s `next` command reports in-progress and ready work across active plans, proposed plans, and open issues; `--plan` narrows the candidate set without claiming an unsupported priority order.
