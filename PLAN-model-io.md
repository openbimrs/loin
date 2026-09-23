# PLAN - typed model I/O (reader + complete writer)

Durable plan for a multi-session effort across openbim-dt and openbim-loin.
Re-read this file after any context compression; conversation is not durable.

## Goal

Every typed-model type round-trips: document -> model -> document, with
the written output validating and byte-stable on re-write.

## Findings (2026-09-23, verified in source)

- No reader exists. Nothing maps LoinDocument to LevelOfInformationNeed.
- per_object_element always errs: ObjectType has no DT serializer.
- model.rs from line 667 (728 of 1395 lines) is unreachable from XML.
- The 11 model enums match validator tables but have no string conversion.
- GeoReferencing: now written (fixed 2026-09-23, commit pending).

## Design decision: codec lives in the crate that owns the type

DT owned types (ObjectType, Property, Unit, ...) gain to_element and
from_element in openbim-dt. LOIN never re-encodes ISO 23387 grammar.
Why: the typed model has one owner per type, and dt already owns the
generated ISO 23387 schema tables, so its writer can be checked against
Document::validate_schema. A LOIN-side copy would drift.

## Open question before dt code: element tree bridge

dt::Element and loin XmlElement are different trees. The DT codec must
emit something LOIN can embed. Resolve by reading both before designing.

## Phases (each ends in a gated, pushed commit)

1. dt: codec for owned types; round-trip + validate_schema tests.
2. dt: release (minor, additive); verify on crates.io.
3. loin: enum string conversions (as_str/FromStr) from validator tables.
4. loin: reader LoinDocument -> model, returning diagnostics not panics.
5. loin: writer completes per-object + geometry via dt codec.
6. loin: round-trip property test over every fixture; cardinality
   derived from validation.rs tables, not restated.
7. loin: release 0.4.0 (reader is new public API).

## Status

- [x] cheap fixes: geo writer, evidence ledger, lockfile check, lockstep
      release check, docs + ADRs 0001-0003 (gate running 2026-09-23)
- [ ] phase 1 (dt codec) - next
