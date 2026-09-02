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
do not claim Level 3 under the current generated workflow. Prove that dist's host-phase
attestation covers every published archive, checksum, and cargo-cyclonedx workspace
SBOM. Produce a separate Syft CycloneDX SBOM when each native Souffle companion is
built, and attest the native binary plus its published archive, checksum, and SBOM.

Prefer platform-native verification that consumers can reproduce. Document what the
attestation proves, what it does not prove, how checksums relate to it, and how to
verify an archive and its SBOM from the command line.

<a id="s-DNQVMS2H5B"></a>
## v0.3.0 baseline

The public v0.3.0 release proves the native archive path is working: mise verifies the
GitHub API digest and a GitHub artifact attestation before installation, and the
attestation binds the archive and its adjacent checksum to the tagged release workflow.
The published cargo-cyclonedx SBOM is populated and identifies `docgraph-cli@0.3.0`,
but it is not an attestation subject. The older immutable Souffle companion releases
have checksums but no producer SBOM or provenance. GitHub and mise may report successful
archive verification when optional SBOM evidence is absent, so release checks must
assert that each required subject exists before accepting a pass.

<a id="s-7G5TTC3D8S"></a>
## Acceptance

- The manually dispatched companion workflow builds Linux and Windows Souffle binaries
  on their native runners, executes the same real Datalog smoke program on both, and
  fans their outputs into one Linux evidence and publication job.
- Each new immutable companion release publishes an archive, adjacent SHA-256 checksum,
  and Syft CycloneDX SBOM. GitHub attestations cover the native executable and every
  published companion file.
- Every dist release publishes its cargo-cyclonedx workspace SBOM, and the final host
  phase attests every archive, adjacent checksum, workspace SBOM, and `sha256.sum` after
  all native builds exist.
- `sha256.sum` covers the dist archives, while each companion's adjacent checksum covers
  its archive. The SBOMs and checksum manifests are authenticated directly as
  attestation subjects instead of relying on a custom combined checksum or SBOM.
- CI verifies attestation subjects, SBOM shape and required components, and documented
  consumer commands before publication. Missing optional evidence must fail rather than
  produce a vacuous green check.
- Consumer-path verification proves both `gh attestation verify` and mise installation
  use the expected archive subject, and verifies the published Rust and native-runtime
  SBOM subjects explicitly.
- Documentation claims SLSA Build Level 2 only while the generated workflow uses the
  current GitHub-hosted builder model, and explains checksums separately from
  provenance.
- Release notes and the runbook link to exact checksum, provenance, and SBOM
  verification instructions.

The existing companion tags remain immutable. Completing this task therefore requires
publishing a newly named companion generation whose identity includes both the upstream
Souffle revision and producer commit, updating `sources.toml` to its verified digests,
and making release staging require the companion attestation and SBOM before the first
hardened product release.

<a id="s-WC3P93NNW5"></a>
## Published companion evidence

The `logic-runtime-linux-a1303be3-d85140ef` and
`logic-runtime-windows-a1303be3-d85140ef` releases were produced by successful run
`33661406059` from commit `d85140ef7c6369ff003a90d4adc860c8c77484e7`.
Their archives, adjacent checksums, and Syft CycloneDX SBOMs all verify against the
`logic-runtime.yml` signer. The pinned digests in `sources.toml` cover each published
file and each extracted runtime binary. Release staging rejects a missing or malformed
checksum or SBOM, while required CI and pre-publication smoke jobs additionally verify
all three attestations against the exact producer commit.

The remaining proof is the first docgraph release from this configuration. Its host
job must attest both platform archives, their adjacent checksums, the cargo-cyclonedx
workspace SBOM, and `sha256.sum`; post-publication verification must then exercise the
documented GitHub CLI and mise consumer paths.
