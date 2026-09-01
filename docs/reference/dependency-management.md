+++

id = "reference:dependency-management"
type = "reference"

[properties]
role = "design"

[docgraph_generated]
schema_version = 1

[[docgraph_generated.incoming]]
source = "task:automate-dependency-management"
predicate = "implements"
target = "reference:dependency-management"

[[docgraph_generated.inverses]]
source = "reference:dependency-management"
type = "implemented_by"
target = "task:automate-dependency-management"

[[docgraph_generated.backlinks]]
source = "issue:automate-dependency-management#s-X480QRF7ND"
target = "docs/reference/dependency-management.md"

+++
<a id="s-PRNVYT7ZFG"></a>
# Dependency management

Renovate is the repository's single dependency-update path. Its root
`renovate.json` configuration covers Cargo manifests and `Cargo.lock`,
digest-pinned GitHub Actions, pinned mise tools, and the source revisions used to
build the native logic-runtime companion.

Non-major updates are grouped to avoid one pull request per package. Major
updates require Dependency Dashboard approval. Updates wait three days after a
release where the datasource exposes a release timestamp; periodic lockfile
maintenance catches transitive Cargo changes.

GitHub Actions remain pinned to full commit digests with readable version
comments. Renovate updates the digest and comment together. Do not replace those
pins with floating tags.

Native runtime changes are approval-gated and grouped because a source update is
not complete by itself. Before merging such a pull request:

1. Rebuild and smoke-test the Windows and Linux runtime companions.
2. Publish both immutable companion releases for the new source revision.
3. Update the release names, archive URLs, archive checksums, and binary checksums
   in `tools/logic-runtime/sources.toml` and the CI/release workflows.
4. Run the same `mise run check-local` contract required for authored changes.

Renovate pull requests use the normal pull-request workflow and receive no CI or
merge bypass. The Linux `rust` job runs the complete shared contract, including Cargo
policy and unused-dependency checks. The path-filtered `windows-e2e` workflow runs the
locked test suite for changes that can affect Windows behavior. Because GitHub leaves
path-filtered required checks pending when their workflow does not run, require `rust`
globally but do not make `windows-e2e` an unconditional branch-protection check.
