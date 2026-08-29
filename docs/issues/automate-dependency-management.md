+++

id = "issue:automate-dependency-management"
type = "issue"
state = "resolved"

[properties]
title = "Automate dependency management"

[[relations]]
type = "affects"
target = "milestone:v1-0"

[docgraph_generated]
schema_version = 1

+++
<a id="s-T1EN7KGEWP"></a>
# Automate dependency management

Choose and configure one dependency-update path for Rust crates, GitHub Actions, mise tools, and pinned external runtime artifacts. Preserve immutable action pins and lockfile review, avoid noisy one-PR-per-package churn, and ensure automated updates run the same checks as human changes.

<a id="s-X480QRF7ND"></a>
## Resolution

Adopted Renovate as the single update path and documented the policy in
[Dependency management](../reference/dependency-management.md). Compatible
non-major changes are grouped, majors require dashboard approval, action digests
remain immutable pins, and Cargo lockfile maintenance is scheduled.

The native logic-runtime source pins are a separate approval-gated group because
they require coordinated binary rebuilds and checksum updates. Renovate pull
requests use the repository's normal CI workflow and receive no validation
bypass. The configuration passes Renovate's current configuration validator and
its custom managers match every declared native-runtime source pin.
