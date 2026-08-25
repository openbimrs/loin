# Changelog

All notable changes to this project are documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/),
and this project follows [Semantic Versioning](https://semver.org/).

## [Unreleased]

### Changed

- Extracted the LOIN family into its canonical standalone repository while
  preserving its OpenBIM.rs path history.
- Made package and dependency metadata independent of the integration workspace.
- Added standalone documentation, CI, package verification, and an executable
  semantic purity gate for the `loin` alias package.
- Added isolated mutation probes for target-qualified dependencies, feature
  drift, version/path drift, alternate targets, alias-owned code, and extra
  implementation files.

## [0.1.0] - 2026-08-24

### Added

- Reserved the `openbim-loin` and `loin` package names.
- Added constants for the 2024 and 2022 draft namespace URIs.
- Added known-namespace recognition.
- Established `loin` as a pure re-export of the canonical package.

[Unreleased]: https://github.com/openbimrs/loin/commits/main
[0.1.0]: https://crates.io/crates/openbim-loin/0.1.0
