# Querying

Use `get`, `neighbors`, `path`, and `search` for generic retrieval. Use
`docgraph query <name> --arg name=value` for a human-readable typed table, or add
`--json` for the stable query/columns/rows envelope. Query-backed repository commands
use the same two output modes. Use `docgraph outline <entity>` to enumerate stable
section IDs, headings, levels, parents, and line spans before reading a long document.
`docgraph get <stable-section>` returns 40 content lines by default; select another
positive limit with `--lines` or opt into the complete body with `--all`. Structured
results distinguish explicit relations, Markdown links, and search matches.

Repository rules in `.docgraph/logic.dl` may call these built-in predicates. The
argument names also appear in `docgraph describe` and `docgraph describe --all`:

```text
entity[id]
entity_type[id, type]
entity_state[id, state]
entity_property_string[id, key, value]
entity_property_integer[id, key, value]
entity_property_float[id, key, value]
entity_property_boolean[id, key, value]
entity_property_datetime[id, key, value]
relation[source, predicate, target]
relation_property_string[source, predicate, target, key, value]
relation_property_integer[source, predicate, target, key, value]
relation_property_float[source, predicate, target, key, value]
relation_property_boolean[source, predicate, target, key, value]
relation_property_datetime[source, predicate, target, key, value]
section[id, document, heading]
document[path]
external_entity[id]
external_entity_provider[id, provider]
external_entity_kind[id, kind]
external_entity_state[id, state]
external_entity_title[id, title]
external_entity_url[id, url]
external_entity_freshness[id, freshness]
external_entity_attribute[id, key, value]
```

Property predicates preserve the declared scalar type. An array uses the predicate
for its declared item type and contributes one fact per member. For example,
`entity_property_string(Id, "labels", Label)` enumerates a string array, while
`!entity_property_string(Id, "labels", _)` tests that it has no members after `Id`
has been grounded. Join a string member that contains an entity ID to `entity/1` to
make the query output entity-valued:

```text
related(Source, Target) :- entity_property_string(Source, "related", Target), entity(Target).
```
