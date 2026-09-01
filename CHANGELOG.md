# Changelog

All notable changes to docgraph are documented here. The format follows
[Keep a Changelog](https://keepachangelog.com/en/1.1.0/), and versions follow
[Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

<!-- git-cliff: end of header -->
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

[Unreleased]: https://github.com/JTarasovic/docgraph/compare/v0.3.0..HEAD
[0.3.0]: https://github.com/JTarasovic/docgraph/compare/v0.2.0..v0.3.0
[0.2.0]: https://github.com/JTarasovic/docgraph/compare/v0.1.0..v0.2.0
[0.1.0]: https://github.com/JTarasovic/docgraph/tree/v0.1.0
