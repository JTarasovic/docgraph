+++

id = "plan:harden-delivery-integrity"
type = "plan"
state = "proposed"

[properties]
title = "Harden delivery integrity"

[[relations]]
type = "implements"
target = "reference:design#s-DHW5KPNDJV"

[[relations]]
type = "implements"
target = "reference:validation-action"

[docgraph_generated]
schema_version = 1

[[docgraph_generated.incoming]]
source = "task:align-local-and-ci-checks"
predicate = "implements"
target = "plan:harden-delivery-integrity#s-KVJAZZB2NX"

[[docgraph_generated.incoming]]
source = "task:align-local-and-ci-checks"
predicate = "part_of"
target = "plan:harden-delivery-integrity"

[[docgraph_generated.incoming]]
source = "task:attest-release-artifacts"
predicate = "implements"
target = "plan:harden-delivery-integrity#s-SBVSRRVQW0"

[[docgraph_generated.incoming]]
source = "task:attest-release-artifacts"
predicate = "part_of"
target = "plan:harden-delivery-integrity"

[[docgraph_generated.incoming]]
source = "task:automate-release-preparation"
predicate = "implements"
target = "plan:harden-delivery-integrity#s-683VPY7SC0"

[[docgraph_generated.incoming]]
source = "task:automate-release-preparation"
predicate = "part_of"
target = "plan:harden-delivery-integrity"

[[docgraph_generated.incoming]]
source = "task:define-repeatable-release-workflow"
predicate = "implements"
target = "plan:harden-delivery-integrity#s-D2EX933DKW"

[[docgraph_generated.incoming]]
source = "task:define-repeatable-release-workflow"
predicate = "part_of"
target = "plan:harden-delivery-integrity"

[[docgraph_generated.incoming]]
source = "task:dogfood-validation-action"
predicate = "implements"
target = "plan:harden-delivery-integrity#s-6R0PYTBY0Y"

[[docgraph_generated.incoming]]
source = "task:dogfood-validation-action"
predicate = "part_of"
target = "plan:harden-delivery-integrity"

[[docgraph_generated.inverses]]
source = "plan:harden-delivery-integrity"
type = "contains"
target = "task:align-local-and-ci-checks"

[[docgraph_generated.inverses]]
source = "plan:harden-delivery-integrity"
type = "contains"
target = "task:attest-release-artifacts"

[[docgraph_generated.inverses]]
source = "plan:harden-delivery-integrity"
type = "contains"
target = "task:automate-release-preparation"

[[docgraph_generated.inverses]]
source = "plan:harden-delivery-integrity"
type = "contains"
target = "task:define-repeatable-release-workflow"

[[docgraph_generated.inverses]]
source = "plan:harden-delivery-integrity"
type = "contains"
target = "task:dogfood-validation-action"

[[docgraph_generated.inverses]]
source = "plan:harden-delivery-integrity#s-683VPY7SC0"
type = "implemented_by"
target = "task:automate-release-preparation"

[[docgraph_generated.inverses]]
source = "plan:harden-delivery-integrity#s-6R0PYTBY0Y"
type = "implemented_by"
target = "task:dogfood-validation-action"

[[docgraph_generated.inverses]]
source = "plan:harden-delivery-integrity#s-D2EX933DKW"
type = "implemented_by"
target = "task:define-repeatable-release-workflow"

[[docgraph_generated.inverses]]
source = "plan:harden-delivery-integrity#s-KVJAZZB2NX"
type = "implemented_by"
target = "task:align-local-and-ci-checks"

[[docgraph_generated.inverses]]
source = "plan:harden-delivery-integrity#s-SBVSRRVQW0"
type = "implemented_by"
target = "task:attest-release-artifacts"

+++
<a id="s-CMG9RSJEZG"></a>
# Harden delivery integrity

<a id="s-G4NTBBFZJY"></a>
## Objective

Make the command developers run locally, the checks enforced in CI, and the artifacts
published from a tag one coherent delivery system. Release preparation should be a
repeatable documented operation rather than a search-and-replace exercise, and every
published archive should be traceable to its source and dependency inventory.

<a id="s-2XHZDW25VH"></a>
## Portfolio priority

Start here, specifically with check parity in #17. Release workflow and action work
follow at the same near-term priority as portable-agent guidance; supply-chain
attestation is deliberately the final slice after the artifact path stabilizes.

<a id="s-7Y7V1NTWH4"></a>
## Report coverage

This plan covers the delivery-system reports as one dependency chain:

- [#17](https://github.com/JTarasovic/docgraph/issues/17): local and CI check parity.
- [#15](https://github.com/JTarasovic/docgraph/issues/15): versioning, changelog,
  release documentation, and release-tool evaluation.
- [#14](https://github.com/JTarasovic/docgraph/issues/14): validation-action dogfooding.
- [#12](https://github.com/JTarasovic/docgraph/issues/12): SBOM and SLSA-oriented
  release provenance.

The v0.2.0 history provides concrete failure evidence. The release-preparation commit
changed five independently versioned surfaces, while commit `c0728fc` was still needed
after the tag to update the validation-action example and CI smoke test. Local
`mise run check` also omitted the dependency-policy checks that CI ran separately and
could not reproduce the Linux job's platform behavior.

<a id="s-8BMH6EEQDG"></a>
## Priority and sequence

1. Establish an explicit parity contract and one authoritative aggregate check.
2. Select and document the release workflow before adding more release automation.
3. Automate version propagation and changelog maintenance from one release input.
4. Exercise both the checked-out action and the last published action at the correct
   layers of CI.
5. Generate and verify provenance and an SBOM for the stabilized artifact pipeline.

<a id="s-T5M7E38GS4"></a>
## Work slices

<a id="s-KVJAZZB2NX"></a>
### Align local and CI checks

Remove accidental command drift while keeping unavoidable operating-system coverage
explicit.

<a id="s-D2EX933DKW"></a>
### Define a repeatable release workflow

Compare the existing scripts with Rust-focused and general release tooling, choose the
smallest suitable ownership boundary, and document the human and automated sequence.

<a id="s-683VPY7SC0"></a>
### Automate release preparation

Drive versioned code, metadata, documentation examples, and a maintained changelog
from one declared release version or generated release change.

<a id="s-6R0PYTBY0Y"></a>
### Dogfood the validation action

Test the action implementation from the checkout and separately verify compatibility
with a published docgraph binary.

<a id="s-SBVSRRVQW0"></a>
### Attest release artifacts and publish an SBOM

Attach verifiable provenance and a standard dependency inventory to every supported
release artifact, with documented consumer verification.

<a id="s-5PBTEZE9CA"></a>
## Completion

All four reports have regression or workflow coverage and are closed or explicitly
split with rationale. A documented release can be rehearsed without publishing,
local checks explain their platform boundary, CI invokes the same authoritative
checks, and release archives have checksums, provenance, and a verifiable SBOM.
