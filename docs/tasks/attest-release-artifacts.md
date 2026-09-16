+++

id = "task:attest-release-artifacts"
type = "task"
state = "done"

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

[[relations]]
type = "depends_on"
target = "task:simplify-release-automation"

[docgraph_generated]
schema_version = 1

[[docgraph_generated.backlinks]]
source = "task:simplify-release-automation#s-559RWSY97B"
target = "docs/tasks/attest-release-artifacts.md"

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
- The companion workflow checks its exact evidence set before publication. For the
  product release, verify the complete published evidence set and its attestation
  subjects after publication; the generated dist workflow cannot require an exact
  set of product attestation subjects before creating the GitHub Release.
- Consumer-path verification proves `gh attestation verify` and the public installation
  action use the expected archive subject, and verifies the published Rust and
  native-runtime SBOM subjects explicitly.
- Documentation claims SLSA Build Level 2 only while the generated workflow uses the
  current GitHub-hosted builder model, and explains checksums separately from
  provenance.
- Release notes and the runbook link to exact checksum, provenance, and SBOM
  verification instructions.

The existing companion tags remain immutable. The newly named companion generation
includes both the upstream Souffle revision and producer commit. `sources.toml` pins
its verified digests. The public runtime action verifies the companion archive and
checksum during release staging; the companion producer checks and attests its SBOM.

<a id="s-WC3P93NNW5"></a>
## Published companion evidence

The `logic-runtime-linux-a1303be3-d85140ef` and
`logic-runtime-windows-a1303be3-d85140ef` releases were produced by successful run
`33661406059` from commit `d85140ef7c6369ff003a90d4adc860c8c77484e7`.
Their archives, adjacent checksums, and Syft CycloneDX SBOMs all verify against the
`logic-runtime.yml` signer. The public runtime action pins that release identity and producer commit. It verifies
the archive attestation and checksum before installation; the producer workflow
verifies and attests the SBOM separately.

**Product release evidence and accepted limitation.**

The [v0.4.1 release workflow](https://github.com/JTarasovic/docgraph/actions/runs/34962720140)
succeeded on September 15, 2026. The release contains both platform archives, their
adjacent SHA-256 files, `docgraph-cli.cdx.xml`, and `sha256.sum`. Both archive hashes
match their adjacent files and `sha256.sum`. All six files verify with
`gh attestation verify` against the tagged release workflow and source commit
`260680252b5c5adda469c886359b441324b2e4e0`. The workspace CycloneDX SBOM
identifies `docgraph-cli` version 0.4.1 and contains dependency components. The
[Linux](https://github.com/JTarasovic/docgraph/actions/runs/34962715184) and
[Windows](https://github.com/JTarasovic/docgraph/actions/runs/34962715164) CI runs
exercise the public installation and validation actions with that release.

The generated dist 0.32.0 workflow attests files matching configured globs but does
not assert that every expected product file exists. A missing optional file, especially
the workspace SBOM, could therefore leave a green release job with incomplete evidence.
The available generated workflow hooks do not place an exact-set check between the
complete artifact download and GitHub Release creation. We accept this narrow gap and
verify the published subject set after release; it is not a claim that future releases
automatically fail when an optional evidence file is absent.
