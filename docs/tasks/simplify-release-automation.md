+++

id = "task:simplify-release-automation"
type = "task"
state = "done"

[properties]
title = "Simplify release installation and automation"

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
# Simplify release installation and automation

<a id="s-559RWSY97B"></a>
## Outcome

[PR #26](https://github.com/JTarasovic/docgraph/pull/26), merged September 14, 2026, addresses
[#23](https://github.com/JTarasovic/docgraph/issues/23) with three public actions:
install the pinned runtime, install a released CLI, and validate with installed tools.
CI and release builds use those same actions.

The installers delegate download, checksum, attestation verification, and extraction
to the SHA-pinned `JTarasovic/download-verify-install` action. They supply the expected repository, producer workflow,
and release tag or pinned runtime producer commit. SBOM production and verification
belong to release evidence, not each installation.

Cargo-release, git-cliff, and cargo-dist retain ownership of release preparation
and publication. Small shell commands stage the installed companion and portable
files, render the changelog, and smoke-test native archives before publication.
The Rust release helper and tests asserting source-text snippets are removed.
The earlier detailed xtask implementation plan is superseded by this scope.

See the [release runbook](../reference/release-workflow.md) and
[release decision](../decisions/release-workflow-ownership.md).
Published-release evidence remains tracked in the
[attestation task](attest-release-artifacts.md).

The [v0.4.1 release workflow](https://github.com/JTarasovic/docgraph/actions/runs/34962720140)
succeeded on September 15, 2026. The same commit passed
[Linux CI](https://github.com/JTarasovic/docgraph/actions/runs/34962715184) and
[Windows CI](https://github.com/JTarasovic/docgraph/actions/runs/34962715164), including
public-action installation and validation with both bundled and standalone runtimes.
This completes the simplification scope; action compatibility acceptance remains in
the [dogfooding task](dogfood-validation-action.md).
