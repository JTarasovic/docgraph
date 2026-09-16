# Changelog

All notable changes to docgraph are documented here. The format follows
[Keep a Changelog](https://keepachangelog.com/en/1.1.0/), and versions follow
[Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

<!-- git-cliff: end of header -->
## [0.5.0] - 2026-09-16

### Added

- Published a signed `packslip.sigstore.json` manifest beside each release so docgraph
  installs through mise's packslip backend
  (`mise use packslip:github.com/JTarasovic/docgraph`), with SHA-256 checksum and GitHub
  attestation parity to the existing cargo-dist archives (#44).

### Changed

- Overhauled the README and split contributor and release documentation into dedicated
  guides (#43).

## [0.4.1] - 2026-09-14

### Changed

- Published the verified installation actions, release attestations, SBOM coverage,
  and rustls security update described in 0.4.0 below. The 0.4.0 tag produced no
  release artifacts because the Release workflow was disabled; this release uses
  a new immutable tag after enabling that workflow.

## [0.4.0] - 2026-09-14

### Added

- Added reusable actions to install a released CLI or the pinned native runtime
  with checksum and producer-attestation verification.
- Added native-runtime SBOMs and attestations, and product-release attestation
  coverage for archives, checksums, and the Rust workspace SBOM.

### Changed

- Simplified validation and release automation around the shared installers.
  The validation action installs the latest stable docgraph release by default;
  an exact version or an existing installation can also be selected.

### Fixed

- Excluded companion-runtime releases when resolving the latest docgraph version.
- Accepted Windows line endings when preparing changelog comparison links.

### Security

- Updated rustls to address RUSTSEC-2026-0285.

## [0.3.0] - 2026-09-01
### Added

- Added safe repair for invalid typed properties without weakening repository policy.
- Exposed the complete logic predicate vocabulary for discoverable custom queries.
- Added lossless YAML-frontmatter migration and expanded query and section inspection.
- Made CLI help and portable agent guidance teach supported authoring and recovery flows.

### Changed

- Replaced hand-maintained release packaging with pinned cargo-release, git-cliff, and
  dist automation, including native smoke gates and conventional change descriptions.

### Fixed

- Rejected nonportable entity IDs before writes.
- Resolved repository-relative Markdown links consistently.

## [0.2.0] - 2026-08-31

### Added

- Added portable agent guidance and a checksum-verifying repository validation action.
- Added external GitHub issue references with cached, stale, and offline behavior.
- Added repository initialization and the first v1-readiness plans.

### Fixed

- Preserved quoted string properties and masked logic comments correctly.

## [0.1.0] - 2026-08-28

### Added

- Published the first docgraph CLI with graph validation, querying, managed mutations,
  and native Windows and Linux release bundles.

[Unreleased]: https://github.com/JTarasovic/docgraph/compare/v0.5.0..HEAD
[0.5.0]: https://github.com/JTarasovic/docgraph/compare/v0.4.1..v0.5.0
[0.4.1]: https://github.com/JTarasovic/docgraph/compare/v0.4.0..v0.4.1
[0.4.0]: https://github.com/JTarasovic/docgraph/compare/v0.3.0..v0.4.0
[0.3.0]: https://github.com/JTarasovic/docgraph/compare/v0.2.0..v0.3.0
[0.2.0]: https://github.com/JTarasovic/docgraph/compare/v0.1.0..v0.2.0
[0.1.0]: https://github.com/JTarasovic/docgraph/tree/v0.1.0
