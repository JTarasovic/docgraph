+++

id = "plan:distribute-via-packslip"
type = "plan"
state = "active"

[properties]
title = "Distribute docgraph via packslip"

[[relations]]
type = "implements"
target = "reference:release-workflow"

[docgraph_generated]
schema_version = 1

[[docgraph_generated.incoming]]
source = "issue:add-packslip-backend"
predicate = "affects"
target = "plan:distribute-via-packslip"

[[docgraph_generated.incoming]]
source = "task:add-packslip-publish-step"
predicate = "implements"
target = "plan:distribute-via-packslip#s-9J6D5YWEDV"

[[docgraph_generated.incoming]]
source = "task:add-packslip-publish-step"
predicate = "part_of"
target = "plan:distribute-via-packslip"

[[docgraph_generated.incoming]]
source = "task:confirm-mise-packslip-install"
predicate = "implements"
target = "plan:distribute-via-packslip#s-5G3W61DYCR"

[[docgraph_generated.incoming]]
source = "task:confirm-mise-packslip-install"
predicate = "part_of"
target = "plan:distribute-via-packslip"

[[docgraph_generated.incoming]]
source = "task:preserve-packslip-verification-parity"
predicate = "implements"
target = "plan:distribute-via-packslip#s-JPXATY681M"

[[docgraph_generated.incoming]]
source = "task:preserve-packslip-verification-parity"
predicate = "part_of"
target = "plan:distribute-via-packslip"

[[docgraph_generated.inverses]]
source = "plan:distribute-via-packslip"
type = "affected_by"
target = "issue:add-packslip-backend"

[[docgraph_generated.inverses]]
source = "plan:distribute-via-packslip"
type = "contains"
target = "task:add-packslip-publish-step"

[[docgraph_generated.inverses]]
source = "plan:distribute-via-packslip"
type = "contains"
target = "task:confirm-mise-packslip-install"

[[docgraph_generated.inverses]]
source = "plan:distribute-via-packslip"
type = "contains"
target = "task:preserve-packslip-verification-parity"

[[docgraph_generated.inverses]]
source = "plan:distribute-via-packslip#s-5G3W61DYCR"
type = "implemented_by"
target = "task:confirm-mise-packslip-install"

[[docgraph_generated.inverses]]
source = "plan:distribute-via-packslip#s-9J6D5YWEDV"
type = "implemented_by"
target = "task:add-packslip-publish-step"

[[docgraph_generated.inverses]]
source = "plan:distribute-via-packslip#s-JPXATY681M"
type = "implemented_by"
target = "task:preserve-packslip-verification-parity"

[[docgraph_generated.backlinks]]
source = "issue:add-packslip-backend#s-438V9ABXKA"
target = "docs/plans/distribute-via-packslip.md"

+++
<a id="s-R0K4JKQQ5G"></a>
# Distribute docgraph via packslip

<a id="s-K9EB342WH2"></a>
## Objective

Publish each docgraph release through packslip so the CLI installs through mise's
packslip backend, without weakening the integrity guarantees of the existing
cargo-dist GitHub release path. The packslip artifact is an additional distribution
surface over the same tagged inputs, not a second source of truth for versions or
build outputs.

<a id="s-F70ABCTEBT"></a>
## Work slices

<a id="s-9J6D5YWEDV"></a>
### Add the packslip package and publish step

Extend the tag-triggered release pipeline with a packslip packaging and publish step
that consumes the archives dist already builds. It runs after the archives and their
checksums exist and before or alongside the host publish, so a release either produces
both distribution surfaces or fails as a unit.

<a id="s-JPXATY681M"></a>
### Preserve checksum and attestation parity

The packslip-published artifact carries the same SHA-256 checksum and GitHub
attestation coverage as the cargo-dist archives, verifiable with the same
`gh attestation verify` procedure documented in the release-workflow runbook.

<a id="s-5G3W61DYCR"></a>
### Confirm the packslip install path

Confirm a released version installs through mise's packslip backend and that its
checksum and attestation verify at install time, before the README documents the
packslip install instructions.

<a id="s-DJKPY1D06Q"></a>
## Completion

A tagged release publishes a packslip artifact with checksum and attestation parity to
the GitHub release archives, a released version installs cleanly through mise's
packslip backend, and the release-workflow runbook and README describe the packslip
path.
