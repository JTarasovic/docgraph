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

Implement the workflow selected for
[#15](https://github.com/JTarasovic/docgraph/issues/15). Make the workspace package
version or an equally explicit release input authoritative, then generate or validate
all required consumers. Replace README release-number edits with stable latest-release
links or badges where an exact version is not part of the user contract.

Maintain a repository changelog from the accepted policy. A preparation run must be
idempotent, reviewable before commit, and able to fail before a tag is created when a
versioned surface or packaged payload is inconsistent.

<a id="s-Z8PZQEQPR2"></a>
## Acceptance

- Release preparation accepts one intended version and does not require manual global
  search-and-replace.
- Cargo metadata, lockfile state, action examples, skill compatibility metadata, and
  package validation agree automatically where exact versions are required.
- README installation links do not require an edit for each ordinary release.
- `CHANGELOG.md` has an unreleased section and deterministic release entries.
- A dry run and CI check catch stale or contradictory versioned surfaces before tag.
