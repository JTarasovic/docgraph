# docgraph

[![CI](https://github.com/JTarasovic/docgraph/actions/workflows/ci.yml/badge.svg)](https://github.com/JTarasovic/docgraph/actions/workflows/ci.yml)
[![Latest release](https://img.shields.io/github/v/release/JTarasovic/docgraph?sort=semver)](https://github.com/JTarasovic/docgraph/releases/latest)
[![License: MIT](https://img.shields.io/badge/license-MIT-blue.svg)](LICENSE)

docgraph makes an arbitrary ontology enforceable in an ordinary Git repository.

- **What it is.** A repository-native engine for codifying an *ontology* — entity
  types, typed properties, relationships, and workflow states — over Markdown
  documentation, then querying, validating, and safely mutating that structure.
  The mechanism lives in the tool; the ontology and policy are defined by the
  repository.
- **The problem it solves.** Structured document semantics drift. Humans, and
  especially agents, don't reliably keep entity state, cross-references, and
  relationships consistent — even with explicit instructions — and end up
  reconstructing meaning with repo-wide grep and inconsistent hand-edits. Any
  state that lives only in an agent's context gets compacted, cleared, or ignored,
  and can't be edited, configured, or queried.
- **What docgraph does about it.** It makes the codified semantics canonical in
  Git alongside the prose — durable, user-editable, configurable, and queryable —
  and answers impact through the tool instead of grep. This works for any ontology
  the repository declares; issue/task/plan workflow tracking is one example of what
  you can model, not the definition of the tool.
- **How it works.** Markdown and frontmatter in Git are canonical; the graph,
  index, and derived state are disposable and rebuildable. Humans and agents edit
  prose freely but change managed state through docgraph commands that validate
  impact before writing. See [`docs/reference/design.md`](docs/reference/design.md)
  for the full model, and the [Quickstart](#quickstart) for hands-on.

## Table of contents

- [Status and support](#status-and-support)
- [Installation](#installation)
- [Quickstart](#quickstart)
- [GitHub Actions validation](#github-actions-validation)
- [Safe editing boundary](#safe-editing-boundary)
- [Security and attestations](#security-and-attestations)
- [License](#license)
- [Changelog](#changelog)
- [Contributing](#contributing)
- [Getting help](#getting-help)

## Status and support

Interfaces and configuration are still evolving before 1.0, so treat
compatibility as best effort and pin the release used by automation. Release
artifacts target **x86-64 Windows and x86-64 Linux only**; macOS and ARM artifacts
are not provided yet.

## Installation

Install a released build from the
[latest GitHub release](https://github.com/JTarasovic/docgraph/releases/latest):

- Download the archive for `x86_64-pc-windows-msvc` or
  `x86_64-unknown-linux-gnu`.
- Unpack it, keeping the `docgraph` executable beside the adjacent
  `docgraph-logic-runtime` and the license files included in the archive.
- Put the unpacked directory on `PATH`, or invoke the executable by its full path.
- Verify the install:

  ```text
  docgraph --version
  docgraph --help
  ```

The matching portable agent skill ships under `skills/docgraph` and is also
embedded in the CLI for verified repository installation.

### Verify the download

Every release archive is published with an adjacent `.sha256` checksum and a
producer attestation. Verify both before use:

- **Checksum** — recompute the archive's SHA-256 and compare it with the adjacent
  `.sha256` file.
- **Attestation** — confirm the archive was built by this repository:

  ```text
  gh attestation verify <archive> --repo JTarasovic/docgraph
  ```

### Install with mise

The GitHub-release backend installs a pinned release as an ordinary tool. Pin an
exact release tag in `mise.toml`:

```toml
[tools]
"github:JTarasovic/docgraph" = "<release-tag>"
```

Then run `mise install`. This works with the current cargo-dist release and
attestation setup.

> The packslip mise backend depends on build and release wiring that does not
> exist yet; it is tracked in
> [#41](https://github.com/JTarasovic/docgraph/issues/41) and will be documented
> once it lands.

## Quickstart

The minimal path to a working repository. From a Git repository root, create the
minimal configuration, compatible portable skill, agent guidance, and default
`docs` directory:

```text
docgraph init --dry-run
docgraph init
```

Inspect the model and validate the corpus:

```text
docgraph describe
docgraph validate
```

Create a managed document using a configured type and its required title:

```text
docgraph document create docs/tasks/next.md --id task:next --type task --title "Next task" --dry-run
docgraph document create docs/tasks/next.md --id task:next --type task --title "Next task"
docgraph validate
```

For adopting existing Markdown, external-entity sources, and the full command and
configuration reference, see
[`docs/reference/v0-config-reference-grammar.md`](docs/reference/v0-config-reference-grammar.md).
Use `--json` on any command when a script or agent needs structured output.

## GitHub Actions validation

The validation action installs a verified release and its matching runtime, then
runs `docgraph validate` against your corpus. Consumers need no Rust, mise, or
docgraph source checkout. Pin every action to a reviewed full commit SHA.

### Basic usage

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

### Pin to a SHA or select a version

- Pin the action to a reviewed full commit SHA (not a tag or branch).
- Omit `version` to install the latest stable docgraph release, or set it to an
  exact release tag (`vMAJOR.MINOR.PATCH`) to install that release.

### Install only or runtime only

- `install: false` — validate with tools already on the runner; `version` is then
  unused, but release lookup still runs using `token`.
- `JTarasovic/docgraph/install@<full-commit-sha>` — install the CLI and matching
  runtime with checksum and attestation verification, without running `validate`.
- `JTarasovic/docgraph/install-runtime@<full-commit-sha>` — install only the
  pinned runtime sidecar, for example when testing a source build.

Both installers require valid producer attestations and SHA-256 checksums. See
[the validation action contract](docs/reference/validation-action.md) for
working-directory, change-aware validation, supported runners, and outputs.

## Safe editing boundary

Markdown prose is directly editable. Managed identity, properties, workflow state,
and semantic relationships should be changed with docgraph commands; generated
frontmatter is a read model and should be refreshed with
`docgraph frontmatter sync`. Preview substantial mutations with `--dry-run`, then
run `docgraph validate`.

## Security and attestations

Every release archive ships with a SHA-256 checksum and a producer attestation, so
consumers can verify that an artifact was built from this repository before running
it (see [Verify the download](#verify-the-download)). To report a vulnerability,
follow [`SECURITY.md`](SECURITY.md).

## License

docgraph is distributed under the [MIT license](LICENSE). The bundled logic runtime
retains its own notices under `THIRD_PARTY_LICENSES`.

## Changelog

Notable changes are recorded in [`CHANGELOG.md`](CHANGELOG.md).

## Contributing

Building from source, the local check workflow, CI notes, and the release runbook
live in [`CONTRIBUTING.md`](CONTRIBUTING.md).

## Getting help

File bugs and feature requests, and ask questions, in the
[issue tracker](https://github.com/JTarasovic/docgraph/issues).
