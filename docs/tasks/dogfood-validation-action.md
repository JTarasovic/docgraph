+++

id = "task:dogfood-validation-action"
type = "task"
state = "backlog"

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

+++
<a id="s-F7SMPDW7XT"></a>
# Dogfood the validation action

Address [#14](https://github.com/JTarasovic/docgraph/issues/14) by separating two
currently conflated checks: testing the action implementation in the current checkout
and testing compatibility with an already published docgraph binary.

Document the publication model. The action source is released through the repository
tag used by consumers; its requested docgraph binary is a versioned release asset, so
it does not require a second repository or unrelated publication pipeline unless the
release-contract task finds a concrete distribution constraint.

<a id="s-PXBJYAZV9H"></a>
## Acceptance

- CI exercises the checked-out composite action against a representative fixture or
  the repository corpus.
- A separate smoke test verifies installation and validation with the latest supported
  published binary without pretending to test unreleased binary behavior.
- Action and binary version compatibility is explicit and covered by failure tests.
- Documentation explains how the action and binary are published and versioned.
- Dogfooding failures point to the action layer, installer layer, or validator layer.
