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

The local aggregate and CI must call the same shared check definitions. Platform-only
jobs may add an explicitly named overlay, but they must not silently replace or omit
the shared contract. Add a cheap consistency test or generated workflow boundary so
future edits cannot drift unnoticed.

<a id="s-NT87MWYWC1"></a>
## Acceptance

- One documented local command runs every platform-independent required check.
- Linux and Windows CI invoke that same contract plus clearly named platform overlays.
- Dependency policy and locked-dependency behavior are consistent in both places.
- A regression test detects meaningful task/workflow divergence.
- Documentation states which CI behavior cannot be reproduced on one developer host.

<a id="s-3MHC3Z5E16"></a>
## Implementation

Implemented with `mise run check` as the single platform-independent contract for
formatting, lint, locked tests, managed-change validation, dependency policy, and
unused-dependency detection. `mise run check-local` prepares the pinned Linux or
Windows logic runtime before invoking that same contract. CI obtains the corresponding
checksum-verified packaged runtime, selects an explicit change base, and invokes the
contract unchanged in both jobs.

`repository_check_contract.rs` parses `mise.toml` and the CI workflow to keep the
required task set, both job invocations, full-history checkout, and change-base
selection synchronized. The README documents the host/CI boundary. On Windows,
`mise run check-local` passed all six checks and 139 tests.
