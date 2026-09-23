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

## Finding: DT subtree shape inside LOIN (2026-09-23)

Outer element is LOIN-local, unqualified (<ObjectType>); its content is
DT-qualified (<dt:Name/>, dt:GUID). dt's codec therefore takes the outer
name as a parameter and owns only the content model.

## Finding: dt owned model is a lossy subset (2026-09-23)

ConceptType declares 15 child kinds + @about; owned Concept keeps 4.
Narrowed: Definition 0..* -> 1; IsSubtypeOfRef 0..* -> Option.
Property.DimensionRef 0..* -> Option. Type-specific content otherwise
matches. ConceptType content is a repeating choice (order free), so
typed values need no order; writers emit declared order.

Decision: dt ROADMAP milestone 3 - complete owned contracts for all
declared content, widen narrowed cardinalities, add a public dt::Element
builder, and to_element/from_element per owned type.

## Phase 1 delegated (2026-09-23)

Brief: /home/friedrich/.cache/briefs/dt-codec-brief.md
Worker branch: dt feature/owned-codec (.worktrees/codec), local only.
Next for me: review diff + evidence, re-run gate, then push/release dt.

- [x] phase 3 (enums single-sourced) - fb2ab3e, local; gate before push

## Reader design (phase 4), decided 2026-09-23

API: LevelOfInformationNeed::from_document(&LoinDocument)
     -> Result<Self, ReadError>. Strict, fails closed.
ReadError = stable kind + element path (same path format as Diagnostic).
Reader does not re-validate: validate() stays the diagnostics API. The
reader refuses what the model cannot represent, naming where.
Round-trip contract: model -> doc -> model is identity. doc -> model is
a semantic projection (drops extensions, comments); the XML tree keeps
those, so lossless editing stays on LoinDocument.
DT bridge: LOIN converts XmlElement <-> dt::Element with dt's new public
builder (phase 1), then calls dt from_element/to_element. The adapter
lives in LOIN; dt stays unaware of LOIN.

- [~] phase 1 (dt codec): worker stalled after step 1 (builder only,
      uncommitted, one namespace bug). Reviewed, fixed, tested, committed
      as dt d159cdb on feature/owned-codec (local). Steps 2-3 remain.

- [x] phase 1 (dt owned codec) - openbim-dt 0.3.0 published 2026-09-23,
      tag v0.3.0 at 05ddf02. CI + docs green; registry build verified.
      Loin still pins ^0.2: bump to 0.3 is the first step of phase 4.
      Phase-1 worker stalled; the codec was written directly, not delegated.
