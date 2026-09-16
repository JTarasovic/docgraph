+++

id = "issue:add-packslip-backend"
type = "issue"
state = "resolved"

[properties]
title = "Add a packslip backend for release distribution"

[[relations]]
type = "affects"
target = "milestone:v1-0"

[[relations]]
type = "affects"
target = "plan:distribute-via-packslip"

[docgraph_generated]
schema_version = 1

+++
<a id="s-438V9ABXKA"></a>
# Add a packslip backend for release distribution

Docgraph is installable only through the cargo-dist GitHub release path today. There
is no packslip packaging or publish wiring in the repository, so the tool cannot be
installed through mise's packslip backend alongside the existing GitHub release. This
is build/release infrastructure work, not documentation.

The packslip artifact must reach the same integrity bar as the current path: SHA-256
checksums and GitHub attestations with parity to the archives dist already publishes,
and a released version that is actually installable through the packslip backend.

Tracked upstream as [#41](https://github.com/JTarasovic/docgraph/issues/41). Blocks the
packslip mise install instructions in the README overhaul
([#40](https://github.com/JTarasovic/docgraph/issues/40)); once this lands, document the
packslip install path there.

Disposition is carried by [plan:distribute-via-packslip](../plans/distribute-via-packslip.md).
