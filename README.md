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

Prebuilt archives for `x86_64-pc-windows-msvc` and `x86_64-unknown-linux-gnu` are
published on the
[latest GitHub release](https://github.com/JTarasovic/docgraph/releases/latest).
Each archive bundles the `docgraph` executable, its `docgraph-logic-runtime`
sidecar, the portable agent skill under `skills/docgraph`, and the license files.

### With mise (recommended)

```toml
# mise.toml — installs docgraph + its logic-runtime sidecar and verifies the
# release before use; lockfile = true records the verified provenance so it is
# pinned and auditable.
[settings]
lockfile = true

[tools]
# Preferred: the packslip backend resolves docgraph's signed release manifest and
# verifies its Sigstore signature and each artifact's digest before unpacking.
"packslip:github.com/JTarasovic/docgraph" = "<release-tag>"
# Fallback: the GitHub backend installs the same archives and verifies their
# GitHub attestation and SLSA provenance.
# "github:JTarasovic/docgraph" = "<release-tag>"
```

```sh
mise install       # verifies signature/attestation, writes provenance to mise.lock
docgraph --version
```

### Manual install

```sh
# gh is required for attestation verification anyway, so use it to fetch the release
gh release download <release-tag> --repo JTarasovic/docgraph --pattern '*'
gh attestation verify <archive> --repo JTarasovic/docgraph   # before unpacking
# then unpack, keeping docgraph beside docgraph-logic-runtime, and put it on PATH
```

## Quickstart

`docgraph init` scaffolds the configuration but an **empty ontology** — you declare
the entity types docgraph should enforce. From a Git repository root:

```sh
docgraph init

# Declare a minimal ontology: one entity type with a required title.
cat > .docgraph/entities.toml <<'EOF'
[entity.note]
description = "A short note."

[entity.note.property.title]
type = "string"
required = true
EOF

docgraph describe                 # confirm the "note" type is registered
docgraph document create docs/notes/first.md --id note:first --type note --title "First note"
docgraph validate
```

Add properties, relations, and workflows the same way; see
[config authorship](skills/docgraph/config-authorship.md) and the
[configuration reference](docs/reference/v0-config-reference-grammar.md). Add
`--json` to any command when a script or agent needs structured output.

## GitHub Actions validation

The action installs a checksum- and attestation-verified release and its matching
runtime, then runs `docgraph validate` against your corpus. Consumers need no Rust,
mise, or docgraph source checkout. Pin every `uses:` to a reviewed full commit SHA
(shown below as `<sha>`).

The jobs below each show one supported mode:

```yaml
# .github/workflows/docgraph.yml
name: docgraph
on: [push, pull_request]

permissions:
  contents: read
  attestations: read   # required for producer-attestation verification on install

jobs:
  # Default: install a verified release + runtime, then run `docgraph validate`.
  validate:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@<sha>
      - uses: JTarasovic/docgraph@<sha>
        with:
          version: <exact-release-tag>   # optional; omit to use the latest stable release

  # Validate with docgraph already on the runner. `version` is unused, but release
  # lookup still runs using `token` (the workflow token by default).
  validate-preinstalled:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@<sha>
      - uses: JTarasovic/docgraph@<sha>
        with:
          install: "false"

  # Install only, without validating: the CLI + runtime (checksum + attestation
  # verified), and separately just the runtime sidecar (e.g. testing a source build).
  install-only:
    runs-on: ubuntu-latest
    steps:
      - uses: JTarasovic/docgraph/install@<sha>
        with:
          version: <exact-release-tag>
      - uses: JTarasovic/docgraph/install-runtime@<sha>
        with:
          version: <exact-release-tag>
```

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
it. Installing [with mise](#with-mise-recommended) verifies the attestation by
default; a [manual install](#manual-install) verifies it with `gh attestation
verify`; and the [validation action](#github-actions-validation) installers verify
it in CI. To report a vulnerability, follow [`SECURITY.md`](SECURITY.md).

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
