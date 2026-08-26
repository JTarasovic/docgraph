# Souffle spike

## Evidence

- The current official release is 2.5 (`5682a9f`, 2025-03-24), under UPL-1.0.
- Its release page provides Fedora, Oracle Linux, and Ubuntu packages. It provides
  no Windows executable or installer.
- The official Ubuntu 22.04 package ran under this machine's existing WSL2 Ubuntu
  after extraction only (no installation). `souffle --version` reported 2.5 with
  SQLite enabled.
- A recursive `path` program read `_edge`/`edge` tables and views from SQLite via
  the official `IO=sqlite` connector and produced `a-b`, `a-c`, and `b-c`.

## What the spike implements

`docgraph-logic` replaces the Cozo crate. It reads `.docgraph/logic.dl`, rejects
engine directives/includes/components/pragmas/types/custom functors, creates an
isolated SQLite database with generated built-in relation tables and views, injects
Souffle declarations/input directives/output relation, and kills the direct helper
process after five seconds. Scratch files are deleted on every return path.

The temporary database proves the connector shape, not integration with docgraph's
durable derived index. Properties are serialized as strings. The wrapper infers
types from named query signatures; intermediate predicates are symbols. The spike
does not yet prove full typed, native end-to-end execution.

## Result

No-go for v0 as a Windows backend. The engine and SQLite connector are real, but
the distribution problem is real too: this repository needs a supported native
Windows executable, a reproducible build/packaging pipeline, and process-tree
cancellation before it can replace an embedded engine. WSL execution is useful
evidence only; it is not a product dependency.

Compared with Mnestic, Souffle removes the Rust graph-engine dependency but adds
an external native runtime and packaging burden. Compared with Z3, it is a closer
fit for recursive Datalog and has the tested SQLite input path, but has the same
external-binary/cross-platform integration class of risk.
