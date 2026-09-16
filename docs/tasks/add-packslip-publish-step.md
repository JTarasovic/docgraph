+++

id = "task:add-packslip-publish-step"
type = "task"
state = "done"

[properties]
title = "Add a packslip package and publish step to the release pipeline"

[[relations]]
type = "part_of"
target = "plan:distribute-via-packslip"

[[relations]]
type = "implements"
target = "plan:distribute-via-packslip#s-9J6D5YWEDV"

[docgraph_generated]
schema_version = 1

[[docgraph_generated.incoming]]
source = "task:confirm-mise-packslip-install"
predicate = "depends_on"
target = "task:add-packslip-publish-step"

[[docgraph_generated.incoming]]
source = "task:preserve-packslip-verification-parity"
predicate = "depends_on"
target = "task:add-packslip-publish-step"

[[docgraph_generated.inverses]]
source = "task:add-packslip-publish-step"
type = "required_by"
target = "task:confirm-mise-packslip-install"

[[docgraph_generated.inverses]]
source = "task:add-packslip-publish-step"
type = "required_by"
target = "task:preserve-packslip-verification-parity"

+++
<a id="s-2TDK9WNJD7"></a>
# Add a packslip package and publish step to the release pipeline

Add a packslip packaging and publish step to the tag-triggered release. It consumes the
Windows and Linux archives dist already builds, generates the packslip manifest that
mise's packslip backend resolves, and publishes it against the release tag. The step
must not re-derive the version or rebuild binaries; the tag and the dist archives are
the only inputs.

Because dist owns the generated `release.yml`, wire the step through the supported
extension points (`dist-workspace.toml` custom jobs / `github-build-setup`) rather than
hand-editing generated workflow YAML, so it survives the next `dist plan` regeneration.

Done when a rehearsed release produces a packslip artifact from the tagged archives and
the release-workflow runbook documents the step.

<a id="s-FFE3E4FEXK"></a>
## Resolution

Implemented as the `release-packslip` global-artifacts job wired through
`global-artifacts-jobs`. It builds a keyless-signed `packslip.sigstore.json` over the
build-local archives (`jdx/packslip`, `upload: false`, `attest: link`, `bin: docgraph`)
and uploads it so the host job publishes it into the immutable release. The signing job
gets `id-token: write` through dist's `github-custom-job-permissions`, keeping the
generated `release.yml` free of drift. First published in v0.5.1.
