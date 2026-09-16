+++

id = "task:confirm-mise-packslip-install"
type = "task"
state = "backlog"

[properties]
title = "Confirm installability through mise's packslip backend"

[[relations]]
type = "part_of"
target = "plan:distribute-via-packslip"

[[relations]]
type = "depends_on"
target = "task:add-packslip-publish-step"

[[relations]]
type = "implements"
target = "plan:distribute-via-packslip#s-5G3W61DYCR"

[docgraph_generated]
schema_version = 1

+++
<a id="s-WCMBW2DBT0"></a>
# Confirm installability through mise's packslip backend

Install a released docgraph version through mise's packslip backend end to end: resolve
the packslip manifest, download the artifact, verify its checksum and attestation, and
run `docgraph --version` from the mise-managed install. Confirm the resolved version
matches the release tag.

Depends on the packslip publish step existing. Done when a published (or rehearsed)
release installs cleanly through the packslip backend and the README documents the
verified install commands.
