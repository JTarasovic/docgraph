+++

id = "task:attest-release-artifacts"
type = "task"
state = "backlog"

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

Address [#12](https://github.com/JTarasovic/docgraph/issues/12) after the artifact
pipeline is stable. Define the promised supply-chain level, generate build provenance
for every published archive, produce a standard SPDX or CycloneDX SBOM that includes
the Rust and bundled native-runtime contents, and bind both to the exact release
artifacts.

Prefer platform-native verification that consumers can reproduce. Document what the
attestation proves, what it does not prove, how checksums relate to it, and how to
verify an archive and its SBOM from the command line.

<a id="s-7G5TTC3D8S"></a>
## Acceptance

- Every supported archive and checksum manifest is covered by signed provenance.
- Every release publishes a machine-readable SBOM including bundled components.
- CI verifies provenance and SBOM shape before publishing and tests consumer commands.
- The documented SLSA claim matches the actual builder isolation and workflow design.
- Release notes and the runbook link to verification instructions.
