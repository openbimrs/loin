# Changelog

All notable changes to this project are documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/),
and this project follows [Semantic Versioning](https://semver.org/).

## [Unreleased]

### Added

- Added a strict, bounded XML 1.0 `LoinDocument` decoder and writer retaining
  qualified names, namespace declarations, attribute/node order, comments,
  processing instructions, CDATA, empty-element style, unknown content, and
  represented control-character semantics.
- Added explicit 2022/2024 namespace migration with observed/current namespace
  tracking, precise reports, non-LOIN namespace isolation, and fail-closed
  expanded-attribute collision detection.
- Added XSD-derived ISO 7817-3 structural and lexical validation with stable,
  path-aware diagnostics for sequences, repeating choices, cardinalities,
  inherited attributes, enumerations, XSI nil semantics, booleans, exact
  XML Schema doubles, and DT-owned scalar values.
- Added synthetic adversarial, migration, validation-coverage, and repeated
  semantic round-trip tests without including restricted standards artifacts.
- Expanded the typed domain model to represent the current schema's ordered
  repeating purpose choice, actor contact/identity fields with restricted email,
  alphanumerical GUID and optional empty-capable group container, current
  geometry/detail/location branches, required-document state, datum registry
  references, and specification-level georeferencing state.

### Changed

- Relicensed repository-authored work from MIT to `AGPL-3.0-or-later`; historical releases remain under their published MIT terms, and third-party material retains its own terms.
- Added direct `quick-xml` and `roxmltree` dependencies for the format-specific
  syntax tree and strict XML preflight while retaining dependency direction
  `openbim-loin -> openbim-dt`.
- Documented validation exactly as clause-level/XSD-derived; imported ISO 23387
  complex-type internals are retained but are not claimed as fully XSD-validated.

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
