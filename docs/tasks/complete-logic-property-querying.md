+++

id = "task:complete-logic-property-querying"
type = "task"
state = "backlog"

[properties]
title = "Complete logic property querying and discovery"

[[relations]]
type = "part_of"
target = "plan:resolve-initial-github-report-backlog"

[[relations]]
type = "implements"
target = "plan:resolve-initial-github-report-backlog#s-ZPVAXE0Y5D"

[docgraph_generated]
schema_version = 1

[[docgraph_generated.incoming]]
source = "task:make-cli-workflows-self-teaching"
predicate = "depends_on"
target = "task:complete-logic-property-querying"

[[docgraph_generated.inverses]]
source = "task:complete-logic-property-querying"
type = "required_by"
target = "task:make-cli-workflows-self-teaching"

+++
<a id="s-4D8Q6SJEF0"></a>
# Complete logic property querying and discovery

Address [GitHub issue #3](https://github.com/JTarasovic/docgraph/issues/3) and
[GitHub issue #4](https://github.com/JTarasovic/docgraph/issues/4).

<a id="s-ZYA801RRTD"></a>
## Outcome

Restricted repository logic can inspect every supported property category, and an
author can discover the predicate names, arities, argument meanings, and value shapes
without probing validation errors.

<a id="s-3MBTXMGAQ1"></a>
## Scope

- Add flat facts for array membership and entity-valued properties without introducing
  list terms or weakening declared property typing.
- Include the complete base-predicate vocabulary in structured `describe` output.
- Generate or maintain the same vocabulary in the portable querying/config guidance.
- Keep predicate naming and result types provider-neutral and backwards compatible.

<a id="s-C6QAM3ZBZ4"></a>
## Acceptance

- Queries can test array membership, detect empty arrays, and join entity-valued
  properties.
- `docgraph describe` exposes all built-ins with arity and argument information.
- The shipped skill/reference material explains each predicate with a minimal example.
- Validation and logic-runtime tests cover supported and rejected property shapes.
