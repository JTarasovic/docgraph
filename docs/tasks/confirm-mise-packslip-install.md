+++

id = "task:confirm-mise-packslip-install"
type = "task"
state = "done"

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

<a id="s-VQR43JDFFQ"></a>
## Resolution

Verified against the v0.5.1 release. `mise use packslip:github.com/JTarasovic/docgraph`
resolved the signed manifest, verified its Sigstore signature and the selected artifact's
digest, and installed a working `docgraph 0.5.1`. The README documents packslip as the
preferred install path with the GitHub backend as a fallback.
