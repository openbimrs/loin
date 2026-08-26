# Changelog

All notable changes to this project are documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/),
and this project follows [Semantic Versioning](https://semver.org/).

## [Unreleased]

## [0.2.0] - 2026-08-25

### Added

- Added a direct `openbim-dt 0.2` dependency and re-exported it as
  `openbim_loin::dt` to preserve one DT type identity for consumers.
- Added DT-backed purpose, actor, milestone, format/document, threshold,
  per-object specification, and alphanumerical-information contracts.
- Added compile-time coverage for every ISO 23387 type/value family referenced
  by the audited ISO 7817-3 schema and a mutation probe that rejects replacement
  with LOIN-owned lookalikes.
- Added mutation coverage for the audited zero-cardinality collections, anonymous
  alphanumerical-information content, optional datum descriptions, and the DT
  `DateTime` milestone attribute.

### Changed

- Extracted the LOIN family into its canonical standalone repository while
  preserving its OpenBIM.rs path history.
- Made package and dependency metadata independent of the integration workspace.
- Added standalone documentation, CI, package verification, and an executable
  semantic purity gate for the `loin` alias package.
- Added isolated mutation probes for target-qualified dependencies, feature
  drift, version/path drift, alternate targets, alias-owned code, and extra
  implementation files.
- Bumped the synchronized `openbim-loin` and pure `loin` alias releases to
  `0.2.0`; complete LOIN XML parsing and validation remain out of scope.
- Removed the unused scaffold-only `openbim-core` dependency; the implemented
  boundary depends only on the DT contracts it consumes.

## [0.1.0] - 2026-08-24

### Added

- Reserved the `openbim-loin` and `loin` package names.
- Added constants for the 2024 and 2022 draft namespace URIs.
- Added known-namespace recognition.
- Established `loin` as a pure re-export of the canonical package.

[Unreleased]: https://github.com/openbimrs/loin/compare/v0.2.0...HEAD
[0.2.0]: https://github.com/openbimrs/loin/compare/v0.1.0...v0.2.0
[0.1.0]: https://crates.io/crates/openbim-loin/0.1.0
