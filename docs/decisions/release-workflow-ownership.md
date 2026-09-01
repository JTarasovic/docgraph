+++

id = "decision:release-workflow-ownership"
type = "decision"
state = "accepted"

[properties]
title = "Release workflow ownership"

[docgraph_generated]
schema_version = 1

[[docgraph_generated.incoming]]
source = "task:define-repeatable-release-workflow"
predicate = "implements"
target = "decision:release-workflow-ownership"

[[docgraph_generated.inverses]]
source = "decision:release-workflow-ownership"
type = "implemented_by"
target = "task:define-repeatable-release-workflow"

[[docgraph_generated.backlinks]]
source = "reference:release-workflow#s-VZ0QKNRXQK"
target = "docs/decisions/release-workflow-ownership.md"

+++
<a id="s-AG47JQZ774"></a>
# Release workflow ownership

<a id="s-8ZF2XGE6H3"></a>
## Context

The v0.2.0 release exposed two problems: preparation was an undocumented global
replacement exercise, while packaging and publication were implemented independently
at every layer. That is avoidable ownership, not evidence that docgraph needs a custom
release system.

Commit ddde1e9 made twelve version replacements: one canonical workspace version in
Cargo.toml, four Cargo-derived entries in Cargo.lock, one required portable-skill
compatibility value, four illustrative README values, and two illustrative validation
action values. Commit c0728fc then made three missed replacements after the tag: two CI
references to the previous published action and one illustrative action.yml value.
Only the workspace version is canonical. Cargo.lock and the skill compatibility value
are derived. Documentation examples should be generic or dynamically linked, and CI
should discover the published version it is intended to test.

The release design must preserve native Windows and Linux builds, bundle the matching
logic runtime and licenses, provide a safe rehearsal, and establish a credible path to
the provenance and SBOM requirements in issue 12.

<a id="s-JT2J0YMRC5"></a>
## Evaluation criteria

Options were compared on their ability to:

1. handle a virtual Cargo workspace with one product version;
2. bundle the CLI, native logic runtime, licenses, README, and portable skill;
3. build and smoke-test Windows and Linux artifacts on native runners;
4. generate archives, SHA-256 checksums, release notes, and a GitHub release;
5. rehearse without creating a tag or release;
6. generate and verify a managed workflow rather than duplicate orchestration;
7. support provenance, SBOM generation, and pinned supply-chain inputs; and
8. keep the pre-tag failure boundary recoverable.

<a id="s-RBV2886ESZ"></a>
## Proof of fit

A disposable spike used dist 0.32.0 and cargo-auditable 0.7.5. Dist planned native
x86-64 Windows MSVC and x86-64 Linux GNU jobs, zip and tar.gz archives, SHA-256 files,
a unified checksum manifest, CycloneDX output, GitHub publication, and artifact
attestations. Its generated workflow also separates configuration from generated
orchestration.

The spike staged the existing Windows companion runtime, portable skill, and
third-party licenses as dist include inputs. Dist built the actual Windows archive;
its manifest contained docgraph.exe, docgraph-logic-runtime.exe, LICENSE, README.md,
skills, and THIRD_PARTY_LICENSES. The existing clean-install smoke-test script passed
against that archive unchanged. The product-specific seam is therefore a small staging
hook plus the existing archive smoke test, not a parallel packaging pipeline.

A second spike used cargo-release 1.1.5 with an explicit 0.3.0 input. Its default dry
run previewed the workspace and portable-skill changes. An isolated execute run with
publishing, pushing, tagging, and verification disabled changed only Cargo.toml,
Cargo.lock, and skills/docgraph/skill.toml and created one consolidated preparation
commit. The skill replacement must be package-scoped to docgraph-cli because
workspace-level replacement rules run once relative to every crate.

<a id="s-ZP8FHV8Q5Q"></a>
## Options considered

<a id="s-FQ2RWH3W3Q"></a>
### dist

[dist](https://axodotdev.github.io/cargo-dist/book/) is selected for release planning,
native builds, archives, checksums, the generated GitHub Actions workflow, GitHub
release publication, baseline CycloneDX generation, and GitHub artifact attestations.
The spike demonstrated the required non-Cargo payload and current smoke-test contract.

<a id="s-7BVDVR0T75"></a>
### cargo-release and git-cliff

[cargo-release](https://github.com/crate-ci/cargo-release) is selected for an explicit
operator-supplied workspace version, dry-run preview, Cargo metadata and lockfile
updates, the package-scoped portable-skill replacement, and the consolidated
preparation commit. [git-cliff](https://git-cliff.org/) is selected to propose the
checked-in changelog entry through cargo-release's pre-release hook. Humans remain
responsible for the final user-facing wording.

<a id="s-AKG0GV13SC"></a>
### release-plz

[release-plz](https://release-plz.dev/docs/) is not selected. Its release-PR and
version-inference model is useful, but this project wants an explicit version decision
and a separately reviewed preparation commit. Restricting release-plz to that role
would overlap cargo-release, while letting it publish would overlap dist.

<a id="s-FSGCTTXE02"></a>
### GoReleaser

[GoReleaser](https://www.goreleaser.com/) is not selected. It can model hooks and extra
files, but adds a separate Rust build and cross-compilation model while dist directly
supports Cargo workspaces, native GitHub runners, checksums, attestations, and generated
workflow ownership.

<a id="s-YCH7Y1VBR2"></a>
### Repository-owned orchestration

Keeping package.ps1 and an authored release workflow as the primary system is not
selected. Its apparent simplicity is the cost of maintaining version propagation,
archive naming, checksums, matrices, publication, provenance, and SBOM coordination
ourselves. The existing smoke test remains valuable because it verifies docgraph's
installed behavior; the packaging script becomes transitional and is removed after
dist reaches parity.

<a id="s-T8VN0H17MF"></a>
## Decision

Use cargo-release, git-cliff, and dist as three non-overlapping authorities:

- Cargo.toml workspace.package.version is the sole checked-in product version.
- cargo-release owns explicit version preparation, Cargo.lock regeneration, the
  portable-skill compatibility replacement, the changelog hook, and one reviewable
  release commit. Its dry run is the default rehearsal for preparation.
- git-cliff proposes CHANGELOG.md content from the previous stable tag. The release
  author edits and accepts it before merge.
- dist-workspace.toml is the canonical distribution configuration. Dist owns the
  generated release workflow, native target matrix, binary build, archive and checksum
  generation, release body, and GitHub publication.
- A small checked-in build-setup hook stages the platform-matched logic runtime,
  licenses, and portable skill for dist. The existing smoke test remains a post-build
  verifier until an equally direct dist check replaces it.

The generated workflow is checked in but never hand-edited. CI runs dist generation
and planning checks to detect drift. Dist, cargo-release, git-cliff, cargo-auditable,
the Rust toolchain, GitHub Actions, companion runtime downloads, and native runner
images must all be pinned in repository configuration. The generated workflow may use
custom runner configuration to retain the repository's supported runner versions.

Adopt dist's standard artifact names before 1.0 instead of preserving the former
docgraph-vVERSION-platform names with custom code. Consumers and the validation action
will use the release manifest or an exact asset selected by platform, not construct
legacy names.

<a id="s-HQB0F3R36H"></a>
## Changelog and supply-chain policy

CHANGELOG.md is canonical release prose and follows Keep a Changelog. It has an
Unreleased section and one dated section per stable tag. Git-cliff generates a
deterministic proposal from commits since the prior stable tag; the release author
curates it for user impact. Dist publishes that accepted text instead of generating a
second narrative.

The initial supply-chain target is SLSA Build Level 2 through GitHub artifact
attestations, not Level 3. Dist's built-in provenance covers native build outputs; the
issue 12 slice must prove that every archive and checksum manifest is included, attest
the final SBOM, and augment or combine cargo-cyclonedx output so the bundled native
runtime and its licenses are represented. Documentation must not claim a higher level
than the implemented builder isolation supports.

<a id="s-WATAQS2WZ2"></a>
## Failure recovery

Preparation happens on a branch and can be rerun, amended, or abandoned before a tag
exists. A tag is created only from the reviewed merged preparation commit. A failed
tag workflow may be rerun without moving the tag if no inputs changed. If inputs must
change, fix them and choose a new version. A published release or externally observed
tag is immutable; correct it with a patch release rather than replacing artifacts.

<a id="s-34EMGKG74A"></a>
## Consequences

The next slice replaces bespoke preparation and publication orchestration with pinned
configuration, adds the changelog, stages the companion payload for dist, checks the
generated workflow, and updates consumers for standard asset names. The later issue 12
slice hardens the generated pipeline and validates complete provenance and SBOM
coverage. Repository-owned code remains only where it expresses docgraph's product
boundary or verifies the installed product.
