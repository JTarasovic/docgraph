+++

id = "task:align-local-and-ci-checks"
type = "task"
state = "done"

[properties]
title = "Align local and CI checks"

[[relations]]
type = "part_of"
target = "plan:harden-delivery-integrity"

[[relations]]
type = "implements"
target = "plan:harden-delivery-integrity#s-KVJAZZB2NX"

[docgraph_generated]
schema_version = 1

[[docgraph_generated.incoming]]
source = "task:dogfood-validation-action"
predicate = "depends_on"
target = "task:align-local-and-ci-checks"

[[docgraph_generated.inverses]]
source = "task:align-local-and-ci-checks"
type = "required_by"
target = "task:dogfood-validation-action"

+++
<a id="s-40TXJNNT2Y"></a>
# Align local and CI checks

Address [#17](https://github.com/JTarasovic/docgraph/issues/17) by making the
repository's check contract explicit and executable from one maintained entry point.
Inventory every CI-only and local-only check, including dependency policy, lockfile
behavior, change-base selection, native runtime setup, and operating-system coverage.

The local aggregate and Linux CI must call the same shared check definitions. The
more expensive Windows runner should execute only the shared tests that provide
cross-platform coverage, through an explicitly named overlay. Add a cheap consistency
test or generated workflow boundary so future edits cannot drift unnoticed.

<a id="s-NT87MWYWC1"></a>
## Acceptance

- One documented local command runs every platform-independent required check.
- Linux CI invokes that contract; Windows CI invokes only a clearly named test overlay.
- Dependency policy and locked-dependency behavior are consistent in both places.
- A regression test detects meaningful task/workflow divergence.
- Documentation states which CI behavior cannot be reproduced on one developer host.

<a id="s-3MHC3Z5E16"></a>
## Implementation

Implemented with `mise run check` as the single platform-independent contract for
formatting, dependency-only Clippy linting exclusion, locked tests, managed-change
validation, dependency policy, and unused-dependency detection. `mise run check-local`
prepares the pinned Linux or Windows logic runtime before invoking that same contract.
Linux CI obtains the checksum-verified packaged runtime, selects an explicit change
base, and invokes the complete contract.

The path-filtered Windows workflow runs in parallel only for Rust, fixture, action,
mise, skill, or native-runtime changes. Its shallow checkout and selective mise install
feed the shared locked test task through `windows-e2e`; formatting, Clippy, dependency
policy, and managed-change validation remain on Linux.

`repository_check_contract.rs` parses `mise.toml` and both workflows to keep those job
roles synchronized. The README documents the host/CI boundary. On Windows,
`mise run check-local` passed all six checks and 139 tests.
