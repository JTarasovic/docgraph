+++

id = "task:preserve-packslip-verification-parity"
type = "task"
state = "backlog"

[properties]
title = "Preserve checksum and attestation parity for the packslip artifact"

[[relations]]
type = "part_of"
target = "plan:distribute-via-packslip"

[[relations]]
type = "depends_on"
target = "task:add-packslip-publish-step"

[[relations]]
type = "implements"
target = "plan:distribute-via-packslip#s-JPXATY681M"

[docgraph_generated]
schema_version = 1

+++
<a id="s-AQV6FB8CYJ"></a>
# Preserve checksum and attestation parity for the packslip artifact

The packslip artifact and its manifest carry the same SHA-256 checksums and GitHub
build attestations as the cargo-dist archives. Extend the host-phase attestation
filters so the packslip manifest and any packslip-specific artifact are attested
subjects, and confirm a consumer can verify them with the same `gh attestation verify`
procedure documented in the release-workflow runbook.

Depends on the packslip publish step existing. Done when a rehearsed release's packslip
artifact passes both checksum and attestation verification with parity to the archives.
