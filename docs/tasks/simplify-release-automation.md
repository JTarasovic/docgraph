+++

id = "task:simplify-release-automation"
type = "task"
state = "in_progress"

[properties]
title = "Simplify release automation in PR 24"

[[relations]]
type = "part_of"
target = "plan:harden-delivery-integrity"

[[relations]]
type = "implements"
target = "decision:release-workflow-ownership"

[docgraph_generated]
schema_version = 1

[[docgraph_generated.incoming]]
source = "task:attest-release-artifacts"
predicate = "depends_on"
target = "task:simplify-release-automation"

[[docgraph_generated.inverses]]
source = "task:simplify-release-automation"
type = "required_by"
target = "task:attest-release-artifacts"

+++
<a id="s-714DK2XTTT"></a>
# Simplify release automation in PR 24

<a id="s-559RWSY97B"></a>
## Outcome and implementation instructions

Rework [PR #24](https://github.com/JTarasovic/docgraph/pull/24) so existing
release tools own their supported operations and a small Rust xtask owns only
docgraph-specific glue. Delete the three authored release shell scripts. Preserve
the useful pinned evidence and complete the pre-publication portion of
[#12](https://github.com/JTarasovic/docgraph/issues/12).

This is an implementation plan, with ownership decisions already settled. Execute
the steps below in order; do not restart a release-tool evaluation. Use the pinned
mise environment. The reviewed baseline is PR head cf5903398c346ed814066e36a67a9f655aefbd8c.
Inspect differences if the branch has advanced, preserving unrelated work.

Read the [release decision](../decisions/release-workflow-ownership.md),
[release contract](../reference/release-workflow.md), and
[attestation task](attest-release-artifacts.md) first. Inspect their semantic context
with docgraph. This corrects the implementation of
[#15](https://github.com/JTarasovic/docgraph/issues/15); it does not replace the
selected cargo-release, git-cliff, and cargo-dist toolchain.

<a id="s-3YPZH5BNTE"></a>
## Scope boundaries

- Keep Cargo.toml workspace.package.version canonical. Cargo-release owns derived
  versions and preparation commits; git-cliff owns changelog rendering; dist owns
  product archives, checksums, manifest, workflow generation and publication.
- Rust xtask is contributor tooling, excluded from product distribution and release
  version propagation. It must not depend on the docgraph crates or native runtime
  merely to stage inputs. Invoke tools with argument arrays and propagate failures.
- No public installer redesign, mise installation project, action input changes,
  or changes to tools/action/validate.ps1. [#23](https://github.com/JTarasovic/docgraph/issues/23)
  remains separate. Preserve existing consumer compatibility for this task.
- Do not port tools/logic-runtime/build-linux.sh or build-windows.ps1, redesign the
  companion producer workflow, upgrade the toolchain, or rebuild published companions.
- No generic release framework, custom provenance cryptography, custom SBOM generator,
  or parallel product packager. Generated shell inside dist's workflow is tool-owned;
  the goal is to remove authored release business logic, not ban every shell invocation.
- Do not publish, tag, merge, or close GitHub issues as part of implementation.

<a id="s-36VSEW225X"></a>
## Delete, configure, or implement

| Current responsibility | Disposition |
| --- | --- |
| tools/release/update-changelog.sh: parse Cargo.toml and infer dry-run from version equality | Delete; consume cargo-release hook environment |
| Same script: render and prepend release prose | Keep git-cliff as owner; configure directly if it can satisfy preview semantics, otherwise a minimal xtask adapter |
| tools/release/stage-dist-inputs.sh: parse pins, verify companion inputs, lay out runtime/skill/licenses | Small xtask operation using typed data and existing gh verifier |
| tools/release/smoke-test.sh: execute installed product scenarios | Rust xtask integration checks |
| release-smoke.yml: grep SBOM XML, count archives and parse checksum lines in shell | Typed evidence checks in xtask, using dist outputs for product artifact identity |
| release.yml: archive creation, checksum/SBOM generation, attestation/publication | Keep dist ownership; regenerate from configuration |
| repository_check_contract.rs: require Bash scripts and exact snippets | Delete those assertions; test behavior and essential wiring |

<a id="s-R7JWNSS69Z"></a>
## Step 1: Add the smallest xtask foundation

Add xtask/Cargo.toml and xtask/src/main.rs, register the workspace member, and add
a cargo xtask alias in .cargo/config.toml. Use small modules as needed, not a plugin
architecture. Reuse workspace clap, serde, serde_json and TOML support. Add narrowly
needed hashing/archive/XML/test dependencies with compatible licenses and update
Cargo.lock. Use maintained parsers rather than string matching structured formats.

Set publish=false and dist=false, and explicitly exclude xtask from cargo-release
preparation. Verify release planning still selects only docgraph-cli and no additional
versioned surface appears. Ensure xtask tests are included in workspace checks.

Use these command boundaries (option spelling can follow existing CLI conventions):

    cargo xtask release stage [--verify-attestations]
    cargo xtask release smoke --target <triple> --version <version> --archive <path>
    cargo xtask release changelog

The last command is needed only if direct git-cliff configuration cannot implement
the following hook behavior. Do not add speculative subcommands.

<a id="s-X2CZTDMBRE"></a>
## Step 2: Remove the changelog wrapper's duplicated decisions

Update crates/docgraph-cli/Cargo.toml pre-release-hook and, only as necessary,
cliff.toml/release.toml. The pinned cargo-release 1.1.5 hook exposes DRY_RUN,
PREV_VERSION, NEW_VERSION and WORKSPACE_ROOT; use these instead of parsing Cargo.toml
or comparing the checked-in version with the requested version. Missing or invalid
hook context must fail clearly. Keep the hook package-scoped to docgraph-cli so it
runs once, with a working directory/manifest path that works from that crate.

Use git-cliff to render the accepted changelog format for the previous product release
through HEAD and the intended tag. Preserve prior curated entries and comparison
links. Companion tags must never become the changelog base. Verify how PREV_VERSION
maps to the prior product tag in this repository; if a tag is absent, diagnose it
rather than silently generating an incorrect range.

DRY_RUN=true renders a proposal without changing files. Execute mode updates the
changelog through git-cliff before cargo-release creates its consolidated commit.
Do not implement a second version updater or commit/tag/push operation. Verify both
modes in a disposable Git repository, including paths containing spaces. Delete
tools/release/update-changelog.sh after rewiring its caller.

<a id="s-MK74DM0SQ7"></a>
## Step 3: Move companion staging and archive smoke behavior

Read tools/logic-runtime/sources.toml as typed data. Preserve PR #24's release names,
archive/checksum/SBOM/binary digests and exact producer revision. Keep the existing
gh release download and gh attestation verify tools as owners of GitHub transport
and provenance verification; pass the current repository, signer workflow, source
digest/ref and hosted-runner constraints. Never print authentication secrets.

Stage verifies pinned file digests, checksum-to-archive agreement, required CycloneDX
components and extracted binary digest before replacing target/release-inputs.
Require one expected runtime and its licenses, reject extraction outside scratch,
and clean up only owned temporary paths. Failed verification must preserve any
previous valid staging directory. Preserve the current dist include layout,
including its extensionless runtime filename, to avoid an unrelated packaging change.
Keep provenance verification mandatory at the CI/release callers that currently
request it; a local staging invocation without the flag still verifies pinned hashes.

Smoke extracts a supplied archive in isolation and exercises the existing version,
help, instructions preview/sync/check, validate, scalar_values query and search cases.
Preserve the payload checks for runtime, README, license and portable skill. Execute
with DOCGRAPH_LOGIC_RUNTIME removed to prove normal adjacent runtime discovery;
do not repair executable permissions to hide packaging defects. Use Rust process
and filesystem APIs instead of translating shell fragments into shell invocations.

Update .github/release-build-setup.yml, the packaged-runtime installation steps in
.github/workflows/ci.yml and windows-e2e.yml, and the smoke callers. Provide pinned
Rust/mise setup before xtask invocation on each runner. Delete stage-dist-inputs.sh
and smoke-test.sh once no authored caller needs them. Keep native source-build tasks
in mise.toml unchanged; add only the release task entry points needed here.

<a id="s-9S8W04P7R8"></a>
## Step 4: Correct the evidence gate and its scheduling

The baseline custom-release-smoke job depends on local builds only, yet tries to
download/check global checksum and SBOM outputs. Fix this scheduling through dist
configuration and supported custom-job phases. Inspect the generated dependency
graph: the complete evidence check must run after both local and global artifacts,
and before host publication. Changing only the download wildcard does not fix it.
If dist 0.32.0 cannot express this, document the exact limitation and stop that portion
instead of hand-editing generated release.yml or silently weakening the gate.

Use the dist plan/build manifest to identify product archives and checksum outputs.
Consume the workflow's existing plan input where appropriate; do not leave an unused
input and maintain a second asset naming scheme. Inspect actual pinned-version
manifest output before defining the minimal deserialized fields. SBOMs not listed
in that manifest require an explicit check of cargo-cyclonedx's generated outputs;
do not assume every output is represented there.

Validate required files, checksum contents and coverage, parsed CycloneDX format,
and docgraph-cli's expected version. Preserve required companion verification and
product attestation subject filters, including the workspace SBOM and sha256.sum.
Do not generate replacement checksums or SBOMs. Missing evidence must fail.

Product attestations are generated in dist's host phase. Pre-host checks can verify
inputs and already published companion provenance; published product provenance
must be verified after a real release. State this distinction in the runbook and
attestation task. This implementation cannot close #12's published-release proof.

Regenerate release.yml with pinned dist after configuration changes. Arrange a
non-publishing native release rehearsal that actually runs the changed archive and
evidence checks before merge. Prefer supported dist PR execution; if its available
mode omits global/custom jobs, explicitly exercise the missing checks in the existing
CI workflow. Do not assume changing pr-run-mode alone proves the complete pipeline.

<a id="s-ZE5K6J7KX3"></a>
## Step 5: Tests, documentation and handoff

Remove portable_release_automation_uses_bash and related assertions enforcing script
contents. Retain meaningful policy checks for pins, permissions, mandatory checks,
required artifact coverage and publication dependencies. Put release helper tests
with xtask rather than increasing docgraph-cli's test ownership of tooling internals.

Add focused behavior tests for dry-run non-mutation and execute changelog output;
valid and corrupt companion inputs; absent/malformed SBOM and wrong product version;
missing checksum coverage; failed verifier exit; and failed staging preserving prior
inputs. Use local fixtures or injected tool results for deterministic failure tests,
with native archive rehearsals providing real tool integration evidence. Avoid tests
that only assert the helper constructs a particular command string.

Update docs/reference/release-workflow.md, docs/tasks/attest-release-artifacts.md and
the release decision's affected prose to show the actual xtask commands and ownership.
Retain accepted versioning, immutability and changelog policy. Use docgraph commands
for managed metadata. Delete obsolete authored-script references; leave historical
discussion and unrelated scripts alone. Record concrete rehearsal results and any
unverified Linux/Windows or post-publication work without claiming completion.

Before handoff, run in the pinned mise environment:

    mise run check-local
    docgraph normalize --dry-run
    docgraph frontmatter sync --dry-run
    docgraph validate
    docgraph validate --changes HEAD

Apply required normalization/frontmatter changes using the CLI and validate again.
check-local includes dist generation/planning checks. Also run the native release
rehearsal described above; ordinary unit tests do not replace it. Validate any commit
or PR title with committed; suggested subject: refactor(release): replace custom shell automation.

Done means the three authored release scripts are gone, existing tools own their
standard operations, the residual Rust glue is tested on both native targets, the
complete pre-publication gate is correctly ordered, and the runbook matches reality.
The parent attestation task remains open for verification of a subsequently published
release. Do not treat this task as permission to expand into #23 or native build rewrites.

<a id="s-9JJTJN6Q4W"></a>
### Pinned dist scheduling result

Dist 0.32.0's `global-artifacts-jobs` starts custom jobs alongside the global-artifact
phase, before the workspace SBOM and unified checksum are available. Its `host-jobs`
run only after dist's host job has already uploaded and created the release. Neither
supported phase supplies a complete post-global, pre-host evidence gate. The generated
workflow is therefore left under dist ownership; no hand-edited dependency is applied.
This limitation leaves the complete pre-publication gate pending rather than adding a
second product-evidence implementation outside dist.
