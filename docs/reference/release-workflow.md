+++

id = "reference:release-workflow"
type = "reference"

[properties]
role = "design"

[docgraph_generated]
schema_version = 1

[[docgraph_generated.incoming]]
source = "task:define-repeatable-release-workflow"
predicate = "implements"
target = "reference:release-workflow"

[[docgraph_generated.inverses]]
source = "reference:release-workflow"
type = "implemented_by"
target = "task:define-repeatable-release-workflow"

+++
<a id="s-VZ0QKNRXQK"></a>
# Release workflow

This runbook defines the contributor workflow selected in
[decision:release-workflow-ownership](../decisions/release-workflow-ownership.md).
The checked-in cargo-release, git-cliff, and dist configuration owns preparation and
distribution; the repository keeps only its product-specific companion-payload staging
and smoke-test seams.

<a id="s-Q9JPZDVN2R"></a>
## Release contract

Docgraph publishes one x86-64 Windows archive and one x86-64 Linux archive from an
immutable vMAJOR.MINOR.PATCH tag. Each contains the CLI, matching native logic runtime,
project and third-party licenses, README, and portable skill. Each archive has a
SHA-256 checksum and passes a clean-install smoke test on its native runner.

Cargo.toml workspace.package.version is the only canonical checked-in version.
Cargo-release derives Cargo.lock and the portable skill's cli_version. Dist derives
the target matrix, archive names, checksums, release manifest, workflow, and GitHub
release from that version and the tag. Documentation examples are generic; CI resolves
the published version it is testing.

<a id="s-5DGJ989FQ4"></a>
## Preconditions

- Start a release branch from current origin/main with a clean working tree.
- Choose an explicit version greater than the latest stable tag under Semantic
  Versioning and review all changes since that tag for user impact and security notes.
- Confirm the pinned Windows and Linux companion-runtime artifacts exist, their
  configured checksums verify, and their producer attestations and Syft CycloneDX
  SBOMs are present. Companion binaries are rebuilt only by the manually dispatched
  native workflow when their source or build recipe changes, not by ordinary CI. Their
  immutable release names identify both the upstream Souffle revision and the docgraph
  producer commit.
- Install the exact mise-managed Rust, cargo-release, git-cliff, dist,
  cargo-auditable, and cargo-cyclonedx versions.
- Authenticate gh for the repository and confirm required Linux and Windows checks can
  run on the release branch.

<a id="s-A4SX0BRFKM"></a>
## Prepare and rehearse

Use these commands, with 0.3.0 replaced by the intended numeric version:

    cargo release 0.3.0 --workspace
    cargo release 0.3.0 --workspace --execute
    bash tools/release/stage-dist-inputs.sh
    dist plan --tag v0.3.0
    dist build --tag v0.3.0 --target <current-host-target>

The first cargo-release command is a non-mutating preview and is always run first. Its
preview must show only Cargo.toml, Cargo.lock, skills/docgraph/skill.toml, and the
git-cliff changelog proposal. The execute command creates one consolidated preparation
commit but neither a tag nor a push. Review and edit CHANGELOG.md for user impact,
amend that commit, then run the repository checks.

Dist plan must show exactly the supported native targets, archives, SHA-256 outputs,
release manifest, cargo-cyclonedx workspace SBOM, and GitHub attestation work. The
staging command downloads and verifies the current host's pinned companion runtime and
lays out the portable skill and third-party notices for dist. Dist build consumes those
inputs and builds the archive. Run tools/release/smoke-test.sh against the resulting
archive. A local rehearsal proves only the current platform; the release pull request
must exercise the other native runner. The native companion's separately attested Syft
SBOM describes the Souffle payload and its shipped licenses; cargo-cyclonedx describes
the Rust workspace. Cargo-auditable remains disabled because dist 0.32's generated
installer for it is not version-pinned. Binary-embedded dependency metadata is not part
of this release contract; the two attested CycloneDX inventories are authoritative.

<a id="s-PPK9N1DX91"></a>
## Review the preparation commit

The release pull request must contain no unrelated work. Verify:

1. Cargo metadata, Cargo.lock, the portable skill, and the intended tag agree.
2. CHANGELOG.md has a curated dated entry and correct comparison links.
3. README and action examples contain no release-owned current-version strings.
4. Dist's generated workflow matches dist-workspace.toml and uses pinned actions,
   tools, companion inputs, and runner images.
5. The native archive contains the CLI, runtime, licenses, README, and skill and passes
   the clean-install smoke test.
6. mise run check-local, docgraph validate --changes origin/main, and docgraph review
   origin/main pass.

Merge only after required Linux and Windows checks are green.

<a id="s-H8CPAF9ZRS"></a>
## Publish

After the preparation commit is merged, update local main with a fast-forward, confirm
the tree is clean, and create the reviewed annotated tag:

    git fetch origin --tags
    git switch main
    git pull --ff-only origin main
    git status --short
    git tag -a v0.3.0 -m "docgraph v0.3.0"
    git show --stat v0.3.0
    git push origin v0.3.0

Replace 0.3.0 with the reviewed version. Do not push unless git show identifies the
merged preparation commit. The tag starts the generated dist workflow; its host phase
publishes only after both native build and smoke-test jobs succeed.

<a id="s-BW2KSCPQFH"></a>
## Verify published evidence

Checksums detect corruption after download but do not identify who produced a file.
GitHub attestations bind each file's digest to this repository, its exact producer
workflow, and a source ref or commit. An SBOM inventories the shipped components; it
does not assert that they are vulnerability-free. The Rust workspace and native
companion have separate CycloneDX SBOMs because cargo-cyclonedx cannot describe the
Souffle binary or its bundled licenses.

For a pinned companion, copy the release, archive, and full producer revision from
`tools/logic-runtime/sources.toml`, then verify all three published subjects:

    repository=JTarasovic/docgraph
    release=logic-runtime-linux-a1303be3-d85140ef
    archive=docgraph-logic-runtime-linux-x86_64-a1303be3-d85140ef.tar.gz
    producer=d85140ef7c6369ff003a90d4adc860c8c77484e7
    mkdir companion-evidence && cd companion-evidence
    gh release download "$release" --repo "$repository" \
      --pattern "$archive" --pattern "$archive.sha256" \
      --pattern "$archive.cdx.json"
    sha256sum --check --strict "$archive.sha256"
    for subject in "$archive" "$archive.sha256" "$archive.cdx.json"; do
      gh attestation verify "$subject" --repo "$repository" \
        --signer-workflow JTarasovic/docgraph/.github/workflows/logic-runtime.yml \
        --source-digest "$producer" --source-ref refs/heads/main \
        --deny-self-hosted-runners
    done
    jq --exit-status \
      '.bomFormat == "CycloneDX" and (.components | length > 0)' \
      "$archive.cdx.json"

Use the Windows release and `.zip` archive values from the adjacent source section to
verify the Windows companion identically.

For a docgraph release, download the complete evidence set, verify both checksum
layers, then verify every archive, adjacent checksum, workspace SBOM, and unified
checksum as an attestation subject:

    repository=JTarasovic/docgraph
    tag=v0.3.1
    mkdir docgraph-evidence && cd docgraph-evidence
    gh release download "$tag" --repo "$repository" \
      --pattern '*.tar.gz' --pattern '*.tar.gz.sha256' \
      --pattern '*.zip' --pattern '*.zip.sha256' \
      --pattern '*.cdx.xml' --pattern sha256.sum
    for checksum in *.sha256 sha256.sum; do
      grep --invert-match '^$' "$checksum" | sha256sum --check
    done
    for subject in *.tar.gz *.tar.gz.sha256 *.zip *.zip.sha256 *.cdx.xml sha256.sum; do
      gh attestation verify "$subject" --repo "$repository" \
        --signer-workflow JTarasovic/docgraph/.github/workflows/release.yml \
        --source-ref "refs/tags/$tag" --deny-self-hosted-runners
    done
    grep --quiet '<name>docgraph-cli</name>' ./*.cdx.xml

These attestations support a SLSA Build Level 2 claim. They do not establish Level 3:
the current generated workflow does not provide the stronger, separately administered
build definition and isolation required for that claim.

<a id="s-8TZDE68XQA"></a>
## Verify and close out

1. Confirm dist's tag workflow and every native build, smoke, checksum, workspace SBOM,
   and host attestation job succeeded.
2. Download both archives and checksum outputs and verify them independently.
3. Verify GitHub artifact attestations for each archive, checksum manifest, and
   cargo-cyclonedx workspace SBOM. Separately verify each pinned native companion's
   archive, checksum, and Syft SBOM against its producer workflow.
4. In a clean directory, run docgraph --version and docgraph --help from each archive,
   then exercise the documented validation action at the exact published version.
5. Confirm the GitHub release body matches the accepted changelog entry and that CI's
   published-action compatibility check observes the new release dynamically.
6. Close the release issue only after artifacts, checksums, notes, attestations, and
   post-release validation are present.

<a id="s-T1ZYCCJ8W6"></a>
## Rollback and correction

Before a tag is pushed, amend or abandon the release branch normally. A failed
tag-triggered workflow may be rerun against the same immutable inputs. If a fix changes
an input, prepare a new version instead of moving the tag. Delete an unpublished draft
release if necessary, but never replace an externally observed tag or published asset;
correct it with a patch release.

<a id="s-XBXJZW4EST"></a>
## Changelog policy

CHANGELOG.md is canonical release prose in Keep a Changelog structure. It contains an
Unreleased section and one dated heading and comparison link per stable release.
Git-cliff proposes entries from commits since the previous stable tag. The release
author owns categorization, omission of internal-only work, compatibility and security
notes, and final wording. Dist publishes the accepted section instead of independently
generating notes. Repository commits and squash-merge pull-request titles are checked
against the same Conventional Commit policy, so the proposal starts from consistently
typed release-facing subjects rather than relying on release-time cleanup.
