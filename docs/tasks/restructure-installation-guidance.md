+++

id = "task:restructure-installation-guidance"
type = "task"
state = "done"

[properties]
title = "Restructure installation into verified bulleted steps"

[[relations]]
type = "part_of"
target = "plan:overhaul-onboarding-readme"

[docgraph_generated]
schema_version = 1

[[docgraph_generated.incoming]]
source = "task:add-quickstart-section"
predicate = "depends_on"
target = "task:restructure-installation-guidance"

[[docgraph_generated.inverses]]
source = "task:restructure-installation-guidance"
type = "required_by"
target = "task:add-quickstart-section"

+++
<a id="s-02XM9935GD"></a>
# Restructure installation into verified bulleted steps

Convert the prose installation block into scannable bulleted steps:

- Download the archive for `x86_64-pc-windows-msvc` or
  `x86_64-unknown-linux-gnu` from the latest GitHub release.
- Unpack it, keeping `docgraph` beside `docgraph-logic-runtime` and the license
  files.
- Put the directory on `PATH`, or invoke by full path.
- Verify with `docgraph --version` and `docgraph --help`.

Add explicit supply-chain verification steps that are currently only implied:
verify the `.sha256` checksum and the producer attestation
(`gh attestation verify ...`) before use.

Add mise install instructions for the GitHub-release backend (a `github:`/ubi-style
tool entry pinned to a release tag), which works with the current cargo-dist
release and attestation setup. The Packslip backend depends on build/release
wiring that does not exist yet and is deferred to
[#41](https://github.com/JTarasovic/docgraph/issues/41); do not document it here.

Note platform support — x86-64 Windows and Linux only, no macOS or ARM yet — and
the MIT license and bundled-runtime notices, here or in a status section.

<a id="s-7JWDTXF093"></a>
## Result

Installation is now bulleted (download, unpack, PATH, verify) with a dedicated
"Verify the download" subsection covering the `.sha256` checksum and
`gh attestation verify`, and an "Install with mise" subsection documenting the
GitHub-release backend. The packslip backend is called out as deferred to #41.
Platform support and license/runtime notices are stated in Status and License.
