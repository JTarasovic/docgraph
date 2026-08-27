# Workflows

Inspect configured states and edges with `docgraph describe workflow <name>`. Change
state with `docgraph transition <entity> <state> [--dry-run]`; derived facts update
through queries and are not additional canonical state changes.

After adding a workflow to an existing entity type, run `docgraph workflow initialize
<entity-type> [--dry-run]` to materialize all missing initial states atomically.
