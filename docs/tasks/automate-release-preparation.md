+++

id = "task:automate-release-preparation"
type = "task"
state = "backlog"

[properties]
title = "Automate release preparation"

[[relations]]
type = "part_of"
target = "plan:harden-delivery-integrity"

[[relations]]
type = "implements"
target = "plan:harden-delivery-integrity#s-683VPY7SC0"

[[relations]]
type = "depends_on"
target = "task:define-repeatable-release-workflow"

[docgraph_generated]
schema_version = 1

[[docgraph_generated.incoming]]
source = "task:attest-release-artifacts"
predicate = "depends_on"
target = "task:automate-release-preparation"

[[docgraph_generated.inverses]]
source = "task:automate-release-preparation"
type = "required_by"
target = "task:attest-release-artifacts"

+++
<a id="s-HM8SD8NKXN"></a>
# Automate release preparation

Implement the cargo-release, git-cliff, and dist workflow selected for
[#15](https://github.com/JTarasovic/docgraph/issues/15). Pin the tools in mise and add
release.toml, cliff.toml, and dist-workspace.toml. Make the workspace package version
the authoritative checked-in input. Configure cargo-release to update Cargo.lock and
the CLI package's portable-skill compatibility value and to invoke git-cliff for one
reviewable preparation commit without publishing or tagging.

Replace the authored release workflow with dist's checked-in generated workflow and a
drift check. Add only the small platform hook needed to stage the pinned native logic
runtime, licenses, and portable skill, then run the existing clean-install smoke test
against dist's archive. Adopt dist's standard asset names and update the validation
action or other consumers to select exact platform assets without constructing the
legacy names. Replace README release-number edits with stable latest-release links or
generic examples where an exact version is not part of the contract.

Create the repository changelog from the accepted policy. A preparation run must be
idempotent, reviewable before commit, and able to fail before a tag is created when a
versioned surface, generated workflow, or packaged payload is inconsistent.

<a id="s-Z8PZQEQPR2"></a>
## Acceptance

- Pinned cargo-release and git-cliff configuration accepts one intended version and
  previews or creates the expected release commit without publishing, pushing, or
  tagging.
- Cargo.toml, Cargo.lock, skill compatibility metadata, and CHANGELOG.md agree
  automatically where exact versions are required; README and examples require no
  ordinary release edit.
- Pinned dist configuration generates the checked-in workflow reproducibly, and CI
  rejects configuration or generated-workflow drift.
- Dist plans native x86-64 Windows and Linux archives, stages every required companion
  payload, produces SHA-256 outputs, and passes the existing clean-install smoke test.
- Release tools, GitHub Actions, companion downloads, and runner images are pinned;
  dry-run and CI checks fail before tag creation on stale or contradictory inputs.
