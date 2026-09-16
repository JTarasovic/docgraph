+++

id = "plan:overhaul-onboarding-readme"
type = "plan"
state = "completed"

[properties]
title = "Overhaul the onboarding README and split contributor docs"

[[relations]]
type = "required_for"
target = "milestone:v1-0"

[[relations]]
type = "affected_by"
target = "issue:overhaul-onboarding-readme"

[[relations]]
type = "implements"
target = "reference:design#s-YNPWHT3H8T"

[docgraph_generated]
schema_version = 1

[[docgraph_generated.incoming]]
source = "task:add-quickstart-section"
predicate = "part_of"
target = "plan:overhaul-onboarding-readme"

[[docgraph_generated.incoming]]
source = "task:add-standard-readme-elements"
predicate = "part_of"
target = "plan:overhaul-onboarding-readme"

[[docgraph_generated.incoming]]
source = "task:bulletize-validation-action-usage"
predicate = "part_of"
target = "plan:overhaul-onboarding-readme"

[[docgraph_generated.incoming]]
source = "task:extract-contributor-docs"
predicate = "part_of"
target = "plan:overhaul-onboarding-readme"

[[docgraph_generated.incoming]]
source = "task:restructure-installation-guidance"
predicate = "part_of"
target = "plan:overhaul-onboarding-readme"

[[docgraph_generated.incoming]]
source = "task:rewrite-project-overview"
predicate = "part_of"
target = "plan:overhaul-onboarding-readme"

[[docgraph_generated.inverses]]
source = "plan:overhaul-onboarding-readme"
type = "contains"
target = "task:add-quickstart-section"

[[docgraph_generated.inverses]]
source = "plan:overhaul-onboarding-readme"
type = "contains"
target = "task:add-standard-readme-elements"

[[docgraph_generated.inverses]]
source = "plan:overhaul-onboarding-readme"
type = "contains"
target = "task:bulletize-validation-action-usage"

[[docgraph_generated.inverses]]
source = "plan:overhaul-onboarding-readme"
type = "contains"
target = "task:extract-contributor-docs"

[[docgraph_generated.inverses]]
source = "plan:overhaul-onboarding-readme"
type = "contains"
target = "task:restructure-installation-guidance"

[[docgraph_generated.inverses]]
source = "plan:overhaul-onboarding-readme"
type = "contains"
target = "task:rewrite-project-overview"

+++
<a id="s-W65BX60Z0N"></a>
# Overhaul the onboarding README and split contributor docs

<a id="s-AR7R79T2B0"></a>
## Objective

Restructure the README so a new user can understand what docgraph is, install it,
and run it within minutes. Lead with a digestible what/why/how overview, make
installation and quickstart example-first and scannable, move contributor and
release-runbook content into a dedicated `CONTRIBUTING.md`, and add the
conventional README elements the project is missing. The overview must present the
general capability first — an enforceable repository-native ontology over Markdown
— and must not frame the tool around any single use case, grounding the framing in
the design's mechanism-versus-policy distinction and its primary acceptance
principle.

<a id="s-ENRTKK8ZB3"></a>
## Report coverage

This plan consolidates one upstream umbrella report and its five superseded
reports:

- [#40](https://github.com/JTarasovic/docgraph/issues/40): umbrella overhaul and
  the new standard-elements scope.
- [#39](https://github.com/JTarasovic/docgraph/issues/39): project-overview
  rewrite.
- [#35](https://github.com/JTarasovic/docgraph/issues/35): installation with
  checksum and attestation verification and mise instructions.
- [#37](https://github.com/JTarasovic/docgraph/issues/37): short quickstart under
  installation.
- [#36](https://github.com/JTarasovic/docgraph/issues/36): validation-action usage
  as bullets and examples.
- [#38](https://github.com/JTarasovic/docgraph/issues/38): source-build and
  release content extracted into `CONTRIBUTING.md`.

<a id="s-ANQZDX9920"></a>
## Priority and sequence

1. Rewrite the overview so the top of the README reads correctly first.
2. Restructure installation into verified bulleted steps, then place the quickstart
   directly beneath it.
3. Convert the validation-action section to bullets and examples.
4. Extract source-build and release-runbook content into `CONTRIBUTING.md` and
   leave a single README link.
5. Add the standard README elements and a root `SECURITY.md`.

The mise Packslip backend is out of scope and deferred to
[#41](https://github.com/JTarasovic/docgraph/issues/41); document only the
GitHub-release backend here.

<a id="s-SJZEZFGQX5"></a>
## Completion

The README leads with a digestible what/why/how overview; installation is bulleted
with checksum and attestation verification plus GitHub-release-backend mise
instructions; a short quickstart sits under installation; the validation-action
section is bullets and examples; `CONTRIBUTING.md` holds the source-build and
release-runbook content with the README linking to it; and the README carries
badges, a table of contents, a license section, a changelog link, a getting-help
section, and a security/attestations section backed by a root `SECURITY.md`.
