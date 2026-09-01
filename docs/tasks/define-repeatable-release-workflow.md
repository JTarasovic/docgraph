+++

id = "task:define-repeatable-release-workflow"
type = "task"
state = "backlog"

[properties]
title = "Define a repeatable release workflow"

[[relations]]
type = "part_of"
target = "plan:harden-delivery-integrity"

[[relations]]
type = "implements"
target = "plan:harden-delivery-integrity#s-D2EX933DKW"

[docgraph_generated]
schema_version = 1

[[docgraph_generated.incoming]]
source = "task:automate-release-preparation"
predicate = "depends_on"
target = "task:define-repeatable-release-workflow"

[[docgraph_generated.inverses]]
source = "task:define-repeatable-release-workflow"
type = "required_by"
target = "task:automate-release-preparation"

+++
<a id="s-YPVW6WP614"></a>
# Define a repeatable release workflow

Address the design portion of
[#15](https://github.com/JTarasovic/docgraph/issues/15). Audit v0.2.0 and commit
`c0728fc`, enumerate every release-owned version and artifact, and decide which are
canonical, generated, dynamically linked, or unnecessary.

Compare the current PowerShell/GitHub Actions pipeline against suitable tools rather
than selecting the first named option. At minimum assess cargo-dist for binary
distribution, release-plz for Rust version/changelog preparation, GoReleaser as a
general release orchestrator, and a smaller retained-script design. Evaluate native
runtime bundling, Windows/Linux parity, checksums, dry runs, generated workflow
ownership, changelog support, provenance integration, maintenance cost, and failure
recovery.

Record the decision and add a contributor-facing release runbook that covers
preconditions, rehearsal, version selection, tag creation, verification, rollback,
and post-release checks.

<a id="s-P86CTH9CRD"></a>
## Acceptance

- The v0.2.0 audit accounts for every manual edit and the missed post-tag change.
- Tool evaluation uses written project-specific criteria and records rejected options.
- One canonical release-version input and every derived consumer are identified.
- A release runbook supports a no-publish rehearsal and a real release.
- The selected design has an explicit changelog policy and ownership boundary.
