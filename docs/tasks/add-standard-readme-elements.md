+++

id = "task:add-standard-readme-elements"
type = "task"
state = "done"

[properties]
title = "Add standard README elements and SECURITY.md"

[[relations]]
type = "part_of"
target = "plan:overhaul-onboarding-readme"

[docgraph_generated]
schema_version = 1

+++
<a id="s-EX1Y06ZHCY"></a>
# Add standard README elements and SECURITY.md

Add the conventional pieces the README is currently missing:

- **Badges** near the top — CI status, latest release, and license.
- **Table of contents** — a simple anchored list, once the restructured sections
  make the README long enough to warrant one.
- **License section** — a dedicated section stating MIT and linking the root
  `LICENSE`, plus the bundled logic-runtime's own notices
  (`THIRD_PARTY_LICENSES`).
- **Security & attestations** — a short section, plus a root `SECURITY.md`,
  covering how to report a vulnerability and the supply-chain guarantees
  (checksums and producer attestations on every release and install).
- **Changelog link** — one line pointing to `CHANGELOG.md`.
- **Getting help / support** — a line on where to file issues and ask questions.

<a id="s-WZKGVGAKGC"></a>
## Result

The README now carries CI/release/license badges, a table of contents, and
dedicated Security and attestations, License, Changelog, Contributing, and Getting
help sections. A root `SECURITY.md` documents private vulnerability reporting (now
enabled on the repository) and the checksum/attestation supply-chain guarantees.
