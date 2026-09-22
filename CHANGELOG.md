# Changelog

All notable changes to this project are documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/),
and this project follows [Semantic Versioning](https://semver.org/).

## [Unreleased]

## [0.3.2] - 2026-09-21

### Fixed

- `Purpose`'s singular setters (`set_definition`, `set_language`, `set_region`,
  `set_dictionary_ref`) no longer delete later occurrences of the branch they
  set. The XSD choice is `maxOccurs="unbounded"`, so a repeated `Language`,
  `Region`, `Definition` or `DictionaryRef` is valid authored content, and the
  setters were silently discarding it. They now replace the first occurrence
  in place and leave the rest untouched. Resolves #5.

### Added

- `Purpose::set_all_definitions`, `set_all_languages`, `set_all_regions` and
  `set_all_dictionary_refs`, the explicit opt-in for callers that mean "this
  Purpose has exactly one of these". Each returns the number of duplicate
  branches removed, so a no-op is distinguishable from real data loss.

## [0.3.1] - 2026-09-21

### Fixed

- ISO 23387 multilingual text (`dt:Name`, `dt:Definition`, `dt:Description`,
  `dt:Example`) now has its required `language` attribute validated.
  `validate_dt_element` was only reachable for a `dt`-namespaced root, which
  the root check already rejects, so the attribute was never inspected in
  either direction. The check moved to `validate_imported_dt_subtree`, which
  is on the live path. Resolves #2.
- `ChildOutOfOrder` now reports the offending child's path instead of the
  parent's, and `child_path` is computed once per child so the trailing index
  means the same thing in every diagnostic. Resolves #6.

### Added

- `Display` and `std::error::Error` for `Diagnostic`, and `Display` for
  `EmailAddress`, so consumers no longer hand-roll formatting. Resolves #7.

### Removed

- `DiagnosticCode::InvalidInteger`, which was never constructed. The ISO 7817-3
  XSD declares no integer-typed attribute or element (only `xs:boolean`,
  `xs:dateTime`, `xs:decimal`, `xs:double`, `xs:language` and `xs:string`), so
  the variant was unreachable by construction and forced consumers matching
  exhaustively to carry a dead branch. Resolves #7.

## [0.3.0] - 2026-09-21

### Changed

- Relicensed repository-authored work from `AGPL-3.0-or-later` back to `MIT`.
  No version was ever published under the AGPL — the relicense landed after
  `0.2.0` and was reverted before any release, so every published version of
  `openbim-loin` and `loin` is MIT. `LICENSING.md` records the version
  boundaries. Resolves #11.

### Added

- `Purpose::names`, `definitions`, `descriptions`, `reference_documents`,
  `regions`, and `dictionary_refs`, which expose every branch of the repeating
  `PurposeType` choice. The XSD declares the choice `maxOccurs="unbounded"`,
  so branches may legitimately repeat and the singular accessors return only
  the first occurrence.
- `tests/schema_conformance.rs`, pinning grammar facts read from the official
  ISO 7817-3 Annex B XSD.
- `LoinDocument::from_model` converts a `LevelOfInformationNeed` into a
  document, so LOIN files can be authored and not only parsed. Output is
  written in schema sequence order and revalidates after a reparse.
- Public construction and mutation API on the document tree: `XmlElement::new`,
  `new_root`, `new_dt`, `with_attribute`, `with_child`, `with_text`,
  `as_empty_element`, `push_node`, `nodes_mut`, `attributes_mut`,
  `XmlAttribute::new` / `new_dt` / `namespace_declaration`, and
  `LoinDocument::root_mut`.
- `AuthoringError`, which names the element that could not be written and why,
  rather than silently dropping content.
- Getters for previously write-only values: `InformationDeliveryMilestone`
  (`descriptions`, `reference_documents`, `date`), `DocumentFormat` (`names`,
  `versions`, `specifications`), and `Document` (`name`, `descriptions`,
  `format`).

### Fixed

- `ShapeInfluence` field order now matches the XSD `xs:sequence`
  (`ThresholdDimension` last), so a serializer walking fields in declaration
  order emits valid document order.
- `Purpose` setters (`set_definition`, `set_language`, `set_region`,
  `set_dictionary_ref`) now replace an existing item in place instead of
  removing it and appending the replacement, which silently reordered the
  document.

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
