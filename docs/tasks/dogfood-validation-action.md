+++

id = "task:dogfood-validation-action"
type = "task"
state = "in_progress"

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

Finish documenting the publication model. The action source is released through the repository
tag used by consumers; its requested docgraph binary is a versioned release asset, so
it does not require a second repository or unrelated publication pipeline unless the
release-contract task finds a concrete distribution constraint.

<a id="s-PXBJYAZV9H"></a>
## Acceptance

- [x] CI exercises the checked-out composite action against a representative fixture or
  the repository corpus.
- [x] A smoke test, separate from implementation checks, verifies installation and validation with the latest supported
  published binary without pretending to test unreleased binary behavior.
- [ ] Action and binary version compatibility is explicit and covered by failure tests.
- [ ] Documentation explains how the action and binary are published and versioned,
  including the relationship between the root action revision, pinned installer
  revision, and requested binary version.
- [ ] Failure tests demonstrate that dogfooding failures point to the action layer,
  installer layer, or validator layer. Named CI and composite-action steps already
  separate these operations; negative-path coverage remains to be added.
