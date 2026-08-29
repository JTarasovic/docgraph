+++

id = "issue:help-subcommand-hides-repository-commands"
type = "issue"
state = "resolved"

[properties]
title = "Help subcommand hides repository commands"

[[relations]]
type = "affects"
target = "milestone:v1-0"

[docgraph_generated]
schema_version = 1

+++
<a id="s-PKN605C0P4"></a>
# Help subcommand hides repository commands

Root `docgraph --help` and `docgraph -h` use the project-aware help renderer,
but the equivalent `docgraph help` subcommand delegates directly to Clap's static
help output. As a result, the subcommand omits every configured repository command
even when the current repository loads and describes those commands successfully.

Route all equivalent root-help invocations through the same project-aware renderer.
They should present the same repository commands in the same order while preserving
the built-in usage, commands, and options below them.

Resolved by routing the root `help` subcommand through the same renderer as
`--help` and `-h`, with regression coverage that requires identical repository
command visibility and ordering for all three forms.
