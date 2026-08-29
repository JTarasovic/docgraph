+++

id = "decision:first-release-contract"
type = "decision"
state = "accepted"

[properties]
title = "First release contract"

[docgraph_generated]
schema_version = 1

[[docgraph_generated.incoming]]
source = "plan:ship-first-release"
predicate = "implements"
target = "decision:first-release-contract"

[[docgraph_generated.incoming]]
source = "task:define-release-contract"
predicate = "implements"
target = "decision:first-release-contract"

[[docgraph_generated.inverses]]
source = "decision:first-release-contract"
type = "implemented_by"
target = "plan:ship-first-release"

[[docgraph_generated.inverses]]
source = "decision:first-release-contract"
type = "implemented_by"
target = "task:define-release-contract"

+++
<a id="s-CCSMJX86Y3"></a>
# First release contract

<a id="s-7FVD4XVWYP"></a>
## Context

The first public release should be easy to install and honest about what this pre-1.0 solo project has actually exercised, without turning release preparation into a packaging program.

<a id="s-NQNQAGNQ8P"></a>
## Decision

- Release `v0.1.0` through GitHub Releases rather than crates.io or an operating-system package manager.
- Provide Windows x86-64 and Linux x86-64 archives.
- Put `docgraph` and its opaque `docgraph-logic-runtime` companion beside each other in each archive, along with the project and required third-party licenses.
- Require no separate Soufflé installation in the normal path; retain `DOCGRAPH_LOGIC_RUNTIME` as an advanced override.
- Publish SHA-256 checksums and ensure the release tag matches the compiled version.
- Treat all compatibility as best-effort until 1.0. Document meaningful changes and fail clearly on recognized incompatible schemas or index formats, but make no pre-1.0 stability guarantee.
- Do not publish macOS or ARM artifacts initially.
- Accept an artifact only after it completes the representative workflow in a clean environment without Rust, mise, a repository checkout, or a preinstalled logic runtime.

<a id="s-1Y148BN1C4"></a>
## Consequences

CI may build Linux on Ubuntu without making Ubuntu itself part of the public contract. Broader packaging, installers, additional targets, signing, and package registries remain follow-up work driven by actual demand.
