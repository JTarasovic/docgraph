# Souffle spike

## Evidence

- Official 2.5 release packages omit Windows, but upstream has active MSVC CI.
- A native source build succeeded without WSL or global installs. It used Souffle
  `a1303be3c0166400dee3d1f36f0d96abe03e6901`, VS 2026/MSVC, bundled CMake and
  Ninja, local winflexbison 2.5.25, and vcpkg
  `cd61e1e26a038e82d6550a3ebbe0fbbfe7da78e3` with static SQLite 3.53.2.
- The minimal 64-bit Release build enables only SQLite and produces an 8,532,480
  byte companion executable (SHA-256
  `769468c7d3e55ed4842d5ee57c42bb742c3a4fda07d8d375cc7d38056280b50b`). Its
  only PE imports are `KERNEL32.dll` and `SHELL32.dll`; SQLite and the MSVC CRT
  are statically linked.
- `souffle --version` reports a 64-bit SQLite-enabled runtime. A recursive
  SQLite-input closure returned `a-b`, `a-c`, and `b-c`. The synthetic
  docgraph named query also returned `github:issue:owner/repo:123` through that
  native executable.
- Upstream Windows CI exercises SQLite only with relative database names and a
  `file:` URI. It does not test an absolute Windows input path. Souffle treats
  `C:/...` as relative, so docgraph now supplies `file:///C:/...` on Windows.

## What the spike implements

`docgraph-logic` replaces the Cozo crate. It reads `.docgraph/logic.dl`, rejects
engine directives/includes/components/pragmas/types/custom functors, creates an
isolated SQLite database with generated built-in relation tables and views,
injects Souffle declarations/input directives/output relation, and kills the
direct helper process after five seconds. Scratch files are deleted on every
return path.

The temporary database proves the connector shape, not integration with
docgraph's durable derived index. Properties are serialized as strings. The
wrapper infers types from named query signatures; intermediate predicates are
symbols. The spike does not yet prove full typed, native end-to-end execution.

## Result

Native Windows is feasible and is not an architectural blocker for v0. Because
upstream does not publish a Windows artifact, docgraph publishes the pinned build as
the opaque `docgraph-logic-runtime` companion with provenance and licence notices.
Windows CI downloads and verifies that package, then runs the end-to-end suite
without rebuilding Soufflé. The generic `DOCGRAPH_LOGIC_RUNTIME` override is for
development and tests. The source pin above is a known-good upstream commit, not a
final product-version choice; maintain an explicit revision policy before release.

Compared with Mnestic, Souffle removes the Rust graph-engine dependency but adds
an external native runtime and release pipeline. Compared with Z3, it is a
closer fit for recursive Datalog and has a verified native SQLite input path.
