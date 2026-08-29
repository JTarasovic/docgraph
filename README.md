# docgraph

docgraph is a repository-native document graph for Markdown documentation. It
keeps document identity, typed metadata, workflows, and explicit semantic
relationships in Git alongside the prose, then provides search, traversal,
validation, and safe mutation commands for people and software agents.

## Status and support

The first release is v0.1.0. Interfaces and configuration are still evolving
before 1.0, so treat compatibility as best effort and pin the release used by
automation. Initial release artifacts target x86-64 Windows and x86-64 Linux;
macOS and ARM artifacts are not provided yet.

## Installation

Release binaries are published through the repository's GitHub Releases. Choose
`docgraph-v0.1.0-windows-x86_64.zip` or
`docgraph-v0.1.0-linux-x86_64.tar.gz`, unpack it, and keep
the `docgraph` executable beside the adjacent `docgraph-logic-runtime` and
license files included in that archive. The matching portable agent skill is
included under `skills/docgraph` and is also embedded in the CLI for verified
repository installation. Put the unpacked directory on `PATH`,
or invoke the executable by its full path. Each archive has an adjacent
`.sha256` checksum file. docgraph is distributed under the MIT license; the
bundled logic runtime retains its own notices under `THIRD_PARTY_LICENSES`.

Check an installation with:

```text
docgraph --version
docgraph --help
```

## GitHub Actions validation

The root composite action installs a checksum-verified released archive and runs
`docgraph validate` without requiring Rust, mise, or a docgraph source checkout.
Pin both actions to reviewed full commit SHAs and select the exact docgraph binary
release explicitly:

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

See [the validation action contract](docs/reference/validation-action.md) for
working-directory, change-aware validation, supported runners, and outputs.

## Quickstart

docgraph operates on the configured `docs` corpus in the current repository.
Run it from the repository root. A repository must have a `.docgraph/`
configuration; this repository's `.docgraph/` directory is a working example.

Inspect the model and validate the complete corpus first:

```text
docgraph describe
docgraph validate
```

To bring an existing Markdown file under management, use `adopt`. The command
adds the managed identity while preserving authored prose and unrelated
frontmatter. Preview a change before applying it:

```text
docgraph adopt <path-to-existing-markdown> --id <entity-id> --type <configured-type> --dry-run
docgraph adopt <path-to-existing-markdown> --id <entity-id> --type <configured-type>
docgraph frontmatter sync
docgraph validate
```

The entity type and properties must be declared in the repository's
`.docgraph/entities.toml`. For a new managed document, use the configured type
and its required title property:

```text
docgraph document create docs/tasks/next.md --id task:next --type task --title "Next task" --dry-run
```

After authoring Markdown, normalize headings and validate again:

```text
docgraph normalize --dry-run
docgraph normalize
docgraph frontmatter sync
docgraph validate
```

Use `docgraph search "term"`, `docgraph get <entity>`, `docgraph neighbors
<entity>`, and `docgraph context <entity>` to inspect the indexed corpus. Use
`--json` on commands when a script or agent needs structured output. For the
full command and configuration reference, see
[`docs/reference/v0-config-reference-grammar.md`](docs/reference/v0-config-reference-grammar.md).

## Safe editing boundary

Markdown prose is directly editable. Managed identity, properties, workflow
state, and semantic relationships should be changed with docgraph commands;
generated frontmatter is a read model and should be refreshed with
`docgraph frontmatter sync`. Preview substantial mutations with `--dry-run`,
then run `docgraph validate`.

## Building from source

The repository pins its toolchain and task commands with mise. With Rust and
mise installed, run:

```text
mise run check
```

This runs the repository's formatting, lint, test, and validation checks.
