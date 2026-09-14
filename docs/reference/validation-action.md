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

Three public composite actions support combined or separate installation and validation:

- `JTarasovic/docgraph/install-runtime@<sha>` installs the pinned native sidecar
  and exports `DOCGRAPH_LOGIC_RUNTIME`.
- `JTarasovic/docgraph/install@<sha>` installs an exact released CLI and its
  bundled runtime. It exports `DOCGRAPH_EXECUTABLE` and `DOCGRAPH_LOGIC_RUNTIME`.
- `JTarasovic/docgraph@<sha>` installs the requested CLI and its bundled runtime,
  then validates. Set `install: false` to use tools already installed.

The installers delegate download, verification, and extraction to the SHA-pinned
`JTarasovic/download-verify-install` action. Docgraph supplies asset names,
producer identity constraints, and environment registration. They support
x86-64 GitHub-hosted Linux and Windows runners.
Consumers need no Rust, mise, or docgraph source checkout.

Each archive must pass SHA-256 checksum verification and gh attestation
verification before extraction or execution. The expected producer is
`JTarasovic/docgraph/.github/workflows/release.yml` at the requested tag for the
CLI, or `logic-runtime.yml` at `refs/heads/main` and the full producer commit
pinned in `install-runtime/action.yml` for the sidecar.
Missing evidence fails installation; there is no verification bypass.
The attested archive digest covers its entire payload. SBOMs remain published
release evidence; installing a binary does not download or inspect them.

`install` requires an exact stable `version` in `vMAJOR.MINOR.PATCH` form.
Both installers accept `token`, defaulting to the workflow token. Private release
access requires a token that can read the producer repository.
`install` outputs `version` (without `v`) and `executable`;
`install-runtime` outputs `executable` and `directory`, including licenses.
Installations use fresh runner temporary directories and are added to PATH.

Validation accepts `working-directory` (default `.`, relative to
`github.workspace`) and optional `changes`, passed as one argument to
`docgraph validate --changes`. Check out enough history for that ref.
Validation propagates the CLI exit status.

The root action's `install` input defaults to `true`. It composes a SHA-pinned
public CLI installer, requiring `version` and accepting `token` with the same
semantics as `install`. The release already contains the matching runtime, so no
second runtime download is needed. With `install: false`, it does not download
anything or change the selected tools; `version` and `token` are unused.

```yaml
permissions:
  contents: read
  attestations: read

steps:
  - uses: actions/checkout@<full-commit-sha>
  - uses: JTarasovic/docgraph@<full-commit-sha>
    with:
      version: <exact-release-tag>
```

Pin actions to reviewed full commit SHAs. The CLI includes its matching runtime.
To select the standalone sidecar, run `install-runtime` after `install`, then
validate with `install: false`.
Source-build CI uses `install-runtime` on its own. The last installer sets the
runtime for subsequent steps, including Windows PowerShell steps.

Existing split-install workflows must set `install: false` on their validation
step. Installation outputs remain available from the separate installer actions.
