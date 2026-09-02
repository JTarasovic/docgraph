+++

id = "task:attest-release-artifacts"
type = "task"
state = "in_progress"

[properties]
title = "Attest release artifacts and publish an SBOM"

[[relations]]
type = "part_of"
target = "plan:harden-delivery-integrity"

[[relations]]
type = "implements"
target = "plan:harden-delivery-integrity#s-SBVSRRVQW0"

[[relations]]
type = "depends_on"
target = "task:automate-release-preparation"

[docgraph_generated]
schema_version = 1

+++
<a id="s-MKS7YRDWHR"></a>
# Attest release artifacts and publish an SBOM

Address [#12](https://github.com/JTarasovic/docgraph/issues/12) after the dist artifact
pipeline is stable. Target SLSA Build Level 2 through GitHub artifact attestations;
do not claim Level 3 under the current generated workflow. Prove that dist's
attestation subjects cover every archive and checksum manifest. Produce a final
CycloneDX SBOM that combines or augments cargo-cyclonedx output so it includes the Rust
workspace, bundled native runtime, and shipped licenses, then attest that SBOM and bind
it to the exact release artifacts.

Prefer platform-native verification that consumers can reproduce. Document what the
attestation proves, what it does not prove, how checksums relate to it, and how to
verify an archive and its SBOM from the command line.

<a id="s-DNQVMS2H5B"></a>
## v0.3.0 baseline

The public v0.3.0 release proves the native archive path is working: mise verifies the
GitHub API digest and a GitHub artifact attestation before installation, and the
attestation binds the archive and its adjacent checksum to the tagged release workflow.
The published CycloneDX SBOM is populated and identifies `docgraph-cli@0.3.0`, but it is
neither a subject of a GitHub attestation nor included in `sha256.sum`. GitHub and mise
may report successful archive verification when optional SBOM evidence is absent, so
release checks must assert that each required subject exists before accepting a pass.

<a id="s-7G5TTC3D8S"></a>
## Acceptance

- Every supported archive and checksum manifest is covered by a verifiable GitHub
  artifact attestation whose subject digest matches the published file.
- Every release publishes an attested CycloneDX SBOM including Rust and bundled native
  runtime components and licenses.
- `sha256.sum` covers the SBOM as well as every supported archive, and is itself covered
  by release provenance.
- CI verifies attestation subjects, SBOM shape and required components, and documented
  consumer commands before publication. Missing optional evidence must fail rather than
  produce a vacuous green check.
- Consumer-path verification proves both `gh attestation verify` and mise installation
  use the expected archive subject, and `gh attestation verify docgraph-cli.cdx.xml`
  succeeds for the published SBOM.
- Documentation claims SLSA Build Level 2 only while the generated workflow uses the
  current GitHub-hosted builder model, and explains checksums separately from
  provenance.
- Release notes and the runbook link to exact checksum, provenance, and SBOM
  verification instructions.
