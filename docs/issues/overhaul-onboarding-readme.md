+++

id = "issue:overhaul-onboarding-readme"
type = "issue"
state = "resolved"

[properties]
title = "Overhaul the README and split contributor docs"

[[relations]]
type = "affects"
target = "milestone:v1-0"

[docgraph_generated]
schema_version = 1

[[docgraph_generated.incoming]]
source = "plan:overhaul-onboarding-readme"
predicate = "affected_by"
target = "issue:overhaul-onboarding-readme"

[[docgraph_generated.inverses]]
source = "issue:overhaul-onboarding-readme"
type = "affects"
target = "plan:overhaul-onboarding-readme"

+++
<a id="s-TNK4TNVVMG"></a>
# Overhaul the README and split contributor docs

The README front-loads a dense prose overview and buries actionable steps in long
paragraphs, so a first-time reader cannot quickly understand what docgraph is,
install it, and run it. Installation, validation-action, and quickstart guidance
are written as prose where headings, bullets, and examples would be faster to act
on. Source-build and release-runbook content lives in the README where it belongs
in a contributor document, and the README is missing conventional elements
(badges, a table of contents, a dedicated license section, a changelog link, a
getting-help section, and a security/attestation story with a `SECURITY.md`).

Tracked upstream as [#40](https://github.com/JTarasovic/docgraph/issues/40),
consolidating [#35](https://github.com/JTarasovic/docgraph/issues/35),
[#36](https://github.com/JTarasovic/docgraph/issues/36),
[#37](https://github.com/JTarasovic/docgraph/issues/37),
[#38](https://github.com/JTarasovic/docgraph/issues/38), and
[#39](https://github.com/JTarasovic/docgraph/issues/39).
