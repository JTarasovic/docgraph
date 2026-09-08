+++

id = "reference:validation-action"
type = "reference"

[properties]
role = "design"

[docgraph_generated]
schema_version = 1

[[docgraph_generated.incoming]]
source = "plan:harden-delivery-integrity"
predicate = "implements"
target = "reference:validation-action"

[[docgraph_generated.incoming]]
source = "task:publish-validation-action"
predicate = "implements"
target = "reference:validation-action"

[[docgraph_generated.inverses]]
source = "reference:validation-action"
type = "implemented_by"
target = "plan:harden-delivery-integrity"

[[docgraph_generated.inverses]]
source = "reference:validation-action"
type = "implemented_by"
target = "task:publish-validation-action"

+++
<a id="s-3CMEBZVCCG"></a>
# Validation action

Three public composite actions separate installation from validation:

- `JTarasovic/docgraph/install-runtime@<sha>` installs the pinned native logic
  runtime and exports `DOCGRAPH_LOGIC_RUNTIME`.
- `JTarasovic/docgraph/install@<sha>` installs an exact released CLI and its
  bundled runtime, adds the installation to `PATH`, and exports
  `DOCGRAPH_EXECUTABLE` and `DOCGRAPH_LOGIC_RUNTIME`.
- `JTarasovic/docgraph@<sha>` runs `docgraph validate` using the installed
  executable and runtime. It performs no installation.

The installers use `gh release download`, `gh attestation verify`,
`sha256sum`, and standard archive tools. The standalone runtime installer also
uses `jq` to read its pins and validate its SBOM. These commands and Bash are
available on GitHub-hosted Linux and Windows runners; self-hosted runners must
provide them. Consumers need no Rust, mise, package manager, or docgraph source
checkout. Supported release targets are x86-64 Linux and Windows.

Every downloaded archive and adjacent checksum must have a valid attestation
from `JTarasovic/docgraph` and the expected producer workflow. CLI evidence must
match `release.yml` and the requested release tag. Standalone runtime evidence
must match `logic-runtime.yml`, `refs/heads/main`, and the full producer commit
pinned in `tools/logic-runtime/artifacts.json`. Self-hosted producers are rejected.
Missing or invalid attestations fail installation; there is no checksum-only
fallback. Archives are verified before extraction or execution.

The standalone runtime installer also verifies its attested CycloneDX SBOM,
checks pinned archive, checksum-file, SBOM, and executable digests, and runs a small logic program.
The CLI installer checks the packaged version and presence of its bundled runtime.
Installations live in fresh runner temporary directories, so an existing cache
cannot bypass verification.

The `install` action requires `version` in exact `vMAJOR.MINOR.PATCH` form.
Floating versions and historical archive layouts are unsupported. Both installers
accept an optional `token`, defaulting to `github.token`; private release access
requires a token that can read the producer repository's releases and attestations.

The validation action accepts `working-directory` (default `.`, relative to
`github.workspace`) and optional `changes`, passed as one argument to
`docgraph validate --changes`. Change-aware validation needs sufficient checkout
history. It propagates the CLI exit status.

Pin the actions to reviewed full commit SHAs:

```yaml
permissions:
  contents: read
  attestations: read

steps:
  - uses: actions/checkout@<full-commit-sha>
  - uses: JTarasovic/docgraph/install@<full-commit-sha>
    with:
      version: <exact-release-tag>
  - uses: JTarasovic/docgraph@<full-commit-sha>
```

The CLI installation includes its matching runtime. To explicitly select the
standalone pinned runtime, run `install-runtime` after `install` and before
validation. Source-build CI uses `install-runtime` on its own; release packaging
uses that same action, then stages its installed payload.

`install` outputs `version` (without `v`) and the absolute `executable`.
`install-runtime` outputs the absolute `executable` and installation `directory`,
which includes licenses. The most recent installer sets the runtime for subsequent
steps, including PowerShell steps on Windows.

When migrating from the previous all-in-one root action, move `version` and
`token` to a preceding `install` step and read installation outputs from that
step. Keep validation inputs on the root action.
