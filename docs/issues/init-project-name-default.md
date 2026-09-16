+++

id = "issue:init-project-name-default"
type = "issue"
state = "open"

[properties]
title = "Init derives project name from worktree directory"

[[relations]]
type = "affects"
target = "reference:config-grammar#s-TW0V0THMJD"

[docgraph_generated]
schema_version = 1

+++
<a id="s-BFGEDP9DBK"></a>
# Init derives project name from worktree directory

When `docgraph init` runs without `--name`, it uses the final directory name of
`git rev-parse --show-toplevel` as the project name. In a linked worktree, that
can be a local worktree label such as `jdt.next` rather than the repository's name.
The value is then written to `.docgraph/project.toml`.

Reconsider the default so a fresh configuration does not silently acquire a
worktree-specific project name. Decide whether to derive a repository identity,
require an explicit name, or retain the directory fallback when no better source
is available.
