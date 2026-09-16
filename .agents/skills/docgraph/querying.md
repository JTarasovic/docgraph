# Querying

Use `get`, `neighbors`, `path`, and `search` for generic retrieval. Use
`docgraph query <name> --arg name=value` for a human-readable typed table, or add
`--json` for the stable query/columns/rows envelope. Query-backed repository commands
use the same two output modes. Use `docgraph outline <entity>` to enumerate stable
section IDs, headings, levels, parents, and line spans before reading a long document.
`docgraph get <stable-section>` returns 40 content lines by default; select another
positive limit with `--lines` or opt into the complete body with `--all`. Structured
results distinguish explicit relations, Markdown links, and search matches.

Repository rules in `.docgraph/logic.dl` may call built-in predicates. Run
`docgraph describe --all` for the current names, arities, ordered arguments, and
value shapes under `logic.predicates`.

Property predicates preserve the declared scalar type. An array uses the predicate
for its declared item type and contributes one fact per member. For example,
`entity_property_string(Id, "labels", Label)` enumerates a string array, while
`!entity_property_string(Id, "labels", _)` tests that it has no members after `Id`
has been grounded. Join a string member that contains an entity ID to `entity/1` to
make the query output entity-valued:

```text
related(Source, Target) :- entity_property_string(Source, "related", Target), entity(Target).
```
