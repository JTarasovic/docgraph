+++

id = "reference:validation-action"
type = "reference"

[properties]
role = "design"

[docgraph_generated]
schema_version = 1

[[docgraph_generated.incoming]]
source = "task:publish-validation-action"
predicate = "implements"
target = "reference:validation-action"

[[docgraph_generated.inverses]]
source = "reference:validation-action"
type = "implemented_by"
target = "task:publish-validation-action"

+++
<a id="s-3CMEBZVCCG"></a>
# Validation action

The repository root publishes a composite GitHub Action through `action.yml`.
It installs one exact docgraph release, verifies the release archive against its
adjacent SHA-256 file, checks the packaged CLI version and adjacent logic runtime,
adds the installation directory to the job path, and validates the configured
corpus.

The action supports `windows-x86_64` and `linux-x86_64`, matching the published
release artifacts. Unsupported operating systems and architectures fail with an
actionable diagnostic.

Inputs:

- `version` is required and must be an exact semantic release tag such as
  `v0.1.0`. Floating versions are rejected.
- `working-directory` defaults to `.` relative to `github.workspace`.
- `changes` optionally supplies the Git ref for `docgraph validate --changes`.
- `token` defaults to the workflow token. Public releases need no additional
  access; a consumer downloading from a private docgraph repository must supply a
  token that can read its releases.

Outputs expose the installed semantic `version` without `v` and the absolute
`executable` path. The executable is also available to later job steps through
`PATH`.

Consumers should pin the action itself to a reviewed full commit SHA and request
only read access:

```yaml
permissions:
  contents: read

steps:
  - uses: actions/checkout@<full-commit-sha>
  - uses: JTarasovic/docgraph@<full-commit-sha>
    with:
      version: v0.1.0
      token: ${{ secrets.DOCGRAPH_RELEASE_TOKEN }} # only while releases are private
```

Pull-request validation that supplies `changes` must check out enough history for
that Git ref to be available. The action requires neither Rust, mise, a source
checkout of docgraph, nor a separately installed logic runtime.
