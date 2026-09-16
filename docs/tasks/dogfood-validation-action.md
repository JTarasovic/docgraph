+++

id = "task:dogfood-validation-action"
type = "task"
state = "done"

[properties]
title = "Dogfood the validation action"

[[relations]]
type = "part_of"
target = "plan:harden-delivery-integrity"

[[relations]]
type = "implements"
target = "plan:harden-delivery-integrity#s-6R0PYTBY0Y"

[[relations]]
type = "depends_on"
target = "task:align-local-and-ci-checks"

[docgraph_generated]
schema_version = 1

[[docgraph_generated.backlinks]]
source = "task:simplify-release-automation#s-559RWSY97B"
target = "docs/tasks/dogfood-validation-action.md"

+++
<a id="s-F7SMPDW7XT"></a>
# Dogfood the validation action

Address [#14](https://github.com/JTarasovic/docgraph/issues/14) by making coverage of
the checked-out action and compatibility with published docgraph binaries explicit.

[PR #26](https://github.com/JTarasovic/docgraph/pull/26) added the main dogfooding paths.
On September 15, 2026, [Linux CI](https://github.com/JTarasovic/docgraph/actions/runs/34962715184)
and [Windows CI](https://github.com/JTarasovic/docgraph/actions/runs/34962715164) both
passed installation and repository validation through the checked-out root action,
then installed the pinned standalone runtime and validated again with `install: false`.
Linux logs confirm selection of the published v0.4.1 binary. Implementation checks run
separately from these published-binary checks.

The successful installation path exercises the checked-out root action together with
its SHA-pinned public CLI installer and a published binary. It covers both action
execution and published-binary compatibility in one path; it is not an independent
test of the checked-out CLI installer or a released root-action revision.

The action source is released through the repository revision selected by consumers.
Its pinned installer revision is part of that source; the requested docgraph binary
is a separately versioned release asset. The compatibility boundary is the
`docgraph validate` invocation, with optional `--changes <ref>`, its working
directory, and its exit status. No older-version support range is promised beyond
that CLI contract. The root action still looks up the latest stable release when
`install: false`, although its `version` input does not select the installed binary.

<a id="s-PXBJYAZV9H"></a>
## Acceptance

- [x] CI exercises the checked-out composite action against a representative fixture or
  the repository corpus.
- [x] A smoke test, separate from implementation checks, verifies installation and validation with the latest supported
  published binary without pretending to test unreleased binary behavior.
- [x] Action and binary compatibility is explicit at the `validate` CLI boundary,
  with a published-binary positive check and failure-path tests.
- [x] Documentation explains how the action and binary are published and versioned,
  including the relationship between the root action revision, pinned installer
  revision, and requested binary version.
- [x] Failure tests exercise invalid working-directory setup, an invalid installer
  version, and a validator failure; named CI and composite-action steps identify
  the failing layer.
