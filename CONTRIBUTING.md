# Contributing to docgraph

This repository builds docgraph. The product contract lives in `docs/reference/`;
keep implementation and tests aligned with it.

## Commit and pull-request titles

Use [Conventional Commits](https://www.conventionalcommits.org/) for every commit
subject and pull-request title. Pull requests are squash-merged, so the title is
the permanent release-facing subject. The executable policy and allowed types live
in `committed.toml`; validate a proposed title with:

```text
"<title>" | committed --commit-file -
```

CI validates both changed commits and pull-request titles.

## Building from source

The repository pins its toolchain and task commands with [mise](https://mise.jdx.dev/).
With Rust and mise installed, run the aggregate local check:

```text
mise run check-local
```

This prepares the pinned native logic runtime for the current Linux or Windows
host, then runs the shared `mise run check` contract: formatting, lint, tests,
managed-change validation, dependency policy, and unused-dependency detection. All
Cargo build and test commands use `Cargo.lock`.

Run `mise run check-local` before handing off changes.

## Continuous integration

Linux CI installs a checksum-verified packaged runtime, then runs the complete
`mise run check` contract. Set `DOCGRAPH_CHANGE_BASE` to review managed changes
against a ref other than the local default, `HEAD`.

A separate path-filtered Windows workflow starts in parallel for changes to Rust,
fixtures, the validation action, or native-runtime infrastructure. It uses a
shallow checkout, installs only Rust and nextest, verifies the packaged Windows
runtime, and runs the shared test suite through the named `windows-e2e` overlay.
Documentation-only changes do not allocate a Windows runner.

A single developer host cannot reproduce the other operating system's runtime
installation, path behavior, or end-to-end execution; CI owns that cross-platform
coverage.

## Releasing

Contributors rehearse and publish releases with the
[release runbook](docs/reference/release-workflow.md).

## Working with the docs corpus

This repository uses docgraph to manage its own `docs` corpus. Edit prose
directly, but use `docgraph` commands for managed frontmatter, states, properties,
relations, and lifecycle changes. Inspect the model with `docgraph describe`,
preview substantial changes with `--dry-run`, and run `docgraph validate` before
committing. Portable guidance lives in `skills/docgraph/SKILL.md`; repository
guidance lives in `AGENTS.md`.
